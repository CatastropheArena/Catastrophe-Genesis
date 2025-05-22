// Copyright (c), Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::externals::{duration_since, get_latest_checkpoint_timestamp, get_reference_gas_price, fetch_first_and_last_pkg_id};
use crate::metrics::{observation_callback, status_callback};
use crate::metrics::{start_basic_prometheus_server, Metrics};
use crate::types::{IbeMasterKey, Network};
use anyhow::Result;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;
use dotenv::dotenv;
use fastcrypto::ed25519::Ed25519KeyPair;
use fastcrypto::encoding::{Base64, Encoding};
use fastcrypto::serde_helpers::ToFromByteArray;
use fastcrypto::traits::KeyPair;
use rand::rngs::StdRng;
use rand::SeedableRng;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use sui_sdk::types::base_types::ObjectID;
use sui_sdk::SuiClient;
use sui_sdk::SuiClientBuilder;
use tokio::sync::watch::channel;
use tokio::sync::watch::Receiver;
use tracing::{info, Level};
use tracing_subscriber::{fmt, EnvFilter};
use crate::sdk::GameManager;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub mod app;
pub mod avatars; // 头像模块
pub mod cache; // 缓存系统，优化性能
pub mod catastrophe; // 游戏模块
pub mod chat; // 聊天系统
pub mod cli; // 命令行接口
pub mod common;
pub mod errors; // 错误类型定义
pub mod externals; // 外部接口，如时间和gas价格
pub mod game; // 游戏模块
pub mod gaming; // 游戏匹配模块
pub mod keys; // 密钥服务器模块
pub mod metrics;
pub mod passport; // 用户护照系统
pub mod signed_message; // 签名消息处理
#[cfg(test)]
pub mod tests;
pub mod tool; // 游戏工具模块
pub mod txb; // 事务构建模块
pub mod types; // 数据类型定义
pub mod valid_ptb; // 可编程交易块验证 // 测试模块
pub mod ws; // WebSocket 会话管理模块
pub mod sdk; // SUI SDK 模块

/// 更新最新检查点时间戳的间隔
const CHECKPOINT_UPDATE_INTERVAL: Duration = Duration::from_secs(10);
/// 更新参考gas价格的间隔
const GAS_PRICE_UPDATE_INTERVAL: Duration = Duration::from_secs(60);
/// 更新Citadel包ID的间隔
const PACKAGE_ID_UPDATE_INTERVAL: Duration = Duration::from_secs(1800); // 30分钟检查一次
/// 更新Profile的间隔
const PROFILE_UPDATE_INTERVAL: Duration = Duration::from_secs(30); // 30秒检查一次
/// 时间戳类型（64位无符号整数）
pub type Timestamp = u64;

/// App state, at minimum needs to maintain the ephemeral keypair.  
pub struct AppState {
    /// Ephemeral keypair on boot
    pub eph_kp: Ed25519KeyPair,
    /// Config Mapping
    pub config: HashMap<String, String>,
    /// 网络类型
    pub network: Network,
    /// Metrics
    pub metrics: Metrics,
    /// SUI客户端（可选，为密钥服务器功能）
    pub sui_client: SuiClient,
    /// IBE主密钥（可选，为密钥服务器功能）
    pub master_key: types::IbeMasterKey,
    /// 密钥服务器对象ID（可选，为密钥服务器功能）
    pub key_server_object_id: ObjectID,
    /// 主密钥持有证明（可选，为密钥服务器功能）
    pub key_server_object_id_sig: types::MasterKeyPOP,
    /// 最新检查点时间戳接收器（可选，为密钥服务器功能）
    pub latest_checkpoint_timestamp_receiver: Receiver<Timestamp>,
    /// 参考gas价格接收器（可选，为密钥服务器功能）
    pub reference_gas_price: Receiver<u64>,
    /// Citadel包ID更新接收器
    pub citadel_package_id_receiver: Receiver<String>,
    /// 游戏数据管理器
    pub game_manager: Arc<GameManager>,
}

impl AppState {
    pub async fn new() -> Self {
        // 初始化环境变量
        dotenv().ok();
        info!("Init tracing logger, level: {:?}", Level::INFO);
        // 生成临时密钥对
        let eph_kp = Self::generate_keypair(None);
        info!("Generate ephemeral keypair: {:?}", eph_kp);
        let network = Self::init_network();
        // 加载环境变量
        let config = Self::load_env_vars(&[
            "API_KEY",
            "MASTER_KEY",
            "KEY_SERVER_OBJECT_ID",
            "CITADEL_PACKAGE",
            "CITADEL_MANAGER_ADDRESS",
            "CITADEL_FRIENDSHIP_ADDRESS",
            "CITADEL_ADMINCAP_ADDRESS",
        ]);
        info!("Load env vars: {:?}", config);
        // 初始化SUI客户端
        let sui_client = SuiClientBuilder::default()
            .build(&network.node_url())
            .await
            .expect(format!("Sui client build failed with {:?}", network.node_url()).as_str());
        info!("Sui client build success, node url: {:?},graphql url: {:?}, network: {:?}, api version: {:?}", network.node_url(), network.graphql_url(), network, sui_client.api_version());
        // 初始化主密钥和服务器ID
        let master_key = IbeMasterKey::from_byte_array(
            &Base64::decode(&config["MASTER_KEY"])
                .expect("MASTER_KEY should be base64 encoded")
                .try_into()
                .expect("Invalid MASTER_KEY length"),
        )
        .expect("Invalid MASTER_KEY value");
        // 初始化密钥服务器对象ID
        let key_server_object_id = ObjectID::from_hex_literal(&config["KEY_SERVER_OBJECT_ID"])
            .expect("Invalid KEY_SERVER_OBJECT_ID");
        let key_server_object_id_sig = crypto::ibe::create_proof_of_possession(
            &master_key,
            &key_server_object_id.into_bytes(),
        );
        info!(
            "Key server object id: {:?} , signature: {:?}",
            key_server_object_id, key_server_object_id_sig
        );
        // 初始化ProfileManager
        let manager_store_id = ObjectID::from_hex_literal(&config["CITADEL_MANAGER_ADDRESS"])
            .expect("Invalid CITADEL_MANAGER_ADDRESS");
        // 初始化GameManager
        let game_manager = Arc::new(GameManager::new(
            sui_client.clone(),
            network.clone(),
            manager_store_id,
        ).await.unwrap());
        // 启动指标服务器,创建分组的metrics
        let registry_service = start_basic_prometheus_server(None);
        let metrics = create_metrics! {
            &registry_service,
            // 请求和错误指标组
            [MetricGroup::Requests, MetricGroup::Errors] => "requests",
            // 时间和延迟指标组
            [
                MetricGroup::CheckpointTimestampDelay,
                MetricGroup::GetCheckpointTimestampDuration,
                MetricGroup::GetCheckpointTimestampStatus,
                MetricGroup::GetReferenceGasPriceStatus,
                MetricGroup::CheckPolicyDuration,
                MetricGroup::FetchPkgIdsDuration,
                MetricGroup::RequestsPerNumberOfIds
            ] => "monitoring"
        };
        info!(
            "Metrics initialized with {} groups",
            registry_service.count_registries()
        );
        let citadel_package_receiver = channel(config["CITADEL_PACKAGE"].clone()).1;
        AppState {
            eph_kp,
            config,
            network,
            metrics,
            sui_client: sui_client.clone(),
            master_key,
            key_server_object_id,
            key_server_object_id_sig,
            latest_checkpoint_timestamp_receiver: channel(0).1,
            reference_gas_price: channel(0).1,
            citadel_package_id_receiver: citadel_package_receiver,
            game_manager,
        }
    }

    pub fn init_network() -> Network {
        // 初始化网络
        let network = env::var("NETWORK")
            .ok()
            .and_then(|n| if n.is_empty() { None } else { Some(n) })
            .map(|n| Network::from_str(&n))
            .unwrap_or(Network::Testnet);
        info!("Network: {:?}", network);
        network
    }

    /// 加载环境变量
    fn load_env_vars(keys: &[&str]) -> HashMap<String, String> {
        let mut config = HashMap::new();
        for key in keys {
            let value = env::var(key).unwrap_or_else(|_| {
                panic!("Environment variable `{}` must be set", key);
            });
            config.insert(key.to_string(), value);
        }
        config
    }

    /// 生成密钥对
    ///
    /// 根据环境变量SEED控制是否使用固定种子
    ///
    /// # 参数
    ///
    /// * `seed` - 可选的指定种子值，优先级高于环境变量
    ///
    /// # 返回值
    ///
    /// 生成的Ed25519密钥对
    pub fn generate_keypair(seed: Option<u64>) -> Ed25519KeyPair {
        // 优先使用参数提供的种子，其次尝试从环境变量读取
        if let Some(seed_value) =
            seed.or_else(|| env::var("SEED").ok().and_then(|s| s.parse::<u64>().ok()))
        {
            // 使用固定种子
            info!("Use fixed seed: {}", seed_value);
            let mut rand = StdRng::seed_from_u64(seed_value);
            Ed25519KeyPair::generate(&mut rand)
        } else {
            // 使用随机种子
            info!("Use random seed");
            Ed25519KeyPair::generate(&mut rand::thread_rng())
        }
    }

    /**
     * 检查全节点数据是否新鲜
     *
     * 验证最新检查点时间戳是否在允许的过时时间范围内
     *
     * 参数:
     * @param allowed_staleness - 允许的过时时间
     *
     * 返回:
     * 成功时返回Ok(())，如果数据过时则返回错误
     */
    pub fn check_full_node_is_fresh(
        &self,
        allowed_staleness: std::time::Duration,
    ) -> Result<(), errors::InternalError> {
        let staleness =
            externals::duration_since(*self.latest_checkpoint_timestamp_receiver.borrow());
        if staleness > allowed_staleness.as_millis() as i64 {
            tracing::warn!(
                "Full node is stale. Latest checkpoint is {} ms old.",
                staleness
            );
            return Err(errors::InternalError::Failure);
        }
        Ok(())
    }
    /**
     * 获取当前参考gas价格
     *
     * 返回:
     * 当前gas价格
     */
    fn reference_gas_price(&self) -> u64 {
        *self.reference_gas_price.borrow()
    }

    /**
     * 生成定期更新器
     *
     * 启动一个线程，定期获取值并将其发送到接收器
     * 用于维护服务器状态，如最新检查点时间和gas价格
     *
     * 参数:
     * @param sui_client - SUI客户端
     * @param update_interval - 更新间隔
     * @param fetch_fn - 获取值的函数
     * @param value_name - 值名称（用于日志）
     * @param subscriber - 值更新时的回调
     * @param duration_callback - 持续时间回调
     * @param success_callback - 成功回调
     *
     * 返回:
     * 包含更新值的接收器
     */
    async fn spawn_periodic_updater<F, Fut, G, H, I>(
        sui_client: sui_sdk::SuiClient,
        update_interval: Duration,
        fetch_fn: F,
        value_name: &'static str,
        subscriber: Option<G>,
        duration_callback: Option<H>,
        success_callback: Option<I>,
    ) -> tokio::sync::watch::Receiver<u64>
    where
        F: Fn(sui_sdk::SuiClient) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = sui_sdk::error::SuiRpcResult<u64>> + Send,
        G: Fn(u64) + Send + 'static,
        H: Fn(Duration) + Send + 'static,
        I: Fn(bool) + Send + 'static,
    {
        let (sender, mut receiver) = channel(0);
        let local_client = sui_client.clone();
        let mut interval = tokio::time::interval(update_interval);

        // 如果由于全节点响应缓慢而错过了一个tick，我们不需要赶上来，而是延迟下一个tick。
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        tokio::task::spawn(async move {
            loop {
                let now = std::time::Instant::now();
                let result = fetch_fn(local_client.clone()).await;
                if let Some(dcb) = &duration_callback {
                    dcb(now.elapsed());
                }
                if let Some(scb) = &success_callback {
                    scb(result.is_ok());
                }
                match result {
                    Ok(new_value) => {
                        sender
                            .send(new_value)
                            .expect("Channel closed, this should never happen");
                        tracing::debug!("{} updated to: {:?}", value_name, new_value);
                        if let Some(subscriber) = &subscriber {
                            subscriber(new_value);
                        }
                    }
                    Err(e) => tracing::warn!("Failed to get {}: {:?}", value_name, e),
                }
                interval.tick().await;
            }
        });

        // 这会阻塞直到获取到一个值。
        // 这样做是为了确保服务器在启动后立即可以处理请求。
        // 如果这不可能，我们无法更新值，服务器不应该启动。
        receiver
            .changed()
            .await
            .unwrap_or_else(|_| panic!("Failed to get {}", value_name));
        receiver
    }

    /**
     * 生成最新检查点时间戳更新器
     *
     * 定期获取最新的检查点时间戳，用于确保服务器使用最新数据
     *
     * 参数:
     * @param app_state - 应用状态，包含SUI客户端和性能指标
     *
     * 返回:
     * 包含检查点时间戳的接收器
     */
    pub async fn spawn_latest_checkpoint_timestamp_updater(
        app_state: &mut AppState,
        interval: Option<Duration>,
    ) -> Receiver<Timestamp> {
        // 启动定期更新任务
        app_state.latest_checkpoint_timestamp_receiver = Self::spawn_periodic_updater(
            app_state.sui_client.clone(),
            interval.unwrap_or(CHECKPOINT_UPDATE_INTERVAL),
            get_latest_checkpoint_timestamp,
            "latest checkpoint timestamp",
            Some(observation_callback(
                &app_state.metrics.checkpoint_timestamp_delay,
                |ts| duration_since(ts) as f64,
            )),
            Some(observation_callback(
                &app_state.metrics.get_checkpoint_timestamp_duration,
                |d: Duration| d.as_millis() as f64,
            )),
            Some(status_callback(
                &app_state.metrics.get_checkpoint_timestamp_status,
            )),
        )
        .await;
        app_state.latest_checkpoint_timestamp_receiver.clone()
    }

    /**
     * 生成参考gas价格更新器
     *
     * 定期获取当前的参考gas价格，用于交易模拟
     *
     * 参数:
     * @param app_state - 应用状态，包含SUI客户端和性能指标
     *
     * 返回:
     * 包含gas价格的接收器
     */
    pub async fn spawn_reference_gas_price_updater(
        app_state: &mut AppState,
        interval: Option<Duration>,
    ) -> Receiver<u64> {
        app_state.reference_gas_price = Self::spawn_periodic_updater(
            app_state.sui_client.clone(),
            interval.unwrap_or(GAS_PRICE_UPDATE_INTERVAL),
            get_reference_gas_price,
            "RGP",
            None::<fn(u64)>,
            None::<fn(Duration)>,
            Some(status_callback(
                &app_state.metrics.get_reference_gas_price_status,
            )),
        )
        .await;
        app_state.reference_gas_price.clone()
    }

    /**
     * 更新Citadel包ID
     * 
     * 定期检查并更新Citadel包ID的最新版本，确保RPC调用使用最新的包ID
     * 通过watch channel通知其他组件配置已更新
     * 
     * 参数:
     * @param app_state - 应用状态，包含SUI客户端和配置信息
     * @param interval - 可选的更新间隔，如未指定则使用默认值
     * 
     * 返回:
     * 包含最新包ID的接收器
     */
    pub async fn spawn_package_id_updater(
        app_state: &mut AppState,
        interval: Option<Duration>,
    ) -> tokio::sync::watch::Receiver<String> {
        // 确保配置中存在CITADEL_PACKAGE键
        if !app_state.config.contains_key("CITADEL_PACKAGE") {
            tracing::warn!("CITADEL_PACKAGE configuration not found, cannot start package ID updater");
            return tokio::sync::watch::channel(String::new()).1;
        }

        let pkg_id_str = app_state.config["CITADEL_PACKAGE"].clone();
        
        // 创建channel，初始值为当前配置的包ID
        let (sender, receiver) = tokio::sync::watch::channel(pkg_id_str.clone());
        
        // 尝试将包ID转换为ObjectID
        match ObjectID::from_hex_literal(&pkg_id_str) {
            Ok(pkg_id) => {
                let update_interval = interval.unwrap_or(PACKAGE_ID_UPDATE_INTERVAL);
                let network = app_state.network.clone();
                
                // 启动更新任务
                tokio::task::spawn(async move {
                    let mut interval = tokio::time::interval(update_interval);
                    
                    loop {
                        interval.tick().await;
                        
                        // 获取最新的包ID
                        if let Ok((_, latest)) = fetch_first_and_last_pkg_id(&pkg_id, &network).await {
                            // 检查是否需要更新
                            if latest != pkg_id && sender.send(latest.to_string()).is_ok() {
                                tracing::info!("Citadel package ID updated: {} -> {}", pkg_id, latest);
                            }
                        }
                    }
                });
                
                tracing::info!("Citadel package ID updater started, initial package ID: {}", pkg_id);
            },
            Err(e) => {
                tracing::error!("Failed to parse CITADEL_PACKAGE value: {}", e);
            }
        }
        
        receiver
    }

    /**
     * 启动档案更新器
     * 
     * 定期更新所有用户档案信息，确保数据的实时性
     * 
     * 参数:
     * @param app_state - 应用状态，包含游戏管理器
     * @param interval - 可选的更新间隔，如未指定则使用默认值
     * 
     * 返回:
     * 包含当前profiles数量的接收器
     */
    pub async fn spawn_profile_updater(
        app_state: &mut AppState,
        interval: Option<Duration>,
    ) -> tokio::sync::watch::Receiver<u64> {
        // 获取初始profiles数量
        let initial_count = app_state.game_manager.get_profile_size().await.unwrap_or(0);
        
        // 创建channel，初始值为当前profiles数量
        let (sender, receiver) = tokio::sync::watch::channel(initial_count);
        
        let update_interval = interval.unwrap_or(PROFILE_UPDATE_INTERVAL);
        let game_manager = app_state.game_manager.clone();

        // 启动更新任务
        tokio::task::spawn(async move {
            loop {
                // 计算距离上次更新的时间
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let last = game_manager.get_last_update();
                let elapsed = now - last;

                // 如果距离上次更新时间小于间隔，则等待剩余时间
                if elapsed < update_interval.as_secs() {
                    let wait_time = update_interval.as_secs() - elapsed;
                    tokio::time::sleep(Duration::from_secs(wait_time)).await;
                }

                // 更新所有profiles
                if let Err(e) = game_manager.update_all_profiles().await {
                    tracing::warn!("Failed to update user profiles: {}", e);
                }
                
                // 获取最新的profiles数量
                if let Ok(count) = game_manager.get_profile_size().await {
                    if sender.send(count).is_ok() {
                        tracing::debug!("Profiles count updated: {}", count);
                    }
                }
            }
        });
        
        tracing::info!(
            "Profile updater started, initial profiles count: {}, update interval: {} seconds, last update time: {}", 
            initial_count, 
            update_interval.as_secs(),
            app_state.game_manager.get_last_update()
        );
        
        receiver
    }
    
    /**
     * 获取当前Citadel包ID
     * 
     * 从接收器中获取最新的Citadel包ID
     * 
     * 返回:
     * 当前包ID字符串
     */
    pub fn citadel_package_id(&self) -> String {
        self.citadel_package_id_receiver.borrow().clone()
    }
}

/// Implement IntoResponse for EnclaveError.
impl IntoResponse for EnclaveError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            EnclaveError::GenericError(e) => (StatusCode::BAD_REQUEST, e),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

/// Enclave errors enum.
#[derive(Debug)]
pub enum EnclaveError {
    GenericError(String),
}

/// 简化 Metrics 构建与注册表映射的宏
///
/// 基于枚举的类型安全实现，支持编译期检查和IDE自动补全
#[macro_export]
macro_rules! create_metrics {
    // 基本形式：只传入注册表服务
    ($service:expr) => {{
        use $crate::metrics::MetricsBuilder;
        MetricsBuilder::from_registry_service($service)
            .build()
            .expect("Failed to build metrics")
    }};

    // 带组映射的形式：指定哪些指标组使用特定注册表
    ($service:expr, $([$($group:expr),+ $(,)?] => $registry_name:expr),* $(,)?) => {{
        use $crate::metrics::{MetricsBuilder, MetricGroup};

        let service = $service;
        let mut builder = MetricsBuilder::from_registry_service(&service);

        $(
            // 为每个注册表名称创建一个注册表
            let (registry, _) = service.create_registry(Some($registry_name));

            // 将每个指标组与此注册表关联
            $(
                builder = builder.with_registry_for($group, registry.clone());
            )+
        )*

        builder.build().expect("Failed to build metrics with mappings")
    }};
}

/// 初始化日志
pub fn init_tracing_logger() {
    // 配置日志系统
    let env_filter = EnvFilter::from_default_env()
        .add_directive(Level::INFO.into())
        .add_directive("nautilus_server=debug".parse().unwrap());

    fmt::fmt()
        .with_env_filter(env_filter)
        .with_target(true)
        .init();
    info!("Log system initialized");
}
