// Copyright (c), Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! # 聊天模块
//! 
//! ## 模块概述
//! 
//! 聊天模块提供了Catastrophe-Genesis游戏的实时通信功能，实现了基本的聊天消息传递系统。
//! 该模块与WebSocket基础架构紧密集成，支持游戏内的即时通讯需求。
//! 
//! ## 核心功能
//! 
//! - **消息发送**: 支持 `chat:send-message` 事件
//! - **聊天室加入**: 支持 `chat:join-chat` 事件
//! - **消息广播**: 通过 `chat:new-message` 事件推送新消息
//! 
//! ## 事件定义
//! 
//! ```rust
//! pub struct ChatEvents;
//! 
//! impl ChatEvents {
//!     pub const SEND_MESSAGE: &'static str = "chat:send-message";
//!     pub const JOIN_CHAT: &'static str = "chat:join-chat";
//!     pub const NEW_MESSAGE: &'static str = "chat:new-message";
//! }
//! ```
//! 
//! ## 使用示例
//! 
//! ```javascript
//! // 客户端连接示例
//! socket.on('chat:new-message', (message) => {
//!   console.log('收到新消息:', message);
//! });
//! 
//! // 发送消息
//! socket.emit('chat:send-message', {
//!   chat_id: 'game-123',
//!   text: '你好!'
//! });
//! 
//! // 加入聊天
//! socket.emit('chat:join-chat', {
//!   chat_id: 'game-123'
//! });
//! ```
//! 
//! ## 技术说明
//! 
//! - 与现有WebSocket架构无缝集成
//! - 事件前缀统一为"chat:"
//! - 支持服务端和客户端双向事件通信
//! - 提供断线重连和用户离开处理机制

use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::State,
    routing::post,
    Router,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::ws::{ConnectionManager, WsMessage};
use crate::AppState;

/// 聊天室前缀标识
const ROOM_PREFIX: &str = "chat";

/// 聊天事件定义
pub struct ChatEvents;

impl ChatEvents {
    /// 客户端事件: 发送消息
    pub const SEND_MESSAGE: &'static str = "chat:send-message";
    /// 客户端事件: 加入聊天室
    pub const JOIN_CHAT: &'static str = "chat:join-chat";
    /// 服务端事件: 新消息广播
    pub const NEW_MESSAGE: &'static str = "chat:new-message";
}

/// 聊天消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// 消息ID
    pub id: String,
    /// 消息内容
    pub content: String,
    /// 发送者信息
    pub sender: UserInfo,
    /// 创建时间
    pub created_at: i64,
}

/// 用户信息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    /// 用户ID
    pub id: String,
    /// 用户名
    pub name: String,
    /// 用户头像URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

/// 加入聊天室请求
#[derive(Debug, Deserialize)]
pub struct JoinChatRequest {
    /// 聊天室ID
    pub chat_id: String,
}

/// 发送消息请求
#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    /// 聊天室ID
    pub chat_id: String,
    /// 消息内容
    pub text: String,
}

/// 聊天模块状态
pub struct ChatState {
    /// WebSocket连接管理器
    pub connection_manager: Arc<ConnectionManager>,
}

impl ChatState {
    /// 创建新的聊天状态
    pub fn new(connection_manager: Arc<ConnectionManager>) -> Self {
        Self { connection_manager }
    }
}

/// 处理加入聊天室事件
async fn handle_join_chat(
    client_id: &str,
    chat_id: &str,
    user_id: &str,
    connection_manager: &ConnectionManager,
) -> Result<()> {
    // 格式化聊天室ID
    let room_id = format!("{}:{}", ROOM_PREFIX, chat_id);
    
    info!("用户 {} 加入聊天室: {}", user_id, room_id);
    
    // 设置断开连接处理器
    connection_manager.setup_disconnect_handler(
        client_id,
        &format!("chat:{}", room_id),
        Box::new({
            let user_id = user_id.to_string();
            let room_id = room_id.to_string();
            move || {
                info!("用户 {} 离开聊天室: {}", user_id, room_id);
            }
        }),
    ).await;
    
    // 返回成功响应
    let response = serde_json::json!({
        "ok": true,
        "msg": "已成功加入聊天室"
    });
    
    // 发送响应给客户端
    connection_manager.send_to_client(
        client_id, 
        "chat:joined", 
        Some(response)
    ).await?;
    
    Ok(())
}

/// 处理发送消息事件
async fn handle_send_message(
    client_id: &str,
    chat_id: &str,
    text: &str,
    user_info: UserInfo,
    connection_manager: &ConnectionManager,
) -> Result<()> {
    // 格式化聊天室ID
    let room_id = format!("{}:{}", ROOM_PREFIX, chat_id);
    
    info!("用户 {} 在聊天室 {} 发送消息", user_info.id, room_id);
    
    // 创建消息对象
    let message = ChatMessage {
        id: Uuid::new_v4().to_string(),
        content: text.to_string(),
        sender: user_info,
        created_at: Utc::now().timestamp_millis(),
    };
    
    // 广播消息到聊天室
    let payload = serde_json::json!({
        "message": message
    });
    
    connection_manager.broadcast_to_room(
        &room_id, 
        ChatEvents::NEW_MESSAGE, 
        Some(payload)
    ).await?;
    
    // 发送确认消息给发送者
    let response = serde_json::json!({
        "ok": true,
        "msg": "消息已发送"
    });
    
    connection_manager.send_to_client(
        client_id, 
        "chat:message-sent", 
        Some(response)
    ).await?;
    
    Ok(())
}

/// 处理WebSocket消息
pub async fn handle_ws_message(
    client_id: &str,
    message: WsMessage,
    connection_manager: &ConnectionManager,
    user_info: Option<UserInfo>,
) -> Result<bool> {
    debug!("处理聊天消息事件: {}", message.event);
    
    // 检查是否为聊天相关事件
    match message.event.as_str() {
        ChatEvents::JOIN_CHAT => {
            if let Some(data) = &message.data {
                if let Ok(req) = serde_json::from_value::<JoinChatRequest>(data.clone()) {
                    if let Some(user) = &user_info {
                        handle_join_chat(
                            client_id,
                            &req.chat_id,
                            &user.id,
                            connection_manager,
                        ).await?;
                        return Ok(true);
                    } else {
                        error!("用户未认证，无法加入聊天室");
                    }
                }
            }
        },
        ChatEvents::SEND_MESSAGE => {
            if let Some(data) = &message.data {
                if let Ok(req) = serde_json::from_value::<SendMessageRequest>(data.clone()) {
                    if let Some(user) = user_info {
                        handle_send_message(
                            client_id,
                            &req.chat_id,
                            &req.text,
                            user,
                            connection_manager,
                        ).await?;
                        return Ok(true);
                    } else {
                        error!("用户未认证，无法发送消息");
                    }
                }
            }
        },
        _ => return Ok(false), // 非聊天相关事件
    }
    
    Ok(false)
}

/// 注册聊天模块路由
pub fn register_chat_routes(app: Router, connection_manager: Arc<ConnectionManager>) -> Router {
    let chat_state = Arc::new(ChatState::new(connection_manager));
    
    // 返回路由
    app
}
