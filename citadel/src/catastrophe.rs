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
}

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
 *
 * 返回:
 * 包含JWT令牌的响应
 */
fn create_session_token_response(
    app_state: &AppState,
    certificate: &Certificate,
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
        ttl_min: certificate.ttl_min,      // 生存时间
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
) -> Result<SessionTokenResponse, InternalError> {
    let req_id = headers
        .get("Request-Id")
        .map(|v| v.to_str().unwrap_or_default());
    let version = headers.get("Client-Sdk-Version");
    let sdk_type = headers.get("Client-Sdk-Type");
    let target_api_version = headers.get("Client-Target-Api-Version");
    app_state.metrics.observe_request("session_token");
    app_state.check_full_node_is_fresh(ALLOWED_STALENESS)?;
    let valid_function = format!("{}::{}::{}",&app_state.config["CITADEL_PACKAGE"],"citadel","seal_approve_verify_nexus_passport");
    
    info!(
        "Request id: {:?}, SDK version: {:?}, SDK type: {:?}, Target API version: {:?}, function: {:?}",
        req_id, version, sdk_type, target_api_version, valid_function
    );

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

    let ptb_b64 = match Base64::decode(&payload.ptb) {
        Ok(bytes) => bytes,
        Err(_) => return Err(InternalError::InvalidPTB),
    };
    
    let ptb: ProgrammableTransaction = match bcs::from_bytes(&ptb_b64) {
        Ok(tx) => tx,
        Err(_) => return Err(InternalError::InvalidPTB),
    };

    let valid_ptb = ValidPtb::try_from(ptb.clone()).unwrap();
    if valid_ptb.full_function() != valid_function {
        return Err(InternalError::InvalidPTB);
    }

    // 获取 passport ID
    let passport_id = match valid_ptb.inner_ids().first() {
        Some(id) => match String::from_utf8(id.clone()) {
            Ok(id_str) => id_str,
            Err(_) => return Err(InternalError::InvalidPTB),
        },
        None => return Err(InternalError::InvalidPTB),
    };

    // 生成头像
    let svg = make_avatar(&passport_id);
    let svg_base64 = Base64::encode(svg.as_bytes());
    let avatar_data = format!("data:image/svg+xml;base64,{}", svg_base64);

    // 获取或创建用户档案
    let profile = match ObjectID::from_hex_literal(&passport_id) {
        Ok(passport_obj_id) => {
            match app_state.game_manager.get_profile_id_by_passport(&passport_obj_id).await {
                Ok(profile_id) => {
                    match app_state.game_manager.get_profile(&profile_id).await {
                        Ok(profile) => {
                            info!("Found existing profile for passport {}: {:?}", passport_id, profile);
                            Some(profile)
                        },
                        Err(e) => {
                            error!("Failed to get profile data for {}: {:?}", passport_id, e);
                            None
                        }
                    }
                },
                Err(_) => {
                    match create_profile_for_passport(
                        app_state,
                        &passport_id,
                        &avatar_data,
                    ).await {
                        Ok(_) => {
                            info!("Successfully created profile for {}", passport_id);
                            match ObjectID::from_hex_literal(&passport_id) {
                                Ok(passport_obj_id) => {
                                    match app_state.game_manager.get_profile_id_by_passport(&passport_obj_id).await {
                                        Ok(profile_id) => {
                                            match app_state.game_manager.get_profile(&profile_id).await {
                                                Ok(profile) => Some(profile),
                                                Err(e) => {
                                                    error!("Failed to get new profile data for {}: {:?}", passport_id, e);
                                                    None
                                                }
                                            }
                                        },
                                        Err(e) => {
                                            error!("Failed to get new profile id for {}: {:?}", passport_id, e);
                                            None
                                        }
                                    }
                                },
                                Err(e) => {
                                    error!("Invalid passport ID format after creation: {:?}", e);
                                    None
                                }
                            }
                        },
                        Err(e) => {
                            error!("Create profile for {} failed: {:?}", passport_id, e);
                            None
                        }
                    }
                }
            }
        },
        Err(e) => {
            error!("Invalid passport ID format: {:?}", e);
            return Err(InternalError::InvalidPTB);
        }
    };

    let mut response = create_session_token_response(
        app_state,
        &payload.certificate,
    );

    if let Some(profile_data) = profile {
        response.profile = Some(profile_data);
    }

    Ok(response)
}

/// 原始的session token处理函数
pub async fn handle_session_token(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<SessionTokenRequest>,
) -> Result<Json<SessionTokenResponse>, InternalError> {
    handle_session_token_core(&app_state, &headers, &payload)
        .await
        .map(Json)
        .tap_err(|e| app_state.metrics.observe_error(e.as_str()))
}

/// 加密的session token响应
#[derive(Serialize, Deserialize)]
pub struct EncryptedSessionTokenResponse {
    pub encrypted_data: ElgamalEncryption,
}

/// 加密版本的session token处理函数
pub async fn handle_encrypted_session_token(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<SessionTokenRequest>,
) -> Result<Json<EncryptedSessionTokenResponse>, InternalError> {
    // 获取核心响应
    let response = handle_session_token_core(&app_state, &headers, &payload).await?;
    
    // 序列化响应
    let response_bytes = serde_json::to_vec(&response)
        .map_err(|_| InternalError::SerializationError)?;
    
    // 使用用户的公钥加密响应
    let key = ibe::extract(&app_state.master_key, response_bytes.as_slice());
    let encrypted_data = encrypt(
        &mut thread_rng(),
        &key,
        &payload.enc_key,
    );

    Ok(Json(EncryptedSessionTokenResponse {
        encrypted_data,
    }))
}

/// 头像请求参数
#[derive(Debug, Deserialize)]
pub struct AvatarParams {
    /// 用于生成头像的种子字符串
    pub address: Option<String>,
    /// 头像的性别：male 或 female
    pub gender: Option<String>,
}

/// 处理头像生成请求
pub async fn generate_avatar(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<AvatarParams>,
) -> impl IntoResponse {
    // 使用当前时间戳作为默认种子
    let seed = params.address.unwrap_or_else(|| {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        timestamp.to_string()
    });
    // 根据参数生成SVG
    let svg = match params.gender.as_deref() {
        Some("male") => make_male_avatar(&seed),
        Some("female") => make_female_avatar(&seed),
        _ => make_avatar(&seed),
    };
   
    // 返回SVG图像
    (
        StatusCode::OK,
        [
            ("Content-Type", "image/svg+xml"),
            ("Cache-Control", "public, max-age=86400"),
        ],
        svg,
    )
}


/**
 * 创建用户档案请求结构
 * 
 * 用于测试SDK中的create_profile_for_passport函数
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProfileRequest {
    pub passport_id: String,  // 护照ID (SuiAddress格式)
}

/**
 * 创建用户档案响应结构
 * 
 * 包含交易结果信息
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProfileResponse {
    pub success: bool,              // 是否成功
    pub digest: Option<String>,     // 交易摘要
    pub error: Option<String>,      // 错误信息(如果有)
}

/**
 * 处理创建用户档案请求
 * 
 * 用于测试SDK中的create_profile_for_passport函数
 * 注意：此端点仅用于测试目的，生产环境应该使用适当的认证机制
 */
pub async fn handle_create_profile(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<CreateProfileRequest>,
) -> Result<Json<CreateProfileResponse>, StatusCode> {
    info!("收到创建用户档案请求: {:?}", payload);
    app_state.metrics.observe_request("test_create_profile");
    // 生成头像
    let svg = make_avatar(&payload.passport_id);

    // 将 SVG 转换为 base64
    let svg_base64 = Base64::encode(svg.as_bytes());
    let avatar_data = format!("data:image/svg+xml;base64,{}", svg_base64);
    
    // 调用SDK函数
    match create_profile_for_passport(
        &app_state,
        &payload.passport_id,
        &avatar_data,
    ).await {
        Ok(response) => {
            // 成功创建档案
            let digest = response.digest.to_string();
            // 使用Network方法生成浏览器URL
            let tx_url = app_state.network.explorer_tx_url(&digest);
            info!("成功创建用户档案，交易摘要: {}", tx_url);
            Ok(Json(CreateProfileResponse {
                success: true,
                digest: Some(digest),
                error: None,
            }))
        },
        Err(err) => {
            // 创建失败
            warn!("创建用户档案失败: {:?}", err);
            Ok(Json(CreateProfileResponse {
                success: false,
                digest: None,
                error: Some(err.to_string()),
            }))
        }
    }
}


/// 获取用户档案请求结构
#[derive(Debug, Serialize, Deserialize)]
pub struct GetProfileRequest {
    pub passport_id: String,  // 护照ID (SuiAddress格式)
}

/// 获取用户档案响应结构
#[derive(Debug, Serialize, Deserialize)]
pub struct GetProfileResponse {
    pub success: bool,
    pub profile: Option<Profile>,  // 用户档案信息
    pub error: Option<String>,    // 错误信息(如果有)
}

/// 处理获取用户档案请求
/// 
/// 用于测试从GameManager获取用户档案信息
/// 注意：此端点仅用于测试目的
pub async fn handle_get_profile(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<GetProfileRequest>,
) -> Result<Json<GetProfileResponse>, StatusCode> {
    info!("收到获取用户档案请求: {:?}", payload);
    app_state.metrics.observe_request("test_get_profile");

    // 将护照ID转换为ObjectID
    let passport_id = match ObjectID::from_hex_literal(&payload.passport_id) {
        Ok(id) => id,
        Err(err) => {
            return Ok(Json(GetProfileResponse {
                success: false,
                profile: None,
                error: Some(format!("无效的护照ID: {}", err)),
            }));
        }
    };
    info!("passport_id: {:?}", passport_id);

    // 使用GameManager获取用户档案
    match app_state.game_manager.get_profile_id_by_passport(&passport_id).await {
        Ok(profile_id) => {
            match app_state.game_manager.get_profile(&profile_id).await {
                Ok(profile) => {
                    info!("成功获取用户档案: {:?}", profile);
                    Ok(Json(GetProfileResponse {
                        success: true,
                        profile: Some(profile),
                        error: None,
                    }))
                },
                Err(err) => {
                    warn!("获取用户档案失败: {:?}", err);
                    Ok(Json(GetProfileResponse {
                        success: false,
                        profile: None,
                        error: Some(format!("获取档案信息失败: {}", err)),
                    }))
                }
            }
        },
        Err(err) => {
            warn!("获取用户档案ID失败: {:?}", err);
            Ok(Json(GetProfileResponse {
                success: false,
                profile: None,
                error: Some(format!("获取档案ID失败: {}", err)),
            }))
        }
    }
} 