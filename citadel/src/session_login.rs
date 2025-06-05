// Copyright (c), Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use axum::extract::Query;
use axum::response::IntoResponse;
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
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crypto::elgamal::{encrypt};
use crypto::ibe;
use fastcrypto::ed25519::{Ed25519PublicKey, Ed25519Signature};
use fastcrypto::encoding::{Base64, Encoding};
use fastcrypto::traits::Signer;
use rand::thread_rng;
use std::sync::Arc;
use std::time::Duration;

use sui_sdk::rpc_types::SuiTransactionBlockEffectsAPI;
use sui_sdk::types::base_types::{ObjectID, SuiAddress};
use sui_sdk::types::signature::GenericSignature;
use sui_sdk::types::transaction::{Command, Argument, CallArg, ProgrammableTransaction, TransactionKind};
use sui_sdk::verify_personal_message_signature::verify_personal_message_signature;
use tap::TapFallible;
use tracing::{debug, info, warn,error};

use crate::errors::InternalError;
use crate::externals::{current_epoch_time, fetch_first_and_last_pkg_id};
use crate::keys::{check_request, Certificate};
use crate::metrics::call_with_duration;
use crate::metrics::Metrics;
use crate::types::{ElGamalPublicKey, ElgamalVerificationKey, ElgamalEncryption, MasterKeyPOP, GAS_BUDGET};
use crate::AppState;
use axum::{
    extract::{Request},
    http::{ StatusCode},
    middleware::Next,
    response::Response,
};
use crate::valid_ptb::ValidPtb;
use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};
use crate::avatars::{make_avatar, make_male_avatar, make_female_avatar};
use crate::sdk::create_profile_for_passport;
use crate::sdk::Profile;  // 从sdk模块直接导入Profile类型
use hex;
use tower_sessions::{Session, Expiry};
use uuid::Uuid;
use axum::extract::Extension;

/**
 * JWT令牌Claims结构
 *
 * 包含用户会话信息的JWT令牌Payload部分
 */
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenClaims {
    // JWT标准字段
    pub iss: String,     // 发行者（服务器标识）
    pub sub: String,     // 主题（用户地址）
    pub exp: u64,        // 过期时间（Unix时间戳，秒）
    pub iat: u64,        // 发行时间（Unix时间戳，秒）
    // 自定义字段
    pub profile: Option<Profile>, // 用户档案
    pub user_address: SuiAddress, // 用户地址
    pub session_vk: String,       // 会话验证密钥（Base64编码）
    pub creation_time: u64,       // 证书创建时间
    pub ttl_min: u16,             // 生存时间（分钟）
}
/**
 * 登录用户信息
 *
 * 包含当前登录用户的基本信息
 */
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_address: SuiAddress,  // 用户地址
    pub session_vk: String,        // 会话验证密钥（Base64编码）
    pub exp: u64,                  // 过期时间（Unix时间戳，秒）
    pub profile: Option<Profile>, // 用户档案
}

/**
 * Session 用户信息
 * 
 * 存储在 session 中的用户数据
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUser {
    pub user_address: SuiAddress,  // 用户地址
    pub session_vk: String,        // 会话验证密钥
    pub exp: u64,                  // 过期时间
    pub profile: Option<Profile>, // 用户档案
}

// 用于 session 的常量键名
pub const SESSION_USER_KEY: &str = "user";

/**
 * 从JWT令牌解析出用户信息
 *
 * 解析并验证JWT令牌，返回用户信息
 */
pub fn decode_token(
    app_state: &Arc<AppState>,
    token: &str,
) -> Result<TokenData<TokenClaims>, InternalError> {
    // 使用与生成令牌相同的密钥派生方法
    let msg = b"jwt_secret";
    let signature: Ed25519Signature = app_state.eph_kp.sign(msg);
    let decoding_key = DecodingKey::from_secret(signature.as_ref());

    // 设置验证参数
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.set_issuer(&["catastrophe"]);

    // 解码并验证令牌
    decode::<TokenClaims>(token, &decoding_key, &validation)
        .map_err(|e| {
            debug!("Token validation failed: {:?}", e);
            InternalError::InvalidToken
        })
}

/**
 * 从HTTP请求头中提取JWT令牌
 *
 * 从Authorization头中提取Bearer令牌
 */
pub fn extract_token_from_headers(headers: &HeaderMap) -> Result<String, InternalError> {
    let auth_header = headers
        .get("Authorization")
        .ok_or(InternalError::MissingAuthToken)?;
    
    let auth_str = auth_header.to_str().map_err(|_| InternalError::InvalidAuthHeader)?;
    
    if !auth_str.starts_with("Bearer ") {
        return Err(InternalError::InvalidAuthHeader);
    }
    
    Ok(auth_str[7..].to_string())
}

/**
 * JWT认证中间件
 *
 * 验证请求头中的JWT令牌，并将用户信息传递给下一个处理器
 */
pub async fn auth_middleware(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, InternalError> {
    // 提取令牌
    let token = extract_token_from_headers(&headers)?;
    
    // 解码并验证令牌
    let token_data = decode_token(&app_state, &token)?;
    
    // 检查令牌是否过期
    let current_time_secs = current_epoch_time() / 1000;
    if token_data.claims.exp < current_time_secs {
        return Err(InternalError::ExpiredToken);
    }
    
    // 创建已认证用户信息
    let user = AuthenticatedUser {
        user_address: token_data.claims.user_address,
        session_vk: token_data.claims.session_vk.clone(),
        exp: token_data.claims.exp,
        profile: token_data.claims.profile.clone(),
    };
    
    // 将用户信息添加到请求扩展中
    request.extensions_mut().insert(user);
    
    // 调用下一个处理器
    Ok(next.run(request).await)
}

/**
 * 从请求扩展中获取当前登录用户
 *
 * 在被auth_middleware保护的路由中使用
 */
pub fn get_authenticated_user(request: &Request) -> Option<AuthenticatedUser> {
    request.extensions().get::<AuthenticatedUser>().cloned()
}

/**
 * 验证JWT令牌
 * 
 * 提供给其他模块调用的辅助函数，用于直接验证JWT令牌
 */
pub fn verify_auth_token(app_state: &Arc<AppState>, token: &str) -> Result<AuthenticatedUser, InternalError> {
    let token_data = decode_token(app_state, token)?;
    
    // 检查令牌是否过期
    let current_time_secs = current_epoch_time() / 1000;
    if token_data.claims.exp < current_time_secs {
        return Err(InternalError::ExpiredToken);
    }
    
    Ok(AuthenticatedUser {
        user_address: token_data.claims.user_address,
        session_vk: token_data.claims.session_vk.clone(),
        exp: token_data.claims.exp,
        profile: token_data.claims.profile.clone(),
    })
} 

/// 允许的全节点数据过时时间
/// 设置此持续时间时，注意Sui上的时间戳可能比当前时间稍晚，但不应超过一秒。
const ALLOWED_STALENESS: Duration = Duration::from_secs(120);

/**
 * 获取密钥请求结构
 *
 * 客户端发送此请求以获取解密密钥
 * 包含签名的请求数据和验证信息
 */
#[derive(Serialize, Deserialize)]
pub struct SessionTokenRequest {
    // 以下字段必须签名，以防止他人代表用户发送请求并能够获取密钥
    ptb: String, // 必须遵循特定结构，参见ValidPtb
    // 我们不想仅依靠HTTPS来限制对此用户的响应，因为在多个服务的情况下，
    // 一个服务可以对另一个服务进行重放攻击以获取其他服务的密钥。
    enc_key: ElGamalPublicKey,                    // ElGamal加密公钥
    enc_verification_key: ElgamalVerificationKey, // ElGamal验证密钥
    request_signature: Ed25519Signature,          // 请求签名
    certificate: Certificate,                     // 用户会话证书
}


/**
 * 会话令牌响应结构
 *
 * 服务器返回的授权令牌，包含加密的证书信息
 */
#[derive(Serialize, Deserialize)]
pub struct SessionTokenResponse {
    pub auth_token: String, // JWT格式的授权令牌
    pub expires_at: u64,    // 令牌过期时间（Unix时间戳，毫秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<Profile>,
}

/**
 * 创建会话令牌响应
 *
 * 使用服务器的密钥生成JWT格式的授权令牌
 *
 * 参数:
 * @param app_state - 应用状态
 * @param certificate - 用户证书
 * @param profile - 用户档案
 *
 * 返回:
 * 包含JWT令牌的响应
 */
fn create_session_token_response(
    app_state: &AppState,
    certificate: &Certificate,
    profile: Option<Profile>,
) -> SessionTokenResponse {
    debug!("Creating session token for user: {:?}", certificate.user);

    // 计算过期时间（当前时间 + 证书的TTL）
    let current_time = current_epoch_time(); // 毫秒时间戳
    let current_time_secs = current_time / 1000; // 转换为秒
    let expires_at = current_time + (certificate.ttl_min as u64 * 60 * 1000); // ttl_min转换为毫秒
    let expires_at_secs = expires_at / 1000; // 转换为秒

    // 创建JWT Claims
    let claims = TokenClaims {
        iss: "catastrophe".to_string(),    // 发行者标识
        sub: certificate.user.to_string(), // 用户地址作为主题
        exp: expires_at_secs,              // 过期时间（秒）
        iat: current_time_secs,            // 当前时间（秒）
        user_address: certificate.user,    // 用户地址
        session_vk: Base64::encode(certificate.session_vk.clone()), // 会话验证密钥
        creation_time: certificate.creation_time, // 证书创建时间
        ttl_min: certificate.ttl_min,
        profile,  // 用户档案
    };

    // 使用服务器的密钥对签名一个消息，然后将签名结果作为JWT的密钥
    let msg = b"jwt_secret";
    let signature: Ed25519Signature = app_state.eph_kp.sign(msg);
    let jwt_key = EncodingKey::from_secret(signature.as_ref());

    // 生成JWT令牌
    let auth_token = encode(&Header::new(Algorithm::HS256), &claims, &jwt_key)
        .expect("Failed to create JWT token");

    SessionTokenResponse {
        auth_token,
        expires_at,
        profile: None,
    }
}

/// 处理获取密钥的核心逻辑
async fn handle_session_token_core(
    app_state: &Arc<AppState>,
    headers: &HeaderMap,
    payload: &SessionTokenRequest,
    session: &Session,
) -> Result<SessionTokenResponse, InternalError> {
    let req_id = headers
        .get("Request-Id")
        .map(|v| v.to_str().unwrap_or_default());
    let version = headers.get("Client-Sdk-Version");
    let sdk_type = headers.get("Client-Sdk-Type");
    let target_api_version = headers.get("Client-Target-Api-Version");
    
    info!("请求头信息 - Request ID: {:?}, SDK版本: {:?}, SDK类型: {:?}, 目标API版本: {:?}", 
        req_id, version, sdk_type, target_api_version);
    
    app_state.metrics.observe_request("session_token");
    info!("检查全节点状态...");
    app_state.check_full_node_is_fresh(ALLOWED_STALENESS)?;
    
    let valid_function = format!("{}::{}::{}",&app_state.config["CITADEL_PACKAGE"],"citadel","seal_approve_verify_nexus_passport");
    info!("验证函数名称: {}", valid_function);

    info!("开始验证请求...");
    check_request(
        app_state,
        &payload.ptb,
        &payload.enc_key,
        &payload.enc_verification_key,
        &payload.request_signature,
        &payload.certificate,
        app_state.reference_gas_price(),
        Some(&app_state.metrics),
        req_id,
    )
    .await?;
    info!("请求验证通过");

    info!("开始解码PTB...");
    let ptb_b64 = match Base64::decode(&payload.ptb) {
        Ok(bytes) => {
            info!("PTB Base64解码成功");
            bytes
        },
        Err(e) => {
            error!("PTB Base64解码失败: {:?}", e);
            return Err(InternalError::InvalidPTB);
        }
    };
    
    let ptb: ProgrammableTransaction = match bcs::from_bytes(&ptb_b64) {
        Ok(tx) => {
            info!("PTB BCS反序列化成功");
            tx
        },
        Err(e) => {
            error!("PTB BCS反序列化失败: {:?}", e);
            return Err(InternalError::InvalidPTB);
        }
    };

    let valid_ptb = ValidPtb::try_from(ptb.clone()).unwrap();
    if valid_ptb.full_function() != valid_function {
        error!("PTB函数不匹配 - 期望: {}, 实际: {}", valid_function, valid_ptb.full_function());
        return Err(InternalError::InvalidPTB);
    }
    info!("PTB验证通过");

    let bytes = valid_ptb.inner_ids().first().unwrap().to_vec();
    let passport_id = format!("0x{}", hex::encode(&bytes));
    info!("提取护照ID: {}", passport_id);

    let svg = make_avatar(&passport_id);
    let svg_base64 = Base64::encode(svg.as_bytes());
    let avatar_data = format!("data:image/svg+xml;base64,{}", svg_base64);
    info!("头像生成完成");

    let profile = match ObjectID::from_hex_literal(&passport_id) {
        Ok(passport_obj_id) => {
            info!("护照ID转换为ObjectID成功: {:?}", passport_obj_id);
            match app_state.game_manager.get_profile_id_by_passport(&passport_obj_id).await {
                Ok(profile_id) => {
                    info!("找到现有档案ID: {:?}", profile_id);
                    match app_state.game_manager.get_profile(&profile_id).await {
                        Ok(profile) => {
                            info!("成功获取现有档案: {:?}", profile);
                            Some(profile)
                        },
                        Err(e) => {
                            error!("获取档案数据失败: {:?}", e);
                            None
                        }
                    }
                },
                Err(_) => {
                    info!("未找到现有档案，开始创建新档案...");
                    match create_profile_for_passport(
                        app_state,
                        &passport_id,
                        &avatar_data,
                    ).await {
                        Ok(_) => {
                            info!("新档案创建成功");
                            match ObjectID::from_hex_literal(&passport_id) {
                                Ok(passport_obj_id) => {
                                    info!("重新获取新创建的档案...");
                                    match app_state.game_manager.get_profile_id_by_passport(&passport_obj_id).await {
                                        Ok(profile_id) => {
                                            info!("获取到新档案ID: {:?}", profile_id);
                                            match app_state.game_manager.get_profile(&profile_id).await {
                                                Ok(profile) => {
                                                    info!("成功获取新创建的档案: {:?}", profile);
                                                    Some(profile)
                                                },
                                                Err(e) => {
                                                    error!("获取新档案数据失败: {:?}", e);
                                                    None
                                                }
                                            }
                                        },
                                        Err(e) => {
                                            error!("获取新档案ID失败: {:?}", e);
                                            None
                                        }
                                    }
                                },
                                Err(e) => {
                                    error!("护照ID格式无效: {:?}", e);
                                    None
                                }
                            }
                        },
                        Err(e) => {
                            error!("创建档案失败: {:?}", e);
                            None
                        }
                    }
                }
            }
        },
        Err(e) => {
            error!("护照ID格式无效: {:?}", e);
            return Err(InternalError::InvalidPTB);
        }
    };

    let mut response = create_session_token_response(
        app_state,
        &payload.certificate,
        profile.clone(),
    );

    // 设置 session
    let session_user = SessionUser {
        user_address: payload.certificate.user,
        session_vk: Base64::encode(payload.certificate.session_vk.clone()),
        exp: response.expires_at / 1000, // 转换为秒
        profile: profile.clone(),
    };
    session.insert(SESSION_USER_KEY, session_user).await?;

    if let Some(profile_data) = profile {
        response.profile = Some(profile_data);
    }

    info!("会话令牌处理完成");
    Ok(response)
}

/// 原始的session token处理函数
#[axum::debug_handler]
pub async fn handle_session_token(
    State(app_state): State<Arc<AppState>>,
    Extension(session): Extension<Session>,
    headers: HeaderMap,
    Json(payload): Json<SessionTokenRequest>,
) -> Result<Json<SessionTokenResponse>, InternalError> {
    handle_session_token_core(&app_state, &headers, &payload, &session)
        .await
        .map(Json)
        .tap_err(|e| app_state.metrics.observe_error(e.as_str()))
}



/// 获取用户Profile响应结构
#[derive(Debug, Serialize)]
pub struct GetUserCredentialsResponse {
    pub success: bool,
    pub credentials: Option<SessionUser>,
    pub error: Option<String>,
}

/**
 * 从 session 中获取当前登录用户
 * 
 * 在被 session 中间件保护的路由中使用
 */
#[axum::debug_handler]
pub async fn get_session_credentials(
    State(app_state): State<Arc<AppState>>,
    Extension(session): Extension<Session>,
) -> Result<Json<GetUserCredentialsResponse>, InternalError> {
    info!("收到获取用户Profile请求");
    app_state.metrics.observe_request("get_user_profile");

    // 从 session 中获取用户信息
    let user = session.get::<SessionUser>(SESSION_USER_KEY).await?
        .ok_or(InternalError::Unauthorized)?;

    if user.profile.is_none() {
        return Ok(Json(GetUserCredentialsResponse {
            success: false,
            credentials: Some(user),
            error: Some("用户档案为空".to_string()),
        }));
    }else{
        Ok(Json(GetUserCredentialsResponse {
            success: true,
            credentials: Some(user),
            error: None,
        }))
    }
}

/// 退出登录响应结构
#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    pub success: bool,
    pub message: String,
}

/// 退出登录接口
#[axum::debug_handler]
pub async fn handler_session_logout(
    Extension(session): Extension<Session>,
) -> Result<Json<LogoutResponse>, InternalError> {
    // 移除 session 中的用户信息
    let _ = session.remove::<serde_json::Value>(SESSION_USER_KEY).await.map_err(|e| {
        error!("移除 session 中的用户信息失败: {:?}", e);
        InternalError::Unauthorized
    })?;
    Ok(Json(LogoutResponse {
        success: true,
        message: "退出登录成功".to_string(),
    }))
} 