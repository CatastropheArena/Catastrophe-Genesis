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
use crate::session_login::{SessionUser, SESSION_USER_KEY};
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
use crate::txb;
use axum::{
    routing::{get, post},
    Router,
};
use crate::sdk::executor;


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

/// 获取用户Profile响应结构
#[derive(Debug, Serialize)]
pub struct GetUserProfileResponse {
    pub success: bool,
    pub profile: Option<Profile>,
    pub error: Option<String>,
}

/// 处理获取用户Profile请求
/// 
/// 从 session 中获取用户地址，并返回对应的Profile信息
#[axum::debug_handler]
pub async fn handle_get_user_profile(
    State(app_state): State<Arc<AppState>>,
    Extension(session): Extension<Session>,
) -> Result<Json<GetUserProfileResponse>, InternalError> {
    info!("收到获取用户Profile请求");
    app_state.metrics.observe_request("get_user_profile");

    // 从 session 中获取用户信息
    let user = session.get::<SessionUser>(SESSION_USER_KEY).await?
        .ok_or(InternalError::Unauthorized)?;

    if user.profile.is_none() {
        return Ok(Json(GetUserProfileResponse {
            success: false,
            profile: None,
            error: Some("用户档案为空".to_string()),
        }));
    }
    
    info!("认证用户地址: {:?}", user.user_address);

    // 将用户地址转换为ObjectID
    let passport_id = ObjectID::from(user.user_address);
    
    // 使用GameManager获取用户档案
    match app_state.game_manager.get_profile_id_by_passport(&passport_id).await {
        Ok(profile_id) => {
            match app_state.game_manager.get_profile(&profile_id).await {
                Ok(profile) => {
                    info!("成功获取用户档案: {:?}", profile);
                    Ok(Json(GetUserProfileResponse {
                        success: true,
                        profile: Some(profile),
                        error: None,
                    }))
                },
                Err(err) => {
                    warn!("获取用户档案失败: {:?}", err);
                    Ok(Json(GetUserProfileResponse {
                        success: false,
                        profile: None,
                        error: Some(format!("获取档案信息失败: {}", err)),
                    }))
                }
            }
        },
        Err(err) => {
            warn!("获取用户档案ID失败: {:?}", err);
            Ok(Json(GetUserProfileResponse {
                success: false,
                profile: None,
                error: Some(format!("获取档案ID失败: {}", err)),
            }))
        }
    }
}

/// 管理员发送好友请求的请求结构
#[derive(Debug, Serialize, Deserialize)]
pub struct AdminSendFriendRequestRequest {
    pub from_profile_id: String,  // 发送者的 Profile ID
    pub to_profile_id: String,    // 接收者的 Profile ID
}

/// 管理员发送好友请求的响应结构
#[derive(Debug, Serialize, Deserialize)]
pub struct AdminSendFriendRequestResponse {
    pub success: bool,
    pub digest: Option<String>,
    pub error: Option<String>,
}

/// 处理管理员发送好友请求
pub async fn handle_admin_send_friend_request(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<AdminSendFriendRequestRequest>,
) -> Result<Json<AdminSendFriendRequestResponse>, StatusCode> {
    info!("收到管理员发送好友请求: {:?}", payload);
    app_state.metrics.observe_request("admin_send_friend_request");

    // 将 profile ID 转换为 ObjectID
    let from_profile_id = ObjectID::from_hex_literal(&payload.from_profile_id)
        .map_err(|e| {
            warn!("无效的发送者 Profile ID: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    let to_profile_id = ObjectID::from_hex_literal(&payload.to_profile_id)
        .map_err(|e| {
            warn!("无效的接收者 Profile ID: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    // 调用执行器函数
    match executor::admin_send_friend_request(&app_state, &from_profile_id, &to_profile_id).await {
        Ok(response) => {
            let digest = response.digest.to_string();
            let tx_url = app_state.network.explorer_tx_url(&digest);
            info!("管理员成功发送好友请求，交易摘要: {}", tx_url);
            Ok(Json(AdminSendFriendRequestResponse {
                success: true,
                digest: Some(digest),
                error: None,
            }))
        },
        Err(err) => {
            warn!("管理员发送好友请求失败: {:?}", err);
            Ok(Json(AdminSendFriendRequestResponse {
                success: false,
                digest: None,
                error: Some(err.to_string()),
            }))
        }
    }
}

/// 获取好友关系请求结构
#[derive(Debug, Serialize, Deserialize)]
pub struct GetRelationshipRequest {
    pub user_id: String,     // 用户的 Profile ID
    pub profile_id: String,  // 目标用户的 Profile ID
}

/// 获取好友关系响应结构
#[derive(Debug, Serialize, Deserialize)]
pub struct GetRelationshipResponse {
    pub success: bool,
    pub relationship: Option<crate::sdk::manager::Relationship>,  // 好友关系信息
    pub error: Option<String>,
}

/// 处理获取好友关系请求
pub async fn handle_get_relationship(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<GetRelationshipRequest>,
) -> Result<Json<GetRelationshipResponse>, StatusCode> {
    info!("收到获取好友关系请求: {:?}", payload);
    app_state.metrics.observe_request("test_get_relationship");

    // 将 ID 转换为 ObjectID
    let user_id = match ObjectID::from_hex_literal(&payload.user_id) {
        Ok(id) => id,
        Err(err) => {
            return Ok(Json(GetRelationshipResponse {
                success: false,
                relationship: None,
                error: Some(format!("无效的用户ID: {}", err)),
            }));
        }
    };

    let profile_id = match ObjectID::from_hex_literal(&payload.profile_id) {
        Ok(id) => id,
        Err(err) => {
            return Ok(Json(GetRelationshipResponse {
                success: false,
                relationship: None,
                error: Some(format!("无效的目标用户ID: {}", err)),
            }));
        }
    };

    // 使用 GameManager 获取好友关系
    match app_state.game_manager.get_relationship(&user_id, &profile_id).await {
        Ok(relationship) => {
            info!("成功获取好友关系: {:?}", relationship);
            Ok(Json(GetRelationshipResponse {
                success: true,
                relationship,
                error: None,
            }))
        },
        Err(err) => {
            warn!("获取好友关系失败: {:?}", err);
            Ok(Json(GetRelationshipResponse {
                success: false,
                relationship: None,
                error: Some(format!("获取好友关系失败: {}", err)),
            }))
        }
    }
}

/// 注册 Catastrophe 相关路由
pub fn register_catastrophe_routes(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router
        .route("/test/create_profile", post(handle_create_profile))
        .route("/test/get_profile", post(handle_get_profile))
        .route("/user/profile", get(handle_get_user_profile))
        .route("/test/avatar", get(generate_avatar))
        .route("/test/send_friend_request", post(handle_admin_send_friend_request))
        .route("/test/get_relationship", post(handle_get_relationship))
}