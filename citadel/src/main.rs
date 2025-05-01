// Copyright (c), Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use axum::{routing::get, routing::post, Router};
use crypto::elgamal::encrypt;
use crypto::ibe;
use crypto::ibe::create_proof_of_possession;
use fastcrypto::ed25519::{Ed25519PublicKey, Ed25519Signature};
use fastcrypto::encoding::{Base64, Encoding};
use fastcrypto::serde_helpers::ToFromByteArray;
use fastcrypto::traits::VerifyingKey;
use fastcrypto::{ed25519::Ed25519KeyPair, traits::KeyPair};

use nautilus_server::app::process_data;
use nautilus_server::common::{get_attestation, health_check};
use nautilus_server::errors::InternalError;
use nautilus_server::externals::{
    current_epoch_time, duration_since, get_latest_checkpoint_timestamp, get_reference_gas_price,
};
use nautilus_server::keys;
use nautilus_server::metrics::{
    call_with_duration, observation_callback, start_basic_prometheus_server, status_callback,
    Metrics,
};
use nautilus_server::signed_message::{signed_message, signed_request};
use nautilus_server::types::{
    ElGamalPublicKey, ElgamalEncryption, ElgamalVerificationKey, IbeMasterKey, MasterKeyPOP,
    Network,
};
use nautilus_server::valid_ptb::ValidPtb;
use nautilus_server::AppState;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use sui_sdk::types::base_types::{ObjectID, SuiAddress};
use sui_sdk::SuiClientBuilder;
use tokio::sync::watch::channel;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, Level};
use tracing_subscriber::{fmt, EnvFilter};

/// 允许的全节点数据过时时间
/// 设置此持续时间时，注意Sui上的时间戳可能比当前时间稍晚，但不应超过一秒。
const ALLOWED_STALENESS: Duration = Duration::from_secs(120);

/// 更新最新检查点时间戳的间隔
const CHECKPOINT_UPDATE_INTERVAL: Duration = Duration::from_secs(10);

/// 更新参考gas价格的间隔
const RGP_UPDATE_INTERVAL: Duration = Duration::from_secs(60);

#[tokio::main]
async fn main() -> Result<()> {
    // 配置日志系统
    let env_filter = EnvFilter::from_default_env()
        .add_directive(Level::INFO.into())
        .add_directive("nautilus_server=debug".parse().unwrap());

    fmt::fmt()
        .with_env_filter(env_filter)
        .with_target(true)
        .init();

    info!("日志系统已初始化");

    // 生成临时密钥对
    // let seed = 42u64;
    // let mut rand = StdRng::seed_from_u64(seed);
    // let eph_kp = Ed25519KeyPair::generate(&mut rand);
    let eph_kp = Ed25519KeyPair::generate(&mut rand::thread_rng());

    // 启动指标服务器
    let registry_service = start_basic_prometheus_server();
    let metrics = Metrics::new(&registry_service.default_registry());

    // 读取API密钥
    let api_key = std::env::var("API_KEY").expect("API_KEY必须设置");
    info!("API密钥长度: {}", api_key.len());

    // 检查是否启用密钥服务器功能
    let enable_key_server = env::var("ENABLE_KEY_SERVER").unwrap_or_else(|_| "false".to_string());
    let enable_key_server = enable_key_server.to_lowercase() == "true";

    let mut state = AppState {
        eph_kp,
        api_key,
        metrics: metrics.clone(),
        key_server_object_id_sig: None,
        latest_checkpoint_timestamp_receiver: None,
        reference_gas_price: None,
        // 添加缺少的字段
        sui_client: None,
        network: None,
        master_key: None,
        key_server_object_id: None,
    };

    info!("正在初始化密钥服务器功能");

    // 读取必要的环境变量
    let master_key = env::var("MASTER_KEY").expect("启用密钥服务器时MASTER_KEY必须设置");
    let object_id =
        env::var("KEY_SERVER_OBJECT_ID").expect("启用密钥服务器时KEY_SERVER_OBJECT_ID必须设置");
    let network = env::var("NETWORK")
        .map(|n| Network::from_str(&n))
        .unwrap_or(Network::Testnet);

    info!("密钥服务器网络: {:?}", network);

    // 初始化SUI客户端
    let sui_client = SuiClientBuilder::default()
        .build(&network.node_url())
        .await
        .expect("SUI客户端构建失败");

    // 初始化主密钥和服务器ID
    let master_key = IbeMasterKey::from_byte_array(
        &Base64::decode(&master_key)
            .expect("MASTER_KEY should be base64 encoded")
            .try_into()
            .expect("Invalid MASTER_KEY length"),
    )
    .expect("Invalid MASTER_KEY value");
    let key_server_object_id =
        ObjectID::from_hex_literal(&object_id).expect("无效的KEY_SERVER_OBJECT_ID");

    // 生成密钥持有证明
    let key_server_object_id_sig =
        crypto::ibe::create_proof_of_possession(&master_key, &key_server_object_id.into_bytes());

    // 启动定期更新任务
    // 更新最新检查点时间戳
    let latest_checkpoint_timestamp_receiver = spawn_periodic_updater(
        sui_client.clone(),
        CHECKPOINT_UPDATE_INTERVAL,
        get_latest_checkpoint_timestamp,
        "latest checkpoint timestamp",
        Some(observation_callback(
            &metrics.checkpoint_timestamp_delay,
            |ts| duration_since(ts) as f64,
        )),
        Some(observation_callback(
            &metrics.get_checkpoint_timestamp_duration,
            |d: Duration| d.as_millis() as f64,
        )),
        Some(status_callback(&metrics.get_checkpoint_timestamp_status)),
    )
    .await;

    // 更新参考gas价格
    let reference_gas_price = spawn_periodic_updater(
        sui_client.clone(),
        RGP_UPDATE_INTERVAL,
        get_reference_gas_price,
        "RGP",
        None::<fn(u64)>,
        None::<fn(Duration)>,
        Some(status_callback(&metrics.get_reference_gas_price_status)),
    )
    .await;

    // 更新AppState
    state.key_server_object_id_sig = Some(key_server_object_id_sig);
    state.latest_checkpoint_timestamp_receiver = Some(latest_checkpoint_timestamp_receiver);
    state.reference_gas_price = Some(reference_gas_price);
    // 设置其他字段
    state.sui_client = Some(sui_client);
    state.network = Some(network);
    state.master_key = Some(master_key);
    state.key_server_object_id = Some(key_server_object_id);

    info!("密钥服务器功能初始化完成");

    let state = Arc::new(state);

    // 定义CORS策略
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    // 配置路由
    let mut app = Router::new()
        .route("/", get(ping))
        .route("/get_attestation", get(get_attestation))
        .route("/process_data", post(process_data))
        .route("/health_check", get(health_check))
        .route("/v1/fetch_key", post(keys::handle_fetch_key))
        .route("/v1/service", get(keys::handle_get_service))
        .with_state(state)
        .layer(cors);

    // 使用nautilus_server的keys模块处理函数

    // 启动服务器
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("服务器已启动，监听地址: {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow::anyhow!("服务器错误: {}", e))
}

async fn ping() -> &'static str {
    "Pong!"
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

    // 如果由于全节点响应缓慢而错过了一个tick，我们不需要
    // 赶上来，而是延迟下一个tick。
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
