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

/**
 * 从 session 中获取当前登录用户
 * 
 * 在被 session 中间件保护的路由中使用
 */
#[axum::debug_handler]
pub async fn get_session_credentials(
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
    }else{
        Ok(Json(GetUserProfileResponse {
            success: true,
            profile: user.profile,
            error: None,
        }))
    }
} 