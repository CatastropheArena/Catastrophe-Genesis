// Copyright (c), Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::auth::AuthenticatedUser;
use crate::AppState;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

/**
 * 受保护资源响应结构
 *
 * 在成功验证JWT令牌后返回的数据
 */
#[derive(Serialize, Deserialize)]
pub struct ProtectedResourceResponse {
    pub message: String,
    pub user_address: String,
}

/**
 * 用户信息响应结构
 *
 * 返回有关当前登录用户的信息
 */
#[derive(Serialize, Deserialize)]
pub struct UserInfoResponse {
    pub user_address: String,
    pub token_expires_at: u64,
}

/**
 * 获取受保护资源的示例处理器
 *
 * 此处理器展示如何在认证中间件保护的路由中访问用户信息
 */
pub async fn get_protected_resource(
    State(app_state): State<Arc<AppState>>,
    request: Request,
) -> Json<ProtectedResourceResponse> {
    // 从请求中获取认证用户信息（由auth_middleware添加）
    let authenticated_user = request
        .extensions()
        .get::<AuthenticatedUser>()
        .expect("authenticated_user extension should be available");
    
    info!("访问受保护资源的用户: {:?}", authenticated_user.user_address);
    
    Json(ProtectedResourceResponse {
        message: "您已成功访问受保护的资源".to_string(),
        user_address: authenticated_user.user_address.to_string(),
    })
}

/**
 * 获取当前用户信息的处理器
 *
 * 返回当前登录用户的基本信息
 */
pub async fn get_current_user(
    State(app_state): State<Arc<AppState>>,
    request: Request,
) -> Json<UserInfoResponse> {
    // 从请求中获取认证用户信息（由auth_middleware添加）
    let authenticated_user = request
        .extensions()
        .get::<AuthenticatedUser>()
        .expect("authenticated_user extension should be available");
    
    Json(UserInfoResponse {
        user_address: authenticated_user.user_address.to_string(),
        token_expires_at: authenticated_user.exp,
    })
} 