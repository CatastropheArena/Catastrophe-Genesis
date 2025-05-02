// Copyright (c), Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/**
 * 性能监控指标模块
 * 
 * 本模块实现了服务器的性能监控系统，提供以下功能：
 * 1. 请求计数器 - 记录不同类型的请求总数
 * 2. 错误计数器 - 按类型记录内部错误次数
 * 3. 时间延迟直方图 - 测量关键操作的执行时间
 * 4. 请求状态监控 - 跟踪外部API调用的成功/失败率
 * 
 * 所有指标均可通过Prometheus监控系统查询，便于服务质量监控。
 */

use axum::{extract::Extension, http::StatusCode, routing::get, Router};
use dashmap::DashMap;
use prometheus::{
    register_histogram_with_registry, register_int_counter_vec_with_registry,
    register_int_counter_with_registry, Histogram, IntCounter, IntCounterVec, Registry, TextEncoder,
};
use std::net::SocketAddr;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::time::Instant;
use tokio;
use uuid::Uuid;

/// Prometheus监控服务器的默认端口号
pub const METRICS_HOST_PORT: u16 = 9184;

/// Prometheus监控数据的API路径
pub const METRICS_ROUTE: &str = "/metrics";

/**
 * 处理指标请求的HTTP处理函数
 * 
 * 当客户端请求metrics端点时，此函数将从注册表服务中收集所有指标并返回
 * 
 * 参数:
 * @param registry_service - 通过Axum依赖注入提供的注册表服务实例
 * 
 * 返回:
 * - 成功时返回状态码200和序列化的Prometheus指标文本
 * - 失败时返回状态码500和错误信息
 */
async fn metrics(
    Extension(registry_service): Extension<RegistryService>,
) -> (StatusCode, String) {
    let metrics_families = registry_service.gather_all();
    match TextEncoder.encode_to_string(&metrics_families) {
        Ok(metrics) => (StatusCode::OK, metrics),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("unable to encode metrics: {error}"),
        ),
    }
}

/**
 * 启动基本的Prometheus监控服务器
 * 
 * 创建一个监听在指定端口的HTTP服务器，提供Prometheus指标数据
 * 服务器在后台线程中运行，不会阻塞调用线程
 * 
 * 参数:
 * @param custom_port - 可选的自定义端口号，如果不提供，则使用默认端口
 * 
 * 返回:
 * 初始化的注册表服务实例，可用于注册和管理指标
 */
pub fn start_basic_prometheus_server(custom_port: Option<u16>) -> RegistryService {
    // 创建监听所有网络接口的Socket地址
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), custom_port.unwrap_or(METRICS_HOST_PORT));
    // 创建新的Prometheus注册表
    let registry = Registry::new();
    // 初始化注册表服务
    let registry_service = RegistryService::new(registry);
    // 创建Axum路由，将metrics函数绑定到指定路径
    let app = Router::new()
        .route(METRICS_ROUTE, get(metrics))
        .layer(Extension(registry_service.clone()));

    // 在后台线程中启动HTTP服务器
    tokio::spawn(async move {
        // 尝试绑定指定端口
        match tokio::net::TcpListener::bind(&addr).await {
            Ok(listener) => {
                axum::serve(listener, app.into_make_service())
                    .await
                    .unwrap_or_else(|e| {
                        eprintln!("Failed to start metrics server: {}", e);
                    });
            }
            Err(e) => {
                // 如果端口被占用，则打印错误但不会导致程序崩溃
                // 这在测试环境中尤其重要，我们不希望因为端口被占用而导致测试失败
                eprintln!("Failed to bind metrics server to port {}: {}", METRICS_HOST_PORT, e);
            }
        }
    });
    registry_service
}

/**
 * Prometheus注册表管理服务
 * 
 * 此服务允许创建、管理和访问多个Prometheus注册表
 * 提供集中化的指标注册和收集功能
 * 设计为线程安全且可自由克隆，方便在不同组件间共享
 */
#[derive(Clone)]
pub struct RegistryService {
    /// 默认的Prometheus注册表，用于常规指标收集
    default_registry: Registry,
    
    /// 按UUID索引的额外注册表映射，支持动态创建的临时注册表
    /// 使用线程安全的DashMap实现并包装在Arc中实现跨线程共享
    registries_by_id: Arc<DashMap<Uuid, Registry>>,
}

impl RegistryService {
    /**
     * 创建新的注册表服务实例
     * 
     * 初始化具有指定默认注册表的服务
     * 默认注册表将被永久保留，不会被自动移除
     * 
     * 参数:
     * @param default_registry - 作为默认注册表使用的Registry实例
     * 
     * 返回:
     * 初始化的RegistryService实例
     */
    pub fn new(default_registry: Registry) -> Self {
        Self {
            default_registry,
            registries_by_id: Arc::new(DashMap::new()),
        }
    }

    /**
     * 获取默认注册表
     * 
     * 提供对服务默认注册表的访问
     * 返回的是克隆的注册表对象，可以安全地在不同线程中使用
     * 
     * 返回:
     * 默认Registry的克隆
     */
    pub fn default_registry(&self) -> Registry {
        self.default_registry.clone()
    }

    /**
     * 注册新的Prometheus注册表
     * 
     * 将提供的注册表添加到服务管理的注册表集合中
     * 生成唯一UUID作为此注册表的标识符
     * 
     * 参数:
     * @param registry - 要注册的Prometheus注册表实例
     * 
     * 返回:
     * 分配给此注册表的唯一UUID
     */
    pub fn register_registry(&self, registry: Registry) -> Uuid {
        let id = Uuid::new_v4();
        self.registries_by_id.insert(id, registry);
        id
    }

    /**
     * 通过ID获取特定注册表
     * 
     * 查找并返回与指定UUID关联的注册表
     * 
     * 参数:
     * @param id - 注册表的唯一标识符
     * 
     * 返回:
     * 如果存在，返回与ID关联的注册表的克隆，否则返回None
     */
    pub fn get_registry_by_id(&self, id: &Uuid) -> Option<Registry> {
        self.registries_by_id.get(id).map(|r| r.value().clone())
    }

    /**
     * 移除指定ID的注册表
     * 
     * 从服务管理的注册表集合中删除特定注册表
     * 
     * 参数:
     * @param id - 要移除的注册表的唯一标识符
     * 
     * 返回:
     * 如果注册表存在并被成功移除则返回true，否则返回false
     */
    pub fn remove_registry(&self, id: &Uuid) -> bool {
        self.registries_by_id.remove(id).is_some()
    }

    /**
     * 创建并注册新的注册表
     * 
     * 创建一个新的Prometheus注册表实例，并将其注册到服务中
     * 
     * 返回:
     * 包含新创建的注册表和其唯一标识符的元组(Registry, Uuid)
     */
    pub fn create_registry(&self, prefix: Option<&str>) -> (Registry, Uuid) {
        let registry_result = Registry::new_custom(prefix.map(|s| s.to_string()), None);
        let registry = registry_result.expect("Create registry failed");
        let id = self.register_registry(registry.clone());
        (registry, id)
    }

    /**
     * 获取所有注册表
     * 
     * 收集服务管理的所有注册表，包括默认注册表和按ID索引的注册表
     * 
     * 返回:
     * 包含所有注册表克隆的向量
     */
    pub fn get_all(&self) -> Vec<Registry> {
        // 收集ID索引的注册表
        let mut registries: Vec<Registry> = self
            .registries_by_id
            .iter()
            .map(|r| r.value().clone())
            .collect();
        // 添加默认注册表
        registries.push(self.default_registry.clone());

        registries
    }

    /**
     * 收集所有指标数据
     * 
     * 从所有注册表中收集指标并合并为单一结果集
     * 用于向Prometheus客户端提供完整的指标数据
     * 
     * 返回:
     * 合并所有注册表数据的指标族集合
     */
    pub fn gather_all(&self) -> Vec<prometheus::proto::MetricFamily> {
        self.get_all().iter().flat_map(|r| r.gather()).collect()
    }

    /**
     * 计算注册表总数
     * 
     * 返回当前服务管理的注册表总数，包括默认注册表
     * 
     * 返回:
     * 注册表总数
     */
    pub fn count_registries(&self) -> usize {
        // 默认注册表加上动态注册的注册表数量
        1 + self.registries_by_id.len()
    }
}

/**
 * 指标结构体
 * 
 * 包含服务器运行过程中收集的所有度量指标
 * 这些指标用于监控服务器性能和健康状态
 */
#[derive(Clone, Debug)]
pub struct Metrics {
    /// 接收的请求总数
    pub requests: IntCounterVec,

    /// 按类型划分的内部错误总数
    pub errors: IntCounterVec,

    /// 最新检查点时间戳的延迟
    pub checkpoint_timestamp_delay: Histogram,

    /// 获取最新检查点时间戳的持续时间
    pub get_checkpoint_timestamp_duration: Histogram,

    /// 获取最新检查点时间戳请求的状态
    pub get_checkpoint_timestamp_status: IntCounterVec,

    /// 获取参考gas价格请求的状态
    pub get_reference_gas_price_status: IntCounterVec,

    /// check_policy操作的持续时间
    pub check_policy_duration: Histogram,

    /// fetch_pkg_ids操作的持续时间
    pub fetch_pkg_ids_duration: Histogram,

    /// 按ID数量划分的请求总数
    pub requests_per_number_of_ids: Histogram,
}

/// 定义指标组的枚举类型，替代字符串标识符
/// 便于编译期检查和自动补全
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricGroup {
    /// 请求相关指标组
    Requests,
    /// 错误相关指标组
    Errors,
    /// 检查点时间戳延迟指标
    CheckpointTimestampDelay,
    /// 获取检查点时间戳持续时间指标
    GetCheckpointTimestampDuration,
    /// 获取检查点时间戳状态指标
    GetCheckpointTimestampStatus,
    /// 获取参考gas价格状态指标
    GetReferenceGasPriceStatus,
    /// 检查策略持续时间指标
    CheckPolicyDuration,
    /// 获取包ID持续时间指标
    FetchPkgIdsDuration,
    /// 按ID数量统计的请求指标
    RequestsPerNumberOfIds,
}

impl MetricGroup {
    /// 将枚举转换为字符串标识符
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requests => "requests",
            Self::Errors => "errors",
            Self::CheckpointTimestampDelay => "checkpoint_timestamp_delay",
            Self::GetCheckpointTimestampDuration => "get_checkpoint_timestamp_duration",
            Self::GetCheckpointTimestampStatus => "get_checkpoint_timestamp_status",
            Self::GetReferenceGasPriceStatus => "get_reference_gas_price_status",
            Self::CheckPolicyDuration => "check_policy_duration", 
            Self::FetchPkgIdsDuration => "fetch_pkg_ids_duration",
            Self::RequestsPerNumberOfIds => "requests_per_number_of_ids",
        }
    }
}

/// 指标构建器结构体，提供流畅的API来创建和注册指标
pub struct MetricsBuilder {
    /// 默认注册表，用于未指定特定注册表的指标
    default_registry: Option<Registry>,
    
    /// 映射指标名称到特定注册表，使用枚举类型作为键
    registry_map: std::collections::HashMap<MetricGroup, Registry>,
}

impl MetricsBuilder {
    /// 创建新的指标构建器
    pub fn new() -> Self {
        Self {
            default_registry: None,
            registry_map: std::collections::HashMap::new(),
        }
    }
    
    /// 设置默认注册表
    pub fn with_default_registry(mut self, registry: Registry) -> Self {
        self.default_registry = Some(registry);
        self
    }
    
    /// 为特定指标名称指定注册表
    pub fn with_registry_for(mut self, metric_group: MetricGroup, registry: Registry) -> Self {
        self.registry_map.insert(metric_group, registry);
        self
    }
    
    /// 从RegistryService中创建构建器
    pub fn from_registry_service(registry_service: &RegistryService) -> Self {
        Self {
            default_registry: Some(registry_service.default_registry()),
            registry_map: std::collections::HashMap::new(),
        }
    }

    /// 构建Metrics实例
    pub fn build(self) -> Result<Metrics, &'static str> {
        let default_registry = self.default_registry.ok_or("Default registry is required")?;

        // 对每个指标组选择适当的注册表
        let requests_registry = self
            .registry_map
            .get(&MetricGroup::Requests)
            .unwrap_or(&default_registry);

        let errors_registry = self
            .registry_map
            .get(&MetricGroup::Errors) 
            .unwrap_or(&default_registry);

        let checkpoint_timestamp_delay_registry = self
            .registry_map
            .get(&MetricGroup::CheckpointTimestampDelay)
            .unwrap_or(&default_registry);

        let get_checkpoint_timestamp_duration_registry = self
            .registry_map
            .get(&MetricGroup::GetCheckpointTimestampDuration)
            .unwrap_or(&default_registry);

        let get_checkpoint_timestamp_status_registry = self
            .registry_map
            .get(&MetricGroup::GetCheckpointTimestampStatus)
            .unwrap_or(&default_registry);

        let get_reference_gas_price_status_registry = self
            .registry_map
            .get(&MetricGroup::GetReferenceGasPriceStatus)
            .unwrap_or(&default_registry);

        let check_policy_duration_registry = self
            .registry_map
            .get(&MetricGroup::CheckPolicyDuration)
            .unwrap_or(&default_registry);

        let fetch_pkg_ids_duration_registry = self
            .registry_map
            .get(&MetricGroup::FetchPkgIdsDuration)
            .unwrap_or(&default_registry);

        let requests_per_number_of_ids_registry = self
            .registry_map
            .get(&MetricGroup::RequestsPerNumberOfIds)
            .unwrap_or(&default_registry);

        // 创建各种指标
        let requests = register_int_counter_vec_with_registry!(
            "citadel_requests_total",
            "Total number of requests received",
            &["type"],
            requests_registry
        )
        .map_err(|_| "Failed to register requests counter")?;

        let errors = register_int_counter_vec_with_registry!(
            "internal_errors",
            "按类型划分的内部错误总数",
            &["internal_error_type"],
            errors_registry
        )
        .unwrap();

        let checkpoint_timestamp_delay = register_histogram_with_registry!(
            "checkpoint_timestamp_delay",
            "最新检查点时间戳的延迟",
            default_external_call_duration_buckets(),
            checkpoint_timestamp_delay_registry
        )
        .unwrap();

        let get_checkpoint_timestamp_duration = register_histogram_with_registry!(
            "checkpoint_timestamp_duration",
            "获取最新检查点时间戳的持续时间",
            default_external_call_duration_buckets(),
            get_checkpoint_timestamp_duration_registry
        )
        .unwrap();

        let get_checkpoint_timestamp_status = register_int_counter_vec_with_registry!(
            "checkpoint_timestamp_status",
            "获取最新时间戳请求的状态",
            &["status"],
            get_checkpoint_timestamp_status_registry
        )
        .unwrap();

        let fetch_pkg_ids_duration = register_histogram_with_registry!(
            "fetch_pkg_ids_duration",
            "fetch_pkg_ids操作的持续时间",
            default_fast_call_duration_buckets(),
            fetch_pkg_ids_duration_registry
        )
        .unwrap();

        let check_policy_duration = register_histogram_with_registry!(
            "check_policy_duration",
            "check_policy操作的持续时间",
            default_fast_call_duration_buckets(),
            check_policy_duration_registry
        )
        .unwrap();

        let get_reference_gas_price_status = register_int_counter_vec_with_registry!(
            "get_reference_gas_price_status",
            "获取参考gas价格请求的状态",
            &["status"],
            get_reference_gas_price_status_registry
        )
        .unwrap();

        let requests_per_number_of_ids = register_histogram_with_registry!(
            "requests_per_number_of_ids",
            "按ID数量划分的请求总数",
            buckets(0.0, 5.0, 1.0),
            requests_per_number_of_ids_registry
        )
        .unwrap();

        Ok(Metrics {
            requests,
            errors,
            checkpoint_timestamp_delay,
            get_checkpoint_timestamp_duration,
            get_checkpoint_timestamp_status,
            get_reference_gas_price_status,
            check_policy_duration,
            fetch_pkg_ids_duration,
            requests_per_number_of_ids,
        })
    }
}

impl Metrics {
    /**
     * 记录错误事件
     * 
     * 增加指定类型错误的计数器
     * 
     * 参数:
     * @param error_type - 错误类型标识符
     */
    pub fn observe_error(&self, error_type: &str) {
        self.errors.with_label_values(&[error_type]).inc();
    }

    /**
     * 记录请求
     * 
     * 参数:
     * @param request_type - 请求类型标识符
     */
    pub fn observe_request(&self, request_type: &str) {
        self.requests.with_label_values(&[request_type]).inc();
    }

}

/**
 * 测量闭包执行时间
 * 
 * 如果指定了直方图，则测量闭包执行时间并记录
 * 否则仅执行闭包
 * 
 * 参数:
 * @param metrics - 可选的直方图指标
 * @param closure - 要执行和测量的闭包
 * 
 * 返回:
 * 闭包的返回值
 */
pub fn call_with_duration<T>(metrics: Option<&Histogram>, closure: impl FnOnce() -> T) -> T {
    if let Some(metrics) = metrics {
        let start = Instant::now();
        let result = closure();
        metrics.observe(start.elapsed().as_millis() as f64);
        result
    } else {
        closure()
    }
}

/**
 * 创建观察回调函数
 * 
 * 返回一个闭包，该闭包将输入通过转换函数处理后记录到直方图
 * 
 * 参数:
 * @param histogram - 要更新的直方图
 * @param f - 将输入值转换为f64的函数
 * 
 * 返回:
 * 接受T类型输入并更新直方图的闭包
 */
pub fn observation_callback<T>(histogram: &Histogram, f: impl Fn(T) -> f64) -> impl Fn(T) {
    let histogram = histogram.clone();
    move |t| {
        histogram.observe(f(t));
    }
}

/**
 * 创建状态回调函数
 * 
 * 返回一个闭包，该闭包根据布尔状态更新计数器向量
 * 
 * 参数:
 * @param metrics - 要更新的计数器向量
 * 
 * 返回:
 * 接受布尔状态并更新相应计数器的闭包
 */
pub fn status_callback(metrics: &IntCounterVec) -> impl Fn(bool) {
    let metrics = metrics.clone();
    move |status: bool| {
        let value = match status {
            true => "success",
            false => "failure",
        };
        metrics.with_label_values(&[value]).inc();
    }
}

/**
 * 创建等距分布的桶值
 * 
 * 生成从起始值到结束值按步长均匀分布的桶值数组
 * 用于创建直方图的桶配置
 * 
 * 参数:
 * @param start - 起始值
 * @param end - 结束值
 * @param step - 步长
 * 
 * 返回:
 * 桶值数组
 */
fn buckets(start: f64, end: f64, step: f64) -> Vec<f64> {
    let mut buckets = vec![];
    let mut current = start;
    while current < end {
        buckets.push(current);
        current += step;
    }
    buckets.push(end);
    buckets
}

/**
 * 默认外部调用持续时间桶
 * 
 * 为外部API调用定义的默认桶配置
 * 范围从50ms到2000ms，步长为50ms
 * 
 * 返回:
 * 适用于外部调用的桶值数组
 */
fn default_external_call_duration_buckets() -> Vec<f64> {
    buckets(50.0, 2000.0, 50.0)
}

/**
 * 默认快速调用持续时间桶
 * 
 * 为内部快速操作定义的默认桶配置
 * 范围从10ms到100ms，步长为10ms
 * 
 * 返回:
 * 适用于快速调用的桶值数组
 */
fn default_fast_call_duration_buckets() -> Vec<f64> {
    buckets(10.0, 100.0, 10.0)
}
