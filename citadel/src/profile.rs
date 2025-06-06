use axum::{
    debug_handler, extract::{Path, State}, http::StatusCode, response::IntoResponse, routing::{get, post}, Extension, Json, Router
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_sessions::Session;
use tracing::{info, error};
use anyhow::Result;

use crate::AppState;
use crate::session_login::{SessionUser, SESSION_USER_KEY};
use crate::errors::InternalError;
use crate::sdk::{Profile,ProfileWithRelationship};

/// 用户统计信息响应
#[derive(Debug, Serialize)]
pub struct UserStats {
    /// 胜场数
    pub won: u64,
    /// 负场数
    pub lost: u64,
    /// 总场数
    pub played: u64,
    /// 胜率
    pub winrate: u64,
    /// 评分
    pub rating: u64,
}

/// 用户档案响应
#[derive(Debug, Serialize)]
pub struct ProfileResponse {
    pub success: bool,
    pub profile: Option<ProfileWithRelationship>,
    pub error: Option<String>,
}

/// 用户统计信息响应
#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub success: bool,
    pub stats: Option<UserStats>,
    pub error: Option<String>,
}

/// 获取当前用户档案
#[debug_handler]
pub async fn get_my_profile(
    State(app_state): State<Arc<AppState>>,
    Extension(session): Extension<Session>,
) -> Result<Json<ProfileResponse>, InternalError> {
    info!("收到获取当前用户档案请求");
    
    // 从session获取用户信息
    let user = session.get::<SessionUser>(SESSION_USER_KEY).await?
        .ok_or(InternalError::Unauthorized)?;
    
    if let Some(profile) = user.profile {
        // 获取带关系信息的Profile
        match app_state.game_manager.get_profile_with_relationship(&profile.id, None).await {
            Ok(profile_with_relationship) => {
                Ok(Json(ProfileResponse {
                    success: true,
                    profile: Some(profile_with_relationship),
                    error: None,
                }))
            },
            Err(e) => {
                error!("获取用户档案失败: {}", e);
                Ok(Json(ProfileResponse {
                    success: false,
                    profile: None,
                    error: Some(format!("获取用户档案失败: {}", e)),
                }))
            }
        }
    } else {
        Ok(Json(ProfileResponse {
            success: false,
            profile: None,
            error: Some("用户档案不存在".to_string()),
        }))
    }
}

/// 获取指定用户档案
#[debug_handler]
pub async fn get_user_profile(
    State(app_state): State<Arc<AppState>>,
    Path(profile_id): Path<String>,
    Extension(session): Extension<Session>,
) -> Result<Json<ProfileResponse>, InternalError> {
    info!("收到获取用户档案请求: {}", profile_id);
    
    // 将ProfileID转换为ObjectID
    let profile_obj_id = sui_types::base_types::ObjectID::from_hex_literal(&profile_id)
        .map_err(|_| InternalError::InvalidInput)?;
    
    // 从session获取当前用户信息(如果有)
    let current_user = session.get::<SessionUser>(SESSION_USER_KEY).await?;
    let current_user_profile = current_user.and_then(|u| u.profile);
    
    // 获取用户档案(带关系信息)
    match app_state.game_manager.get_profile_with_relationship(
        &profile_obj_id,
        current_user_profile.as_ref().map(|p| &p.id)
    ).await {
        Ok(profile) => {
            Ok(Json(ProfileResponse {
                success: true,
                profile: Some(profile),
                error: None,
            }))
        },
        Err(e) => {
            error!("获取用户档案失败: {}", e);
            Ok(Json(ProfileResponse {
                success: false,
                profile: None,
                error: Some(format!("获取用户档案失败: {}", e)),
            }))
        }
    }
}

/// 获取当前用户统计信息
#[debug_handler]
pub async fn get_my_stats(
    State(app_state): State<Arc<AppState>>,
    Extension(session): Extension<Session>,
) -> Result<Json<StatsResponse>, InternalError> {
    info!("收到获取当前用户统计信息请求");
    
    // 从session获取用户信息
    let user = session.get::<SessionUser>(SESSION_USER_KEY).await?
        .ok_or(InternalError::Unauthorized)?;
    
    if let Some(profile) = user.profile {
        let stats = UserStats {
            won: profile.won,
            lost: profile.lost,
            played: profile.played,
            winrate: if profile.played > 0 {
                (profile.won as f64 / profile.played as f64 * 100.0) as u64
            } else {
                0
            },
            rating: profile.rating,
        };
        
        Ok(Json(StatsResponse {
            success: true,
            stats: Some(stats),
            error: None,
        }))
    } else {
        Ok(Json(StatsResponse {
            success: false,
            stats: None,
            error: Some("用户档案不存在".to_string()),
        }))
    }
}

/// 获取指定用户统计信息
#[debug_handler]
pub async fn get_user_stats(
    State(app_state): State<Arc<AppState>>,
    Path(profile_id): Path<String>,
) -> Result<Json<StatsResponse>, InternalError> {
    info!("收到获取用户统计信息请求: {}", profile_id);
    
    // 将ProfileID转换为ObjectID
    let profile_obj_id = sui_types::base_types::ObjectID::from_hex_literal(&profile_id)
        .map_err(|_| InternalError::InvalidInput)?;
    
    // 获取用户档案
    match app_state.game_manager.get_profile(&profile_obj_id).await {
        Ok(profile) => {
            let stats = UserStats {
                won: profile.won,
                lost: profile.lost,
                played: profile.played,
                winrate: if profile.played > 0 {
                    (profile.won as f64 / profile.played as f64 * 100.0) as u64
                } else {
                    0
                },
                rating: profile.rating,
            };
            
            Ok(Json(StatsResponse {
                success: true,
                stats: Some(stats),
                error: None,
            }))
        },
        Err(e) => {
            error!("获取用户档案失败: {}", e);
            Ok(Json(StatsResponse {
                success: false,
                stats: None,
                error: Some(format!("获取用户档案失败: {}", e)),
            }))
        }
    }
}

/// 注册Profile路由
pub fn register_profile_routes(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router
        .route("/profile/me", get(get_my_profile))
        .route("/profile/me/stats", get(get_my_stats))
        .route("/profile/:profile_id", get(get_user_profile))
        .route("/profile/:profile_id/stats", get(get_user_stats))
} 