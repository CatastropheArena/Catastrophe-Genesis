// Copyright (c), Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
/**
 * 密钥服务器实现
 *
 * 此模块实现了Seal密钥服务器的核心功能，包括：
 * 1. HTTP API端点，用于处理密钥请求
 * 2. 用户请求验证机制
 * 3. 使用IBE为授权用户提供解密密钥
 * 4. 安全策略验证
 */
use axum::{extract::State, http::HeaderMap, Json};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crypto::elgamal::encrypt;
use crypto::ibe;
use fastcrypto::ed25519::{Ed25519PublicKey, Ed25519Signature};
use fastcrypto::encoding::{Base64, Encoding};
use fastcrypto::traits::VerifyingKey;
use rand::thread_rng;
use std::sync::Arc;
use std::time::Duration;

use sui_sdk::rpc_types::SuiTransactionBlockEffectsAPI;
use sui_sdk::types::base_types::{ObjectID, SuiAddress};
use sui_sdk::types::signature::GenericSignature;
use sui_sdk::types::transaction::{ProgrammableTransaction, TransactionKind};
use sui_sdk::verify_personal_message_signature::verify_personal_message_signature;
use tap::TapFallible;
use tracing::{debug, info, warn};

use crate::errors::InternalError;
use crate::externals::{current_epoch_time, fetch_first_and_last_pkg_id};
use crate::metrics::call_with_duration;
use crate::metrics::Metrics;
use crate::signed_message::{signed_message, signed_request};
use crate::types::{ElGamalPublicKey, ElgamalEncryption, ElgamalVerificationKey, MasterKeyPOP, GAS_BUDGET};
use crate::valid_ptb::ValidPtb;
use crate::AppState;

/// 会话密钥的最大生存时间（分钟）
pub const SESSION_KEY_TTL_MAX: u16 = 10;

/// 允许的全节点数据过时时间
/// 设置此持续时间时，注意Sui上的时间戳可能比当前时间稍晚，但不应超过一秒。
pub const ALLOWED_STALENESS: Duration = Duration::from_secs(120);

/**
 * 会话证书，由用户签名
 * 用于验证用户身份和请求合法性
 *
 * 包含以下信息：
 * - 用户地址
 * - 会话验证密钥
 * - 创建时间
 * - 生存时间
 * - 用户签名
 */
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Certificate {
    pub user: SuiAddress,             // 用户的Sui地址
    pub session_vk: Ed25519PublicKey, // 会话验证密钥
    pub creation_time: u64,           // 创建时间（Unix时间戳）
    pub ttl_min: u16,                 // 生存时间（分钟）
    pub signature: GenericSignature,  // 用户签名
}

/**
 * 获取密钥请求结构
 *
 * 客户端发送此请求以获取解密密钥
 * 包含签名的请求数据和验证信息
 */
#[derive(Serialize, Deserialize)]
pub struct FetchKeyRequest {
    // 以下字段必须签名，以防止他人代表用户发送请求并能够获取密钥
    ptb: String, // 必须遵循特定结构，参见ValidPtb
    // 我们不想仅依靠HTTPS来限制对此用户的响应，因为在多个服务的情况下，
    // 一个服务可以对另一个服务进行重放攻击以获取其他服务的密钥。
    enc_key: ElGamalPublicKey,                    // ElGamal加密公钥
    enc_verification_key: ElgamalVerificationKey, // ElGamal验证密钥
    request_signature: Ed25519Signature,          // 请求签名

    certificate: Certificate, // 用户会话证书
}

/// 密钥ID类型（字节数组）
pub type KeyId = Vec<u8>;

/**
 * 解密密钥结构
 *
 * 包含密钥ID和加密后的密钥
 * 返回给客户端用于解密其数据
 */
#[derive(Serialize, Deserialize)]
pub struct DecryptionKey {
    id: KeyId,                            // 密钥标识符
    pub encrypted_key: ElgamalEncryption, // 加密的密钥
}

/**
 * 获取密钥响应结构
 *
 * 服务器返回的加密密钥列表
 */
#[derive(Serialize, Deserialize)]
pub struct FetchKeyResponse {
    pub decryption_keys: Vec<DecryptionKey>, // 解密密钥列表
}

/**
 * 获取服务信息响应
 *
 * 包含服务ID和主密钥持有证明
 */
#[derive(Serialize, Deserialize)]
pub struct GetServiceResponse {
    service_id: ObjectID,
    pop: MasterKeyPOP,
}

/**
 * 检查请求签名的有效性
 *
 * 验证用户证书和会话签名，确保请求的合法性
 *
 * 参数:
 * @param pkg_id - 包ID
 * @param ptb - 可编程交易块
 * @param enc_key - ElGamal加密公钥
 * @param enc_verification_key - ElGamal验证密钥
 * @param session_sig - 会话签名
 * @param cert - 用户证书
 * @param req_id - 请求ID（用于日志）
 *
 * 返回:
 * 成功时返回Ok(())，失败时返回错误
 */
#[allow(clippy::too_many_arguments)]
async fn check_signature(
    app_state: &AppState,
    pkg_id: &ObjectID,
    ptb: &ProgrammableTransaction,
    enc_key: &ElGamalPublicKey,
    enc_verification_key: &ElgamalVerificationKey,
    session_sig: &Ed25519Signature,
    cert: &Certificate,
    req_id: Option<&str>,
) -> Result<(), InternalError> {
    // 检查证书有效性
    if cert.ttl_min > SESSION_KEY_TTL_MAX
        || cert.creation_time > current_epoch_time()
        || current_epoch_time() < 60_000 * (cert.ttl_min as u64) // 检查溢出
        || current_epoch_time() - 60_000 * (cert.ttl_min as u64) > cert.creation_time
    {
        debug!(
            "Certificate has invalid expiration time (req_id: {:?})",
            req_id
        );
        return Err(InternalError::InvalidCertificate);
    }

    let msg = signed_message(pkg_id, &cert.session_vk, cert.creation_time, cert.ttl_min);
    debug!(
        "Checking signature on message: {:?} (req_id: {:?})",
        msg, req_id
    );
    // 验证用户签名
    verify_personal_message_signature(
        cert.signature.clone(),
        msg.as_bytes(),
        cert.user,
        Some(app_state.sui_client.clone()),
    )
    .await
    .tap_err(|e| {
        debug!(
            "Signature verification failed: {:?} (req_id: {:?})",
            e, req_id
        );
    })
    .map_err(|_| InternalError::InvalidSignature)?;

    // 验证会话签名（请求签名）
    let signed_msg = signed_request(ptb, enc_key, enc_verification_key);
    cert.session_vk
        .verify(&signed_msg, session_sig)
        .map_err(|_| {
            debug!(
                "Session signature verification failed (req_id: {:?})",
                req_id
            );
            InternalError::InvalidSessionSignature
        })
}

/**
 * 检查策略合规性
 *
 * 通过模拟执行交易确认用户是否有权限获取密钥
 *
 * 参数:
 * @param sender - 发送者地址
 * @param vptb - 验证过的可编程交易块
 * @param gas_price - 当前gas价格
 * @param req_id - 请求ID（用于日志）
 *
 * 返回:
 * 成功时返回Ok(())，失败时返回错误
 */
async fn check_policy(
    app_state: &AppState,
    sender: SuiAddress,
    vptb: &ValidPtb,
    gas_price: u64,
    req_id: Option<&str>,
) -> Result<(), InternalError> {
    debug!(
        "Checking policy for ptb: {:?} (req_id: {:?})",
        vptb.ptb(),
        req_id
    );
    // 评估`seal_approve*`函数
    let tx_data = app_state
        .sui_client
        .transaction_builder()
        .tx_data_for_dry_run(
            sender,
            TransactionKind::ProgrammableTransaction(vptb.ptb().clone()),
            GAS_BUDGET,
            gas_price,
            None,
            None,
        )
        .await;
    let dry_run_res = app_state
        .sui_client
        .read_api()
        .dry_run_transaction_block(tx_data)
        .await
        .map_err(|e| {
            warn!("Dry run execution failed ({:?}) (req_id: {:?})", e, req_id);
            InternalError::Failure
        })?;
    debug!("Dry run response: {:?} (req_id: {:?})", dry_run_res, req_id);
    if dry_run_res.effects.status().is_err() {
        debug!("Dry run execution asserted (req_id: {:?})", req_id);
        // TODO: 我们是否应该根据状态返回不同的错误，例如InsufficientGas？
        return Err(InternalError::NoAccess);
    }

    // 一切正常！
    Ok(())
}

/**
 * 检查请求的有效性
 *
 * 全面验证请求，包括：
 * 1. 验证PTB格式
 * 2. 验证签名
 * 3. 检查策略合规性
 *
 * 参数:
 * @param ptb_str - PTB的Base64编码字符串
 * @param enc_key - ElGamal加密公钥
 * @param enc_verification_key - ElGamal验证密钥
 * @param request_signature - 请求签名
 * @param certificate - 用户证书
 * @param gas_price - 当前gas价格
 * @param metrics - 性能指标收集器
 * @param req_id - 请求ID（用于日志）
 *
 * 返回:
 * 成功时返回密钥ID列表，失败时返回错误
 */
#[allow(clippy::too_many_arguments)]
pub async fn check_request(
    app_state: &AppState,
    ptb_str: &str,
    enc_key: &ElGamalPublicKey,
    enc_verification_key: &ElgamalVerificationKey,
    request_signature: &Ed25519Signature,
    certificate: &Certificate,
    gas_price: u64,
    metrics: Option<&Metrics>,
    req_id: Option<&str>,
) -> Result<Vec<KeyId>, InternalError> {
    debug!(
        "Checking request for ptb_str: {:?}, cert {:?} (req_id: {:?})",
        ptb_str, certificate, req_id
    );
    let ptb_b64 = Base64::decode(ptb_str).map_err(|_| InternalError::InvalidPTB)?;
    let ptb: ProgrammableTransaction =
        bcs::from_bytes(&ptb_b64).map_err(|_| InternalError::InvalidPTB)?;
    let valid_ptb = ValidPtb::try_from(ptb.clone())?;

    // 向指标报告请求中的ID数量
    if let Some(m) = metrics {
        m.requests_per_number_of_ids
            .observe(valid_ptb.inner_ids().len() as f64);
    }

    // 处理包升级：只调用最新版本，但使用第一个版本作为命名空间
    let (first_pkg_id, last_pkg_id) =
        call_with_duration(metrics.map(|m| &m.fetch_pkg_ids_duration), || async {
            fetch_first_and_last_pkg_id(&valid_ptb.pkg_id(), &app_state.network).await
        })
        .await?;

    if valid_ptb.pkg_id() != last_pkg_id {
        debug!(
            "Last package version is {:?} while ptb uses {:?} (req_id: {:?})",
            last_pkg_id,
            valid_ptb.pkg_id(),
            req_id
        );
        return Err(InternalError::OldPackageVersion);
    }

    // 检查所有条件
    check_signature(
        app_state,
        &first_pkg_id,
        &ptb,
        enc_key,
        enc_verification_key,
        request_signature,
        certificate,
        req_id,
    )
    .await?;

    call_with_duration(metrics.map(|m| &m.check_policy_duration), || async {
        check_policy(app_state, certificate.user, &valid_ptb, gas_price, req_id).await
    })
    .await?;

    info!(
        "Valid request: {}",
        json!({ "user": certificate.user, "package_id": valid_ptb.pkg_id(), "req_id": req_id })
    );

    // 返回以第一个包ID为前缀的完整ID
    Ok(valid_ptb.full_ids(&first_pkg_id))
}

/**
 * 创建响应
 *
 * 为每个密钥ID生成加密的解密密钥
 *
 * 参数:
 * @param ids - 密钥ID列表
 * @param enc_key - 用于加密的ElGamal公钥
 *
 * 返回:
 * 包含加密密钥的响应
 */
pub fn create_response(
    app_state: &AppState,
    ids: &[KeyId],
    enc_key: &ElGamalPublicKey,
) -> FetchKeyResponse {
    debug!("Checking response for ids: {:?}", ids);
    let decryption_keys = ids
        .iter()
        .map(|id| {
            // 请求的密钥
            let key = ibe::extract(&app_state.master_key, id);
            // 使用用户的公钥对密钥进行ElGamal加密
            let encrypted_key = encrypt(&mut thread_rng(), &key, enc_key);
            DecryptionKey {
                id: id.to_owned(),
                encrypted_key,
            }
        })
        .collect();
    FetchKeyResponse { decryption_keys }
}

/**
 * 处理获取密钥请求
 *
 * 处理客户端的密钥请求，验证其有效性并返回加密的密钥
 *
 * 参数:
 * @param app_state - 应用状态
 * @param headers - HTTP请求头
 * @param payload - 请求负载
 *
 * 返回:
 * 成功时返回密钥响应，失败时返回错误
 */
pub async fn handle_fetch_key(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<FetchKeyRequest>,
) -> Result<Json<FetchKeyResponse>, InternalError> {
    let req_id = headers
        .get("Request-Id")
        .map(|v| v.to_str().unwrap_or_default());
    let version = headers.get("Client-Sdk-Version");
    let sdk_type = headers.get("Client-Sdk-Type");
    let target_api_version = headers.get("Client-Target-Api-Version");
    info!(
        "Request id: {:?}, SDK version: {:?}, SDK type: {:?}, Target API version: {:?}",
        req_id, version, sdk_type, target_api_version
    );

    app_state.metrics.observe_request("fetch_key");
    app_state.check_full_node_is_fresh(ALLOWED_STALENESS)?;

    check_request(
        &app_state,
        &payload.ptb,
        &payload.enc_key,
        &payload.enc_verification_key,
        &payload.request_signature,
        &payload.certificate,
        app_state.reference_gas_price(),
        Some(&app_state.metrics),
        req_id,
    )
    .await
    .map(|full_id| Json(create_response(&app_state, &full_id, &payload.enc_key)))
    .tap_err(|e| app_state.metrics.observe_error(e.as_str()))
}
/**
 * 处理获取服务信息请求
 *
 * 返回服务器ID和密钥持有证明，用于客户端验证服务器身份
 *
 * 参数:
 * @param app_state - 应用状态
 *
 * 返回:
 * 服务信息响应
 */
pub async fn handle_get_service(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<GetServiceResponse>, InternalError> {
    app_state.metrics.observe_request("get_service");
    Ok(Json(GetServiceResponse {
        service_id: app_state.key_server_object_id.clone(),
        pop: app_state.key_server_object_id_sig.clone(),
    }))
}
