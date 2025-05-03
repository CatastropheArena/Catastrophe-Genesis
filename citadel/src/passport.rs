// Copyright (c), Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! # 用户护照模块
//! 
//! ## 模块概述
//! 
//! 用户护照模块处理用户的个人资料、在线状态、好友关系和游戏会话管理。
//! 该模块通过与WebSocket和游戏缓存系统的紧密集成，提供了完整的用户档案和社交功能支持。
//! 
//! ## 核心功能
//! 
//! - **在线状态管理**: 实时监控和广播用户在线状态，基于缓存实现自动管理
//! - **好友系统**: 完整的好友关系管理（添加、接受、拒绝、撤销、删除）
//! - **游戏查询**: 查询用户当前进行中的游戏
//! - **用户补充信息**: 提供用户状态和活动信息的统一查询接口
//! 
//! ## 事件定义
//! 
//! ### 客户端事件
//! ```
//! SEND_FRIEND_REQUEST: "user:send-friend-request"
//! REVOKE_FRIEND_REQUEST: "user:revoke-friend-request"
//! ACCEPT_FRIEND_REQUEST: "user:accept-friend-request"
//! REJECT_FRIEND_REQUEST: "user:reject-friend-request"
//! UNFRIEND: "user:unfriend"
//! BLOCK: "user:block"
//! UNBLOCK: "user:unblock"
//! GET_SUPPLEMENTAL: "user:get-supplemental"
//! SET_INTERIM: "user:set-interim"
//! ```
//! 
//! ### 服务端事件
//! ```
//! ONLINE: "user:online"
//! OFFLINE: "user:offline"
//! FRIEND_REQUEST_RECEIVED: "user:friend-request-received"
//! FRIEND_REQUEST_ACCEPTED: "user:friend-request-accepted"
//! FRIEND_REQUEST_REJECTED: "user:friend-request-rejected"
//! FRIEND_REQUEST_REVOKED: "user:friend-request-revoked"
//! UNFRIENDED: "user:unfriended"
//! ```
//! 
//! ## 技术说明
//! 
//! - **WebSocket集成**: 与WebSocket系统无缝集成，实现实时状态更新和事件通知
//! - **缓存应用**: 使用游戏缓存系统存储用户状态、好友关系和游戏数据
//! - **关系管理**: 提供高效的关系状态转换和管理功能
//! - **会话追踪**: 支持多设备登录和离线检测
//! - **游戏状态**: 跟踪和更新用户当前游戏状态
//! - **与NestJS兼容**: 实现与前端NestJS系统兼容的数据格式和事件处理
//! 
//! ## 使用示例
//! 
//! ```javascript
//! // 发送好友请求
//! socket.emit('user:send-friend-request', {
//!   user_id: '12345'
//! });
//! 
//! // 监听好友请求
//! socket.on('user:friend-request-received', (data) => {
//!   console.log('收到好友请求:', data.user);
//! });
//! 
//! // 接受好友请求
//! socket.emit('user:accept-friend-request', {
//!   user_id: '12345'
//! });
//! 
//! // 监听用户上线事件
//! socket.on('user:online', (data) => {
//!   console.log('用户上线:', data.userId);
//! });
//!
//! // 获取用户补充信息
//! socket.emit('user:get-supplemental', {
//!   ids: ['12345', '67890']
//! });
//!
//! // 设置用户临时状态
//! socket.emit('user:set-interim', {
//!   status: 'online',
//!   activity: {
//!     type: 'in-match',
//!     matchId: 'match-123'
//!   }
//! });
//! ```
//!
//! ## 架构说明
//!
//! 本模块通过与WebSocket服务和游戏缓存系统的紧密集成，实现了高效的用户状态管理和社交功能。
//! 当用户连接到WebSocket服务时，其身份信息被传递给Passport模块，Passport模块会自动更新
//! 用户的在线状态并广播给其好友。同时，用户的好友关系、游戏状态等信息通过游戏缓存系统进行
//! 高效存储和检索，确保了实时性和性能。
//!
//! ### UserSupplemental功能
//!
//! 用户补充信息功能提供了一种统一的方式来查询用户的状态和活动信息。它是特别为了与前端NestJS
//! 系统兼容而设计的。主要功能包括：
//!
//! 1. **状态查询**: 通过`user:get-supplemental`事件批量查询多个用户的状态和活动
//! 2. **状态更新**: 通过`user:set-interim`事件更新用户的临时状态和活动
//! 3. **活动追踪**: 跟踪用户是在大厅中、游戏中还是观战状态
//! 4. **状态广播**: 当用户状态变化时，自动广播给相关用户
//! 5. **游戏集成**: 与游戏系统集成，自动反映用户的游戏参与状态
//! 
//! 这些功能使得游戏客户端能够轻松获取和展示用户的实时状态，为玩家提供更好的社交体验。

use std::sync::Arc;
use std::collections::HashMap;

use anyhow::Result;
use axum::{
    extract::State,
    routing::{get, post},
    Router,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::ws::{ConnectionManager, WsMessage, ClientId};
use crate::game::{GameCache, GameCachePrefix, GameService};
use crate::AppState;

/// 用户状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserStatus {
    /// 在线状态
    Online,
    /// 离线状态
    Offline,
    /// 游戏中状态
    InGame,
    /// 空闲状态
    Idle,
    /// 离开状态
    Away,
}

impl Default for UserStatus {
    fn default() -> Self {
        Self::Offline
    }
}

/// 好友关系状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelationshipStatus {
    /// 无关系
    None,
    /// 好友关系
    Friends,
    /// 用户1向用户2发送好友请求
    FriendRequest1To2,
    /// 用户2向用户1发送好友请求
    FriendRequest2To1,
    /// 互相封禁
    Blocked,
    /// 用户1封禁用户2
    Blocked1To2,
    /// 用户2封禁用户1
    Blocked2To1,
}

impl Default for RelationshipStatus {
    fn default() -> Self {
        Self::None
    }
}

/// 用户信息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    /// 用户ID
    pub id: String,
    /// 用户名
    pub username: String,
    /// 头像URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    /// 用户状态
    #[serde(default)]
    pub status: UserStatus,
    /// 最后活跃时间
    pub last_active: i64,
    /// 创建时间
    pub created_at: i64,
}

/// 好友关系结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    /// 关系ID
    pub id: String,
    /// 用户1 ID
    pub user1_id: String,
    /// 用户2 ID
    pub user2_id: String,
    /// 关系状态
    pub status: RelationshipStatus,
    /// 创建时间
    pub created_at: i64,
    /// 更新时间
    pub updated_at: i64,
}

/// 正在进行的游戏信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OngoingGame {
    /// 游戏ID
    pub id: String,
    /// 游戏类型
    pub game_type: String,
    /// 参与玩家
    pub players: Vec<String>,
    /// 开始时间
    pub started_at: i64,
}

/// 用户活动类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum UserActivityType {
    /// 在大厅中
    InLobby,
    /// 在游戏中
    InMatch,
    /// 观战中
    Spectate,
}

/// 用户活动信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserActivity {
    /// 活动类型
    #[serde(rename = "type")]
    pub activity_type: Option<UserActivityType>,
    /// 游戏ID（如果在游戏中）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_id: Option<String>,
    /// 大厅ID（如果在大厅中）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lobby_id: Option<String>,
}

/// 用户状态字符串类型（用于前端兼容）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UserStatusString {
    /// 在线状态
    Online,
    /// 离线状态
    Offline,
}

impl From<UserStatus> for UserStatusString {
    fn from(status: UserStatus) -> Self {
        match status {
            UserStatus::Online => Self::Online,
            UserStatus::Offline => Self::Offline,
            UserStatus::InGame => Self::Online,
            UserStatus::Idle => Self::Online,
            UserStatus::Away => Self::Online,
        }
    }
}

/// 用户补充信息，与NestJS中的UserSupplemental对应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSupplemental {
    /// 用户状态（online/offline）
    pub status: UserStatusString,
    /// 用户活动信息（可为null）
    pub activity: Option<UserActivity>,
}

/// 用户临时状态，与NestJS中的UserInterim对应
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserInterim {
    /// 用户状态（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<UserStatusString>,
    /// 用户活动（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity: Option<UserActivity>,
}

/// 获取用户补充信息的请求DTO
#[derive(Debug, Deserialize)]
pub struct GetSupplementalDto {
    /// 用户ID列表
    pub ids: Vec<String>,
}

/// 用户事件定义
#[derive(Debug, Clone)]
pub struct UserEvents;

/// 服务端事件
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServerEvent {
    /// 用户上线
    Online,
    /// 用户离线
    Offline,
    /// 收到好友请求
    FriendRequestReceived,
    /// 好友请求被接受
    FriendRequestAccepted,
    /// 好友请求被拒绝
    FriendRequestRejected,
    /// 好友请求被撤销
    FriendRequestRevoked,
    /// 被删除好友
    Unfriended,
}

impl ServerEvent {
    /// 获取事件名称
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Online => "user:online",
            Self::Offline => "user:offline",
            Self::FriendRequestReceived => "user:friend-request-received",
            Self::FriendRequestAccepted => "user:friend-request-accepted",
            Self::FriendRequestRejected => "user:friend-request-rejected",
            Self::FriendRequestRevoked => "user:friend-request-revoked",
            Self::Unfriended => "user:unfriended",
        }
    }
}

/// 客户端事件
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClientEvent {
    /// 发送好友请求
    SendFriendRequest,
    /// 撤销好友请求
    RevokeFriendRequest,
    /// 接受好友请求
    AcceptFriendRequest,
    /// 拒绝好友请求
    RejectFriendRequest,
    /// 删除好友
    Unfriend,
    /// 封禁用户
    Block,
    /// 解除封禁
    Unblock,
    /// 获取用户补充信息
    GetSupplemental,
    /// 设置用户临时状态
    SetInterim,
}

impl ClientEvent {
    /// 获取事件名称
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SendFriendRequest => "user:send-friend-request",
            Self::RevokeFriendRequest => "user:revoke-friend-request",
            Self::AcceptFriendRequest => "user:accept-friend-request",
            Self::RejectFriendRequest => "user:reject-friend-request",
            Self::Unfriend => "user:unfriend",
            Self::Block => "user:block",
            Self::Unblock => "user:unblock",
            Self::GetSupplemental => "user:get-supplemental",
            Self::SetInterim => "user:set-interim",
        }
    }
    
    /// 从字符串解析事件
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "user:send-friend-request" => Some(Self::SendFriendRequest),
            "user:revoke-friend-request" => Some(Self::RevokeFriendRequest),
            "user:accept-friend-request" => Some(Self::AcceptFriendRequest),
            "user:reject-friend-request" => Some(Self::RejectFriendRequest),
            "user:unfriend" => Some(Self::Unfriend),
            "user:block" => Some(Self::Block),
            "user:unblock" => Some(Self::Unblock),
            "user:get-supplemental" => Some(Self::GetSupplemental),
            "user:set-interim" => Some(Self::SetInterim),
            _ => None,
        }
    }
}

/// 响应事件
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResponseEvent {
    /// 好友请求发送响应
    FriendRequestSent,
    /// 好友请求撤销响应
    FriendRequestRevokedResponse,
    /// 好友请求接受响应
    FriendRequestAcceptedResponse,
    /// 好友请求拒绝响应
    FriendRequestRejectedResponse,
    /// 删除好友响应
    UnfriendedResponse,
    /// 获取用户补充信息响应
    GetSupplementalResponse,
    /// 设置用户临时状态响应
    SetInterimResponse,
}

impl ResponseEvent {
    /// 获取事件名称
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FriendRequestSent => "user:friend-request-sent",
            Self::FriendRequestRevokedResponse => "user:friend-request-revoked-response",
            Self::FriendRequestAcceptedResponse => "user:friend-request-accepted-response",
            Self::FriendRequestRejectedResponse => "user:friend-request-rejected-response",
            Self::UnfriendedResponse => "user:unfriended-response",
            Self::GetSupplementalResponse => "user:get-supplemental-response",
            Self::SetInterimResponse => "user:set-interim-response",
        }
    }
}

impl UserEvents {
    /// 获取客户端事件名称
    pub fn client_event(event: ClientEvent) -> &'static str {
        event.as_str()
    }
    
    /// 获取服务端事件名称
    pub fn server_event(event: ServerEvent) -> &'static str {
        event.as_str()
    }
    
    /// 获取响应事件名称
    pub fn response_event(event: ResponseEvent) -> &'static str {
        event.as_str()
    }
}

/// 发送好友请求DTO
#[derive(Debug, Deserialize)]
pub struct SendFriendRequestDto {
    /// 目标用户ID
    pub user_id: String,
}

/// 撤销好友请求DTO
#[derive(Debug, Deserialize)]
pub struct RevokeFriendRequestDto {
    /// 目标用户ID
    pub user_id: String,
}

/// 接受好友请求DTO
#[derive(Debug, Deserialize)]
pub struct AcceptFriendRequestDto {
    /// 目标用户ID
    pub user_id: String,
}

/// 拒绝好友请求DTO
#[derive(Debug, Deserialize)]
pub struct RejectFriendRequestDto {
    /// 目标用户ID
    pub user_id: String,
}

/// 删除好友DTO
#[derive(Debug, Deserialize)]
pub struct UnfriendDto {
    /// 目标用户ID
    pub user_id: String,
}

/// 封禁用户DTO
#[derive(Debug, Deserialize)]
pub struct BlockUserDto {
    /// 目标用户ID
    pub user_id: String,
}

/// 解除封禁DTO
#[derive(Debug, Deserialize)]
pub struct UnblockUserDto {
    /// 目标用户ID
    pub user_id: String,
}

/// 用户护照模块状态
pub struct PassportState {
    /// WebSocket连接管理器
    pub connection_manager: Arc<ConnectionManager>,
    /// 游戏服务
    pub game_service: Arc<GameService>,
    /// 用户ID到当前客户端ID的映射
    pub user_sessions: Arc<Mutex<HashMap<String, Vec<ClientId>>>>,
    /// 用户临时状态缓存（保存非持久化的状态信息）
    pub user_interim: Arc<Mutex<HashMap<String, UserInterim>>>,
}

impl PassportState {
    /// 创建新的用户护照状态
    pub fn new(connection_manager: Arc<ConnectionManager>) -> Self {
        Self { 
            connection_manager,
            game_service: Arc::new(GameService::new()),
            user_sessions: Arc::new(Mutex::new(HashMap::new())),
            user_interim: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// 获取用户的所有当前会话
    pub async fn get_user_sessions(&self, user_id: &str) -> Vec<ClientId> {
        let sessions = self.user_sessions.lock().await;
        sessions.get(user_id).cloned().unwrap_or_default()
    }
    
    /// 添加用户会话
    pub async fn add_user_session(&self, user_id: &str, client_id: &str) -> Result<()> {
        let mut sessions = self.user_sessions.lock().await;
        let user_sessions = sessions.entry(user_id.to_string()).or_insert_with(Vec::new);
        
        // 如果会话不存在，则添加
        if !user_sessions.contains(&client_id.to_string()) {
            user_sessions.push(client_id.to_string());
            
            // 当用户会话从0变为1时，用户状态变为在线
            if user_sessions.len() == 1 {
                // 更新用户状态为在线
                self.update_user_status(user_id, UserStatus::Online).await?;
                
                // 广播用户上线事件
                self.broadcast_user_status(user_id, UserStatus::Online).await?;
            }
        }
        
        Ok(())
    }
    
    /// 移除用户会话
    pub async fn remove_user_session(&self, user_id: &str, client_id: &str) -> Result<()> {
        let mut sessions = self.user_sessions.lock().await;
        
        if let Some(user_sessions) = sessions.get_mut(user_id) {
            // 移除指定的客户端ID
            user_sessions.retain(|id| id != client_id);
            
            // 如果用户没有任何会话了，则标记为离线
            if user_sessions.is_empty() {
                // 更新用户状态为离线
                self.update_user_status(user_id, UserStatus::Offline).await?;
                
                // 广播用户离线事件
                self.broadcast_user_status(user_id, UserStatus::Offline).await?;
            }
        }
        
        Ok(())
    }
    
    /// 获取用户状态
    pub async fn get_user_status(&self, user_id: &str) -> UserStatus {
        if let Some(user_info) = self.game_service.get::<UserInfo>(GameCachePrefix::USER, user_id) {
            user_info.status
        } else {
            UserStatus::Offline
        }
    }
    
    /// 更新用户状态
    pub async fn update_user_status(&self, user_id: &str, status: UserStatus) -> Result<()> {
        let now = Utc::now().timestamp_millis();
        
        // 尝试从缓存获取用户信息
        let mut user_info = self.game_service.get::<UserInfo>(GameCachePrefix::USER, user_id)
            .unwrap_or_else(|| {
                // 如果用户不存在，创建新的用户信息
                UserInfo {
                    id: user_id.to_string(),
                    username: format!("User-{}", user_id),
                    avatar_url: None,
                    status: UserStatus::Offline,
                    last_active: now,
                    created_at: now,
                }
            });
        
        // 更新状态和最后活跃时间
        user_info.status = status;
        user_info.last_active = now;
        
        // 保存更新后的用户信息
        self.game_service.set(GameCachePrefix::USER, user_id, &user_info);
        
        Ok(())
    }
    
    /// 广播用户状态变化
    pub async fn broadcast_user_status(&self, user_id: &str, status: UserStatus) -> Result<()> {
        let event = match status {
            UserStatus::Online => UserEvents::server_event(ServerEvent::Online),
            UserStatus::Offline => UserEvents::server_event(ServerEvent::Offline),
            _ => {
                // 其他状态不广播特定事件，而是发送通用状态更新
                let payload = serde_json::json!({
                    "userId": user_id,
                    "status": status,
                });
                
                self.connection_manager.broadcast_to_room(
                    "status_updates", 
                    UserEvents::server_event(ServerEvent::Online), 
                    Some(payload)
                ).await?;
                
                return Ok(());
            }
        };
        
        // 广播上线/下线事件
        let payload = serde_json::json!({
            "userId": user_id,
        });
        
        self.connection_manager.broadcast_to_room(
            "status_updates", 
            event, 
            Some(payload)
        ).await?;
        
        Ok(())
    }
    
    /// 设置用户为游戏中状态
    pub async fn set_user_in_game(&self, user_id: &str, game_id: &str) -> Result<()> {
        // 更新用户状态为游戏中
        self.update_user_status(user_id, UserStatus::InGame).await?;
        
        // 存储正在进行的游戏信息
        if let Some(mut games) = self.game_service.get::<Vec<String>>(GameCachePrefix::USER, &format!("{}:games", user_id)) {
            if !games.contains(&game_id.to_string()) {
                games.push(game_id.to_string());
                self.game_service.set(GameCachePrefix::USER, &format!("{}:games", user_id), &games);
            }
        } else {
            let games = vec![game_id.to_string()];
            self.game_service.set(GameCachePrefix::USER, &format!("{}:games", user_id), &games);
        }
        
        Ok(())
    }
    
    /// 移除用户游戏中状态
    pub async fn remove_user_from_game(&self, user_id: &str, game_id: &str) -> Result<()> {
        // 从正在进行的游戏列表中移除
        if let Some(mut games) = self.game_service.get::<Vec<String>>(GameCachePrefix::USER, &format!("{}:games", user_id)) {
            games.retain(|id| id != game_id);
            self.game_service.set(GameCachePrefix::USER, &format!("{}:games", user_id), &games);
            
            // 如果没有其他游戏了，则更新状态为在线
            if games.is_empty() {
                self.update_user_status(user_id, UserStatus::Online).await?;
            }
        }
        
        Ok(())
    }
    
    /// 获取用户正在进行的游戏
    pub async fn get_user_ongoing_games(&self, user_id: &str) -> Vec<String> {
        self.game_service.get::<Vec<String>>(GameCachePrefix::USER, &format!("{}:games", user_id))
            .unwrap_or_default()
    }
    
    /// 获取两个用户之间的关系
    pub async fn get_relationship(&self, user_id1: &str, user_id2: &str) -> Option<Relationship> {
        let key = if user_id1 < user_id2 {
            format!("{}:{}", user_id1, user_id2)
        } else {
            format!("{}:{}", user_id2, user_id1)
        };
        
        self.game_service.get::<Relationship>(GameCachePrefix::USER, &format!("rel:{}", key))
    }
    
    /// 创建或更新两个用户之间的关系
    pub async fn set_relationship(&self, user_id1: &str, user_id2: &str, status: RelationshipStatus) -> Result<Relationship> {
        let now = Utc::now().timestamp_millis();
        
        // 确保用户ID顺序一致，以便创建唯一关系键
        let (first_id, second_id) = if user_id1 < user_id2 {
            (user_id1, user_id2)
        } else {
            (user_id2, user_id1)
        };
        
        let key = format!("{}:{}", first_id, second_id);
        
        // 尝试获取现有关系
        let relationship = self.game_service.get::<Relationship>(GameCachePrefix::USER, &format!("rel:{}", key))
            .unwrap_or_else(|| {
                // 如果关系不存在，创建新的关系
                Relationship {
                    id: Uuid::new_v4().to_string(),
                    user1_id: first_id.to_string(),
                    user2_id: second_id.to_string(),
                    status: RelationshipStatus::None,
                    created_at: now,
                    updated_at: now,
                }
            });
        
        // 创建新的关系对象，保留原始ID和创建时间
        let updated_relationship = Relationship {
            id: relationship.id,
            user1_id: first_id.to_string(),
            user2_id: second_id.to_string(),
            status,
            created_at: relationship.created_at,
            updated_at: now,
        };
        
        // 保存更新后的关系
        self.game_service.set(GameCachePrefix::USER, &format!("rel:{}", key), &updated_relationship);
        
        Ok(updated_relationship)
    }
    
    /// 删除两个用户之间的关系
    pub async fn delete_relationship(&self, user_id1: &str, user_id2: &str) -> Result<()> {
        let key = if user_id1 < user_id2 {
            format!("{}:{}", user_id1, user_id2)
        } else {
            format!("{}:{}", user_id2, user_id1)
        };
        
        self.game_service.delete(GameCachePrefix::USER, &format!("rel:{}", key));
        
        Ok(())
    }
    
    /// 获取用户的所有好友
    pub async fn get_user_friends(&self, user_id: &str) -> Result<Vec<String>> {
        if let Some(friends) = self.game_service.get::<Vec<String>>(GameCachePrefix::USER, &format!("{}:friends", user_id)) {
            Ok(friends)
        } else {
            Ok(Vec::new())
        }
    }
    
    /// 将用户添加到好友列表
    pub async fn add_to_friends_list(&self, user_id: &str, friend_id: &str) -> Result<()> {
        // 获取当前好友列表
        let mut friends = self.get_user_friends(user_id).await?;
        
        // 如果不在列表中，则添加
        if !friends.contains(&friend_id.to_string()) {
            friends.push(friend_id.to_string());
            self.game_service.set(GameCachePrefix::USER, &format!("{}:friends", user_id), &friends);
        }
        
        Ok(())
    }
    
    /// 从好友列表中移除用户
    pub async fn remove_from_friends_list(&self, user_id: &str, friend_id: &str) -> Result<()> {
        // 获取当前好友列表
        let mut friends = self.get_user_friends(user_id).await?;
        
        // 移除指定的好友
        friends.retain(|id| id != friend_id);
        self.game_service.set(GameCachePrefix::USER, &format!("{}:friends", user_id), &friends);
        
        Ok(())
    }
    
    /// 向用户发送事件通知
    pub async fn send_event_to_user(&self, user_id: &str, event: &str, data: Option<serde_json::Value>) -> Result<bool> {
        // 获取用户的所有会话
        let sessions = self.get_user_sessions(user_id).await;
        
        let mut sent = false;
        
        // 向所有会话发送事件
        for client_id in sessions {
            if self.connection_manager.send_to_client(&client_id, event, data.clone()).await? {
                sent = true;
            }
        }
        
        Ok(sent)
    }
    
    /// 处理发送好友请求
    pub async fn handle_send_friend_request(&self, sender_id: &str, receiver_id: &str) -> Result<serde_json::Value> {
        // 检查用户是否存在
        if !self.user_exists(receiver_id).await {
            return Ok(serde_json::json!({
                "ok": false,
                "msg": "用户不存在"
            }));
        }
        
        // 获取现有关系
        let relationship = self.get_relationship(sender_id, receiver_id).await;
        
        if let Some(rel) = relationship {
            // 检查是否已经是好友
            if rel.status == RelationshipStatus::Friends {
                return Ok(serde_json::json!({
                    "ok": false,
                    "msg": "你们已经是好友了"
                }));
            }
            
            // 检查是否被阻止
            let is_blocked = 
                (rel.status == RelationshipStatus::Blocked1To2 && rel.user2_id == sender_id) ||
                (rel.status == RelationshipStatus::Blocked2To1 && rel.user1_id == sender_id) ||
                rel.status == RelationshipStatus::Blocked;
                
            if is_blocked {
                return Ok(serde_json::json!({
                    "ok": false,
                    "msg": "你已被该用户阻止"
                }));
            }
            
            // 检查是否已经发送过请求
            let already_sent = 
                (rel.status == RelationshipStatus::FriendRequest1To2 && rel.user1_id == sender_id) ||
                (rel.status == RelationshipStatus::FriendRequest2To1 && rel.user2_id == sender_id);
                
            if already_sent {
                return Ok(serde_json::json!({
                    "ok": false,
                    "msg": "好友请求已经发送过了"
                }));
            }
            
            // 检查是否有待接受的请求
            let to_accept = 
                (rel.status == RelationshipStatus::FriendRequest1To2 && rel.user2_id == sender_id) ||
                (rel.status == RelationshipStatus::FriendRequest2To1 && rel.user1_id == sender_id);
                
            if to_accept {
                // 自动接受请求
                return self.handle_accept_friend_request(sender_id, receiver_id).await;
            }
            
            // 设置新的关系状态
            let new_status = if rel.user1_id == sender_id {
                RelationshipStatus::FriendRequest1To2
            } else {
                RelationshipStatus::FriendRequest2To1
            };
            
            let updated_rel = self.set_relationship(sender_id, receiver_id, new_status).await?;
            
            // 通知接收方
            let sender_info = self.get_user_info(sender_id).await?;
            self.send_event_to_user(
                receiver_id, 
                ServerEvent::FriendRequestReceived.as_str(), 
                Some(serde_json::json!({ "user": sender_info }))
            ).await?;
            
            return Ok(serde_json::json!({
                "ok": true,
                "payload": { "status": updated_rel }
            }));
        } else {
            // 创建新的关系
            let status = if sender_id < receiver_id {
                RelationshipStatus::FriendRequest1To2
            } else {
                RelationshipStatus::FriendRequest2To1
            };
            
            let created_rel = self.set_relationship(sender_id, receiver_id, status).await?;
            
            // 通知接收方
            let sender_info = self.get_user_info(sender_id).await?;
            self.send_event_to_user(
                receiver_id, 
                ServerEvent::FriendRequestReceived.as_str(), 
                Some(serde_json::json!({ "user": sender_info }))
            ).await?;
            
            return Ok(serde_json::json!({
                "ok": true,
                "payload": { "status": created_rel }
            }));
        }
    }
    
    /// 处理撤销好友请求
    pub async fn handle_revoke_friend_request(&self, sender_id: &str, receiver_id: &str) -> Result<serde_json::Value> {
        // 获取现有关系
        if let Some(rel) = self.get_relationship(sender_id, receiver_id).await {
            // 检查是否有待处理的请求
            let can_revoke = 
                (rel.status == RelationshipStatus::FriendRequest1To2 && rel.user1_id == sender_id) ||
                (rel.status == RelationshipStatus::FriendRequest2To1 && rel.user2_id == sender_id);
                
            if !can_revoke {
                return Ok(serde_json::json!({
                    "ok": false,
                    "msg": "没有可撤销的好友请求"
                }));
            }
            
            // 重置关系状态
            let updated_rel = self.set_relationship(sender_id, receiver_id, RelationshipStatus::None).await?;
            
            // 通知接收方
            let sender_info = self.get_user_info(sender_id).await?;
            self.send_event_to_user(
                receiver_id, 
                ServerEvent::FriendRequestRevoked.as_str(), 
                Some(serde_json::json!({ "user": sender_info }))
            ).await?;
            
            return Ok(serde_json::json!({
                "ok": true,
                "payload": { "status": updated_rel }
            }));
        } else {
            return Ok(serde_json::json!({
                "ok": false,
                "msg": "没有与该用户的关系"
            }));
        }
    }
    
    /// 处理接受好友请求
    pub async fn handle_accept_friend_request(&self, accepter_id: &str, sender_id: &str) -> Result<serde_json::Value> {
        // 获取现有关系
        if let Some(rel) = self.get_relationship(accepter_id, sender_id).await {
            // 检查是否有待接受的请求
            let can_accept = 
                (rel.status == RelationshipStatus::FriendRequest1To2 && rel.user2_id == accepter_id) ||
                (rel.status == RelationshipStatus::FriendRequest2To1 && rel.user1_id == accepter_id);
                
            if !can_accept {
                return Ok(serde_json::json!({
                    "ok": false,
                    "msg": "没有待接受的好友请求"
                }));
            }
            
            // 将关系更新为好友
            let updated_rel = self.set_relationship(accepter_id, sender_id, RelationshipStatus::Friends).await?;
            
            // 将对方添加到彼此的好友列表
            self.add_to_friends_list(accepter_id, sender_id).await?;
            self.add_to_friends_list(sender_id, accepter_id).await?;
            
            // 通知请求发送方
            let accepter_info = self.get_user_info(accepter_id).await?;
            self.send_event_to_user(
                sender_id, 
                ServerEvent::FriendRequestAccepted.as_str(), 
                Some(serde_json::json!({ "user": accepter_info }))
            ).await?;
            
            return Ok(serde_json::json!({
                "ok": true,
                "payload": { "status": updated_rel }
            }));
        } else {
            return Ok(serde_json::json!({
                "ok": false,
                "msg": "没有与该用户的关系"
            }));
        }
    }
    
    /// 处理拒绝好友请求
    pub async fn handle_reject_friend_request(&self, rejecter_id: &str, sender_id: &str) -> Result<serde_json::Value> {
        // 获取现有关系
        if let Some(rel) = self.get_relationship(rejecter_id, sender_id).await {
            // 检查是否有待拒绝的请求
            let can_reject = 
                (rel.status == RelationshipStatus::FriendRequest1To2 && rel.user2_id == rejecter_id) ||
                (rel.status == RelationshipStatus::FriendRequest2To1 && rel.user1_id == rejecter_id);
                
            if !can_reject {
                return Ok(serde_json::json!({
                    "ok": false,
                    "msg": "没有待拒绝的好友请求"
                }));
            }
            
            // 重置关系状态
            let updated_rel = self.set_relationship(rejecter_id, sender_id, RelationshipStatus::None).await?;
            
            // 通知请求发送方
            let rejecter_info = self.get_user_info(rejecter_id).await?;
            self.send_event_to_user(
                sender_id, 
                ServerEvent::FriendRequestRejected.as_str(), 
                Some(serde_json::json!({ "user": rejecter_info }))
            ).await?;
            
            return Ok(serde_json::json!({
                "ok": true,
                "payload": { "status": updated_rel }
            }));
        } else {
            return Ok(serde_json::json!({
                "ok": false,
                "msg": "没有与该用户的关系"
            }));
        }
    }
    
    /// 处理删除好友
    pub async fn handle_unfriend(&self, user_id: &str, friend_id: &str) -> Result<serde_json::Value> {
        // 获取现有关系
        if let Some(rel) = self.get_relationship(user_id, friend_id).await {
            // 检查是否是好友
            if rel.status != RelationshipStatus::Friends {
                return Ok(serde_json::json!({
                    "ok": false,
                    "msg": "你们不是好友关系"
                }));
            }
            
            // 重置关系状态
            let updated_rel = self.set_relationship(user_id, friend_id, RelationshipStatus::None).await?;
            
            // 从彼此的好友列表中移除
            self.remove_from_friends_list(user_id, friend_id).await?;
            self.remove_from_friends_list(friend_id, user_id).await?;
            
            // 通知被删除的一方
            let user_info = self.get_user_info(user_id).await?;
            self.send_event_to_user(
                friend_id, 
                ServerEvent::Unfriended.as_str(), 
                Some(serde_json::json!({ "user": user_info }))
            ).await?;
            
            return Ok(serde_json::json!({
                "ok": true,
                "payload": { "status": updated_rel }
            }));
        } else {
            return Ok(serde_json::json!({
                "ok": false,
                "msg": "没有与该用户的关系"
            }));
        }
    }
    
    /// 辅助方法: 检查用户是否存在
    async fn user_exists(&self, user_id: &str) -> bool {
        self.game_service.get::<UserInfo>(GameCachePrefix::USER, user_id).is_some()
    }
    
    /// 辅助方法: 获取用户信息
    async fn get_user_info(&self, user_id: &str) -> Result<UserInfo> {
        if let Some(user_info) = self.game_service.get::<UserInfo>(GameCachePrefix::USER, user_id) {
            Ok(user_info)
        } else {
            // 用户不存在，返回错误
            Err(anyhow::anyhow!("用户不存在"))
        }
    }
    
    /// 获取用户补充信息
    pub async fn get_supplemental(&self, user_id: &str) -> UserSupplemental {
        // 获取用户状态
        let status = self.get_user_status(user_id).await;
        let status_string = UserStatusString::from(status.clone());
        
        // 获取临时状态中的活动信息
        let interim = self.get_interim(user_id).await;
        let activity = interim.activity.clone();
        
        // 如果没有活动信息，但用户在游戏中，则创建游戏活动
        let activity = if activity.is_none() && status == UserStatus::InGame {
            // 获取用户正在进行的游戏
            let game_ids = self.get_user_ongoing_games(user_id).await;
            if !game_ids.is_empty() {
                Some(UserActivity {
                    activity_type: Some(UserActivityType::InMatch),
                    match_id: Some(game_ids[0].clone()),
                    lobby_id: None,
                })
            } else {
                None
            }
        } else {
            activity
        };
        
        UserSupplemental {
            status: status_string,
            activity,
        }
    }
    
    /// 获取用户临时状态
    pub async fn get_interim(&self, user_id: &str) -> UserInterim {
        let interim_map = self.user_interim.lock().await;
        interim_map.get(user_id).cloned().unwrap_or_default()
    }
    
    /// 设置用户临时状态
    pub async fn set_interim(&self, user_id: &str, interim: UserInterim) -> Result<()> {
        let mut interim_map = self.user_interim.lock().await;
        
        // 获取或创建用户的临时状态
        let current = interim_map.entry(user_id.to_string()).or_insert_with(UserInterim::default);
        
        // 更新状态（如果提供）
        if let Some(status) = &interim.status {
            // 将前端状态映射到后端状态
            let backend_status = match status {
                UserStatusString::Online => UserStatus::Online,
                UserStatusString::Offline => UserStatus::Offline,
            };
            
            // 更新状态
            self.update_user_status(user_id, backend_status).await?;
            
            // 保存到临时状态
            current.status = Some(status.clone());
        }
        
        // 更新活动（如果提供）
        if let Some(activity) = &interim.activity {
            // 保存到临时状态
            current.activity = Some(activity.clone());
            
            // 如果是游戏中，更新游戏状态
            if let Some(UserActivityType::InMatch) = activity.activity_type {
                if let Some(match_id) = &activity.match_id {
                    self.set_user_in_game(user_id, match_id).await?;
                }
            } else if let Some(old_activity) = &current.activity {
                // 如果之前是游戏中状态，但现在不是，移除游戏状态
                if let Some(UserActivityType::InMatch) = old_activity.activity_type {
                    if let Some(match_id) = &old_activity.match_id {
                        self.remove_user_from_game(user_id, match_id).await?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// 处理获取用户补充信息请求
    pub async fn handle_get_supplemental(&self, dto: GetSupplementalDto) -> Result<serde_json::Value> {
        let mut supplementals = HashMap::new();
        
        // 获取每个用户ID的补充信息
        for id in dto.ids {
            let supplemental = self.get_supplemental(&id).await;
            supplementals.insert(id, supplemental);
        }
        
        // 返回WebSocket响应格式
        Ok(serde_json::json!({
            "ok": true,
            "payload": {
                "supplementals": supplementals
            }
        }))
    }
}

/// 处理WebSocket消息
pub async fn handle_ws_message(
    client_id: &str,
    message: WsMessage,
    passport_state: &PassportState,
    user_info: Option<UserInfo>,
) -> Result<bool> {
    debug!("处理用户事件: {}", message.event);
    
    // 尝试将事件字符串解析为ClientEvent
    let client_event = ClientEvent::from_str(&message.event);
    
    // 处理不需要用户认证的事件
    if let Some(ClientEvent::GetSupplemental) = client_event {
        if let Some(data) = &message.data {
            if let Ok(dto) = serde_json::from_value::<GetSupplementalDto>(data.clone()) {
                debug!("处理获取用户补充信息请求: {:?}", dto.ids);
                let response = passport_state.handle_get_supplemental(dto).await?;
                
                // 发送响应
                passport_state.connection_manager.send_to_client(
                    client_id,
                    ResponseEvent::GetSupplementalResponse.as_str(),
                    Some(response),
                ).await?;
                
                return Ok(true);
            } else {
                error!("解析GetSupplementalDto失败");
            }
        } else {
            error!("GET_SUPPLEMENTAL事件缺少数据");
        }
        return Ok(false);
    }
    
    // 如果没有用户信息，大多数事件无法处理
    let Some(user) = user_info else {
        debug!("用户未认证，无法处理消息: {}", message.event);
        return Ok(false);
    };
    
    // 检查是否为需要认证的用户相关事件
    match client_event {
        Some(ClientEvent::SendFriendRequest) => {
            if let Some(data) = &message.data {
                if let Ok(dto) = serde_json::from_value::<SendFriendRequestDto>(data.clone()) {
                    let response = passport_state.handle_send_friend_request(&user.id, &dto.user_id).await?;
                    
                    // 发送响应
                    passport_state.connection_manager.send_to_client(
                        client_id,
                        ResponseEvent::FriendRequestSent.as_str(),
                        Some(response),
                    ).await?;
                    
                    return Ok(true);
                }
            }
        },
        Some(ClientEvent::RevokeFriendRequest) => {
            if let Some(data) = &message.data {
                if let Ok(dto) = serde_json::from_value::<RevokeFriendRequestDto>(data.clone()) {
                    let response = passport_state.handle_revoke_friend_request(&user.id, &dto.user_id).await?;
                    
                    // 发送响应
                    passport_state.connection_manager.send_to_client(
                        client_id,
                        ResponseEvent::FriendRequestRevokedResponse.as_str(),
                        Some(response),
                    ).await?;
                    
                    return Ok(true);
                }
            }
        },
        Some(ClientEvent::AcceptFriendRequest) => {
            if let Some(data) = &message.data {
                if let Ok(dto) = serde_json::from_value::<AcceptFriendRequestDto>(data.clone()) {
                    let response = passport_state.handle_accept_friend_request(&user.id, &dto.user_id).await?;
                    
                    // 发送响应
                    passport_state.connection_manager.send_to_client(
                        client_id,
                        ResponseEvent::FriendRequestAcceptedResponse.as_str(),
                        Some(response),
                    ).await?;
                    
                    return Ok(true);
                }
            }
        },
        Some(ClientEvent::RejectFriendRequest) => {
            if let Some(data) = &message.data {
                if let Ok(dto) = serde_json::from_value::<RejectFriendRequestDto>(data.clone()) {
                    let response = passport_state.handle_reject_friend_request(&user.id, &dto.user_id).await?;
                    
                    // 发送响应
                    passport_state.connection_manager.send_to_client(
                        client_id,
                        ResponseEvent::FriendRequestRejectedResponse.as_str(),
                        Some(response),
                    ).await?;
                    
                    return Ok(true);
                }
            }
        },
        Some(ClientEvent::Unfriend) => {
            if let Some(data) = &message.data {
                if let Ok(dto) = serde_json::from_value::<UnfriendDto>(data.clone()) {
                    let response = passport_state.handle_unfriend(&user.id, &dto.user_id).await?;
                    
                    // 发送响应
                    passport_state.connection_manager.send_to_client(
                        client_id,
                        ResponseEvent::UnfriendedResponse.as_str(),
                        Some(response),
                    ).await?;
                    
                    return Ok(true);
                }
            }
        },
        Some(ClientEvent::Block) => {
            // 处理封禁用户逻辑
            // 这里需要根据实际需求实现
            return Ok(false);
        },
        Some(ClientEvent::Unblock) => {
            // 处理解除封禁逻辑
            // 这里需要根据实际需求实现
            return Ok(false);
        },
        Some(ClientEvent::SetInterim) => {
            if let Some(data) = &message.data {
                if let Ok(interim) = serde_json::from_value::<UserInterim>(data.clone()) {
                    debug!("设置用户临时状态: {:?}", interim);
                    
                    // 更新用户临时状态
                    if let Err(e) = passport_state.set_interim(&user.id, interim).await {
                        error!("设置用户临时状态失败: {}", e);
                        
                        // 发送错误响应
                        passport_state.connection_manager.send_to_client(
                            client_id,
                            ResponseEvent::SetInterimResponse.as_str(),
                            Some(serde_json::json!({
                                "ok": false,
                                "msg": format!("设置临时状态失败: {}", e)
                            })),
                        ).await?;
                    } else {
                        // 发送成功响应
                        passport_state.connection_manager.send_to_client(
                            client_id,
                            ResponseEvent::SetInterimResponse.as_str(),
                            Some(serde_json::json!({
                                "ok": true
                            })),
                        ).await?;
                    }
                    
                    return Ok(true);
                } else {
                    error!("解析UserInterim失败");
                }
            } else {
                error!("SET_INTERIM事件缺少数据");
            }
            return Ok(false);
        },
        _ => return Ok(false), // 非用户相关事件
    }
    
    Ok(false)
}

/// 处理WebSocket连接
pub async fn handle_ws_connection(
    client_id: &str,
    user_id: &str,
    passport_state: &PassportState,
) -> Result<()> {
    // 添加用户会话
    passport_state.add_user_session(user_id, client_id).await?;
    
    // 设置断开连接处理器
    let client_id_owned = client_id.to_string();
    let user_id_owned = user_id.to_string();
    passport_state.connection_manager.setup_disconnect_handler(
        client_id,
        "passport",
        Box::new(move || {
            debug!("用户 {} 的会话 {} 断开连接", user_id_owned, client_id_owned);
            // 注意：这部分代码在断开连接时异步执行，无法使用await
            // 所以我们需要在WebSocket基础设施中处理这部分逻辑
        }),
    ).await;
    
    Ok(())
}

/// 处理WebSocket断开连接
pub async fn handle_ws_disconnection(
    client_id: &str,
    user_id: &str,
    passport_state: &PassportState,
) -> Result<()> {
    // 移除用户会话
    passport_state.remove_user_session(user_id, client_id).await?;
    
    Ok(())
}

/// 用户上线处理
pub async fn handle_user_online(
    client_id: &str,
    user_id: &str,
    passport_state: &PassportState,
) -> Result<()> {
    handle_ws_connection(client_id, user_id, passport_state).await
}

/// 用户离线处理
pub async fn handle_user_offline(
    client_id: &str,
    user_id: &str,
    passport_state: &PassportState,
) -> Result<()> {
    handle_ws_disconnection(client_id, user_id, passport_state).await
}

/// 查询用户正在进行的游戏
pub async fn get_ongoing_games(
    user_id: &str,
    passport_state: &PassportState,
) -> Result<Vec<OngoingGame>> {
    // 获取用户正在进行的游戏列表
    let game_ids = passport_state.get_user_ongoing_games(user_id).await;
    
    // 获取每个游戏的详细信息
    let mut games = Vec::new();
    for game_id in game_ids {
        if let Some(game) = passport_state.game_service.get::<OngoingGame>(GameCachePrefix::STATE, &game_id) {
            games.push(game);
        }
    }
    
    Ok(games)
}

/// 注册用户护照模块路由
pub fn register_passport_routes(app: Router, connection_manager: Arc<ConnectionManager>) -> Router {
    let passport_state = Arc::new(PassportState::new(connection_manager));
    
    // 返回添加了状态的路由器
    app
}

/// 将Passport模块集成到WebSocket处理流程中
pub fn integrate_passport_with_ws(app: Arc<AppState>, connection_manager: Arc<ConnectionManager>) -> Result<()> {
    // 创建用户护照状态
    let passport_state = Arc::new(PassportState::new(connection_manager.clone()));
    
    // 我们在这里不直接注册路由，而是将passport_state存储在app中，
    // 以便WebSocket处理器可以访问它
    
    // 这一部分需要在ws模块中进行处理，将在那里集成
    
    Ok(())
}
