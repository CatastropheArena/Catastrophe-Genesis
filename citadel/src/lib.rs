// Copyright (c), Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::metrics::Metrics;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;
use fastcrypto::ed25519::Ed25519KeyPair;
use serde_json::json;
use fastcrypto::traits::KeyPair;
use std::sync::Arc;
use tokio::sync::watch::Receiver;
use sui_sdk::SuiClient;
use fastcrypto::ed25519::Ed25519PublicKey;
use sui_sdk::types::base_types::ObjectID;

pub mod app;
pub mod common;
pub mod metrics;
pub mod cache;        // 缓存系统，优化性能
pub mod errors;       // 错误类型定义
pub mod externals;    // 外部接口，如时间和gas价格
pub mod signed_message; // 签名消息处理
pub mod types;        // 数据类型定义
pub mod valid_ptb;    // 可编程交易块验证
pub mod keys;         // 密钥服务器模块
#[cfg(test)]
pub mod tests;    // 测试模块

/// 时间戳类型（64位无符号整数）
pub type Timestamp = u64;

/// App state, at minimum needs to maintain the ephemeral keypair.  
pub struct AppState {
    /// Ephemeral keypair on boot
    pub eph_kp: Ed25519KeyPair,
    /// API key when querying api.twitter.com
    pub api_key: String,
    /// Metrics
    pub metrics: Metrics,
    /// SUI客户端（可选，为密钥服务器功能）
    pub sui_client: Option<SuiClient>,
    /// 网络配置（可选，为密钥服务器功能）
    pub network: Option<types::Network>,
    /// IBE主密钥（可选，为密钥服务器功能）
    pub master_key: Option<types::IbeMasterKey>,
    /// 密钥服务器对象ID（可选，为密钥服务器功能）
    pub key_server_object_id: Option<ObjectID>,
    // /// 主密钥持有证明（可选，为密钥服务器功能）
    pub key_server_object_id_sig: Option<types::MasterKeyPOP>,
    // /// 最新检查点时间戳接收器（可选，为密钥服务器功能）
    pub latest_checkpoint_timestamp_receiver: Option<Receiver<Timestamp>>,
    // /// 参考gas价格接收器（可选，为密钥服务器功能）
    pub reference_gas_price: Option<Receiver<u64>>,
}

impl AppState {
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
    pub fn check_full_node_is_fresh(&self, allowed_staleness: std::time::Duration) -> Result<(), errors::InternalError> {
        if let Some(receiver) = &self.latest_checkpoint_timestamp_receiver {
            let staleness = externals::duration_since(*receiver.borrow());
            if staleness > allowed_staleness.as_millis() as i64 {
                tracing::warn!("Full node is stale. Latest checkpoint is {} ms old.", staleness);
                return Err(errors::InternalError::Failure);
            }
        } else {
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
    pub fn reference_gas_price(&self) -> u64 {
        if let Some(receiver) = &self.reference_gas_price {
            *receiver.borrow()
        } else {
            0
        }
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
