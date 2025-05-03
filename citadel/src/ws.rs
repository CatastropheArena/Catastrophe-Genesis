// Copyright (c), Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! WebSocket会话管理模块
//! 
//! 本模块提供WebSocket连接管理、消息广播和断线重连机制。
//! 集成了axum框架，易于与现有服务集成。

use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use anyhow::Result;
use async_trait::async_trait;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::{
    sync::{mpsc, Mutex},
    time::sleep,
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::AppState;
use crate::chat::{self, UserInfo};
use crate::passport::{self, PassportState};
use crate::gaming as match_game;

/// 客户端连接标识
pub type ClientId = String;
/// 房间标识
pub type RoomId = String;

/// 连接状态统计
#[derive(Debug, Default, Clone, Serialize)]
pub struct ConnectionStats {
    /// 当前活跃连接数
    pub active_connections: usize,
    /// 总连接次数
    pub total_connections: usize,
    /// 重连次数
    pub reconnection_count: usize,
    /// 消息发送总数
    pub messages_sent: usize,
    /// 消息接收总数
    pub messages_received: usize,
}

/// 房间定义
#[derive(Debug)]
struct Room {
    /// 房间ID
    id: RoomId,
    /// 客户端和其消息发送器映射
    clients: HashMap<ClientId, mpsc::Sender<Message>>,
}

impl Room {
    /// 创建新房间
    fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            clients: HashMap::new(),
        }
    }

    /// 添加客户端到房间
    fn join(&mut self, client_id: ClientId, sender: mpsc::Sender<Message>) {
        self.clients.insert(client_id, sender);
    }

    /// 从房间中移除客户端
    fn leave(&mut self, client_id: &str) {
        self.clients.remove(client_id);
    }

    /// 向房间内所有客户端广播消息
    fn broadcast(&self, message: Message) -> usize {
        let mut sent_count = 0;
        for (_, sender) in &self.clients {
            if sender.try_send(message.clone()).is_ok() {
                sent_count += 1;
            }
        }
        sent_count
    }

    /// 向特定客户端发送消息
    fn send_to(&self, client_id: &str, message: Message) -> Result<()> {
        if let Some(sender) = self.clients.get(client_id) {
            sender.try_send(message)?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("客户端不在房间中"))
        }
    }

    /// 获取房间内客户端数量
    fn size(&self) -> usize {
        self.clients.len()
    }
}

/// 房间管理器
#[derive(Debug, Clone)]
struct Rooms {
    /// 房间映射
    rooms: Arc<Mutex<HashMap<RoomId, Room>>>,
}

impl Default for Rooms {
    fn default() -> Self {
        Self {
            rooms: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Rooms {
    /// 获取现有房间或创建新房间
    async fn get_or_create(&self, room_id: String) -> RoomId {
        let mut rooms = self.rooms.lock().await;
        if !rooms.contains_key(&room_id) {
            rooms.insert(room_id.clone(), Room::new(&room_id));
        }
        room_id
    }

    /// 获取特定房间的客户端数量
    async fn get_room_size(&self, room_id: &str) -> usize {
        let rooms = self.rooms.lock().await;
        if let Some(room) = rooms.get(room_id) {
            room.size()
        } else {
            0
        }
    }

    /// 客户端加入房间
    async fn join(&self, room_id: &str, client_id: ClientId, sender: mpsc::Sender<Message>) {
        let mut rooms = self.rooms.lock().await;
        let room = rooms.entry(room_id.to_string()).or_insert_with(|| Room::new(room_id));
        room.join(client_id, sender);
    }

    /// 客户端离开房间
    async fn leave(&self, room_id: &str, client_id: &str) {
        let mut rooms = self.rooms.lock().await;
        if let Some(room) = rooms.get_mut(room_id) {
            room.leave(client_id);
            
            // 如果房间为空，移除房间
            if room.size() == 0 {
                rooms.remove(room_id);
            }
        }
    }

    /// 向房间广播消息
    async fn broadcast(&self, room_id: &str, message: Message) -> usize {
        let rooms = self.rooms.lock().await;
        if let Some(room) = rooms.get(room_id) {
            room.broadcast(message)
        } else {
            0
        }
    }

    /// 向房间中的特定客户端发送消息
    async fn send_to_client(&self, room_id: &str, client_id: &str, message: Message) -> Result<()> {
        let rooms = self.rooms.lock().await;
        if let Some(room) = rooms.get(room_id) {
            room.send_to(client_id, message)
        } else {
            Err(anyhow::anyhow!("房间不存在"))
        }
    }

    /// 获取所有房间信息
    async fn get_all_rooms(&self) -> HashMap<RoomId, usize> {
        let rooms = self.rooms.lock().await;
        rooms.iter()
            .map(|(id, room)| (id.clone(), room.size()))
            .collect()
    }
}

/// 连接管理器
#[derive(Clone)]
pub struct ConnectionManager {
    /// 连接计数器
    connection_counter: Arc<AtomicUsize>,
    /// 客户端->房间映射
    client_rooms: Arc<Mutex<HashMap<ClientId, HashSet<RoomId>>>>,
    /// 连接统计
    stats: Arc<Mutex<ConnectionStats>>,
    /// 房间管理
    rooms: Arc<Rooms>,
    /// 断开连接处理器
    disconnect_handlers: Arc<Mutex<HashMap<String, Box<dyn Fn() + Send + Sync + 'static>>>>,
}

impl std::fmt::Debug for ConnectionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectionManager")
            .field("connection_counter", &self.connection_counter)
            .field("clients", &format!("<{} clients>", self.client_rooms.try_lock().map(|rooms| rooms.len()).unwrap_or(0)))
            .field("stats", &self.stats)
            .field("rooms", &self.rooms)
            .field("disconnect_handlers", &format!("<{} handlers>", self.disconnect_handlers.try_lock().map(|h| h.len()).unwrap_or(0)))
            .finish()
    }
}

/// WebSocket响应格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsResponse {
    /// 操作是否成功
    pub ok: bool,
    /// 可选的消息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg: Option<String>,
    /// 可选的负载数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

/// WebSocket消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsMessage {
    /// 消息事件名称
    pub event: String,
    /// 消息数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl ConnectionManager {
    /// 创建新的连接管理器
    pub fn new() -> Self {
        Self {
            connection_counter: Arc::new(AtomicUsize::new(0)),
            client_rooms: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(ConnectionStats::default())),
            rooms: Arc::new(Rooms::default()),
            disconnect_handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 获取连接统计
    pub async fn get_stats(&self) -> ConnectionStats {
        self.stats.lock().await.clone()
    }

    /// 处理新的WebSocket连接
    pub async fn handle_socket(
        &self, 
        socket: WebSocket,
        client_id: Option<String>,
    ) -> Result<()> {
        // 生成客户端ID或使用提供的ID (用于重连)
        let client_id = client_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let connection_id = self.connection_counter.fetch_add(1, Ordering::SeqCst);
        
        info!("新WebSocket连接: id={}, connection_id={}", client_id, connection_id);
        
        // 更新统计
        {
            let mut stats = self.stats.lock().await;
            stats.active_connections += 1;
            stats.total_connections += 1;
        }

        // 创建消息通道
        let (mut sender, mut receiver) = socket.split();
        let (tx, mut rx) = mpsc::channel::<Message>(100);

        // 提前克隆client_id供任务使用
        let client_id_for_send = client_id.clone();
        let client_id_for_heartbeat = client_id.clone();

        // 管理从服务器到客户端的消息发送
        let send_task = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = sender.send(message).await {
                    error!("发送消息错误: {}", e);
                    break;
                }
            }
            debug!("发送任务结束: client_id={}", client_id_for_send);
        });

        // 设置心跳检测
        let heartbeat_tx = tx.clone();
        let heartbeat_task = tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(30)).await;
                debug!("发送心跳ping到客户端: {}", client_id_for_heartbeat);
                if heartbeat_tx.send(Message::Ping(vec![])).await.is_err() {
                    error!("心跳发送失败，客户端可能已断开连接: {}", client_id_for_heartbeat);
                    break;
                }
            }
        });
        
        // 如果有用户ID，通知Passport模块用户已连接
        let user_id = if let Some(passport_state) = GLOBAL_PASSPORT_STATE.get() {
            // 这里应该从认证系统中获取用户ID
            // 为了简单起见，我们使用客户端ID作为用户ID
            let user_id = client_id.clone();
            
            if let Err(e) = passport::handle_user_online(&client_id, &user_id, passport_state).await {
                error!("处理用户上线失败: {}", e);
            }
            
            Some(user_id)
        } else {
            None
        };

        // 处理从客户端接收的消息
        while let Some(result) = receiver.next().await {
            match result {
                Ok(message) => {
                    self.handle_message(&client_id, message, &tx).await?;
                    
                    // 更新消息计数
                    let mut stats = self.stats.lock().await;
                    stats.messages_received += 1;
                }
                Err(e) => {
                    error!("接收消息错误: {}", e);
                    break;
                }
            }
        }

        // 客户端断开连接
        info!("WebSocket连接关闭: id={}", client_id);
        
        // 如果有用户ID，通知Passport模块用户已断开连接
        if let Some(user_id) = user_id {
            if let Some(passport_state) = GLOBAL_PASSPORT_STATE.get() {
                if let Err(e) = passport::handle_user_offline(&client_id, &user_id, passport_state).await {
                    error!("处理用户离线失败: {}", e);
                }
            }
        }
        
        // 执行断开连接处理器
        self.execute_disconnect_handlers(&client_id).await;
        
        // 清理资源
        send_task.abort();
        heartbeat_task.abort();
        
        // 更新统计
        {
            let mut stats = self.stats.lock().await;
            stats.active_connections = stats.active_connections.saturating_sub(1);
        }

        // 保留客户端的房间信息以便重连
        // (不立即清除client_rooms中的记录，便于重连)

        Ok(())
    }

    /// 处理WebSocket消息
    async fn handle_message(
        &self,
        client_id: &str,
        message: Message,
        tx: &mpsc::Sender<Message>,
    ) -> Result<()> {
        match message {
            Message::Text(text) => {
                debug!("接收到文本消息: {}", text);
                
                // 尝试解析为WsMessage
                if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                    debug!("处理事件: {} 来自客户端: {}", ws_msg.event, client_id);
                    
                    // 创建一个模拟用户（真实系统中应该从认证信息获取）
                    let user_info = Some(UserInfo {
                        id: client_id.to_string(),
                        name: format!("User-{}", client_id.split('-').next().unwrap_or("unknown")),
                        avatar_url: None,
                    });
                    
                    // 创建Passport用户信息
                    let passport_user_info = user_info.clone().map(|u| passport::UserInfo {
                        id: u.id,
                        username: u.name,
                        avatar_url: u.avatar_url,
                        status: passport::UserStatus::Online,
                        last_active: chrono::Utc::now().timestamp_millis(),
                        created_at: chrono::Utc::now().timestamp_millis(),
                    });
                    
                    // 首先尝试处理用户护照相关事件
                    if ws_msg.event.starts_with("user:") {
                        // 获取全局PassportState实例
                        if let Some(passport_state) = GLOBAL_PASSPORT_STATE.get() {
                            // 特殊处理不需要身份验证的事件，如获取用户补充信息
                            if let Some(passport::ClientEvent::GetSupplemental) = passport::ClientEvent::from_str(&ws_msg.event) {
                                if let Ok(handled) = passport::handle_ws_message(
                                    client_id, 
                                    ws_msg.clone(), 
                                    passport_state, 
                                    None // 不需要用户信息
                                ).await {
                                    if handled {
                                        return Ok(());
                                    }
                                }
                            } else if let Ok(handled) = passport::handle_ws_message(
                                client_id, 
                                ws_msg.clone(), 
                                passport_state, 
                                passport_user_info
                            ).await {
                                if handled {
                                    // 消息已由用户护照模块处理
                                    return Ok(());
                                }
                            }
                        }
                    }
                    
                    // 其次尝试处理聊天相关事件
                    if ws_msg.event.starts_with("chat:") {
                        if let Ok(handled) = chat::handle_ws_message(
                            client_id, 
                            ws_msg.clone(), 
                            self, 
                            user_info
                        ).await {
                            if handled {
                                // 消息已由聊天模块处理
                                return Ok(());
                            }
                        }
                    }
                    
                    // 如果不是特定模块的事件或模块未处理，则继续处理其他事件
                    match ws_msg.event.as_str() {
                        "join_room" => {
                            if let Some(data) = ws_msg.data {
                                if let Some(room_id) = data.get("roomId").and_then(|v| v.as_str()) {
                                    self.handle_join_room(client_id, room_id, tx).await?;
                                }
                            }
                        }
                        "leave_room" => {
                            if let Some(data) = ws_msg.data {
                                if let Some(room_id) = data.get("roomId").and_then(|v| v.as_str()) {
                                    self.handle_leave_room(client_id, room_id, tx).await?;
                                }
                            }
                        }
                        "reconnect" => {
                            if let Some(data) = ws_msg.data {
                                if let Some(old_client_id) = data.get("clientId").and_then(|v| v.as_str()) {
                                    self.handle_reconnect(client_id, old_client_id, tx).await?;
                                }
                            }
                        }
                        _ => {
                            // 其他自定义事件处理
                            debug!("未处理的事件类型: {}", ws_msg.event);
                        }
                    }
                } else {
                    debug!("无法解析消息为WsMessage: {}", text);
                }
            }
            Message::Binary(data) => {
                debug!("接收到二进制消息: {} 字节", data.len());
                // 注意：我们主要处理文本消息，二进制消息仅用于特殊情况
            }
            Message::Ping(data) => {
                debug!("接收到Ping");
                let _ = tx.send(Message::Pong(data)).await;
            }
            Message::Pong(_) => {
                debug!("接收到Pong");
            }
            Message::Close(frame) => {
                info!("接收到关闭消息: {:?}", frame);
            }
        }

        Ok(())
    }

    /// 处理加入房间请求
    async fn handle_join_room(
        &self,
        client_id: &str,
        room_id: &str,
        tx: &mpsc::Sender<Message>,
    ) -> Result<()> {
        info!("客户端加入房间: client_id={}, room_id={}", client_id, room_id);
        
        // 将客户端添加到房间
        self.rooms.join(room_id, client_id.to_string(), tx.clone()).await;
        
        // 更新客户端->房间映射
        let mut client_rooms = self.client_rooms.lock().await;
        client_rooms
            .entry(client_id.to_string())
            .or_insert_with(HashSet::new)
            .insert(room_id.to_string());
        
        // 发送确认消息
        let response = WsResponse {
            ok: true,
            msg: Some(format!("已加入房间: {}", room_id)),
            payload: None,
        };
        
        let response_msg = WsMessage {
            event: "room_joined".to_string(),
            data: Some(serde_json::to_value(response)?),
        };
        
        let msg_json = serde_json::to_string(&response_msg)?;
        let _ = tx.send(Message::Text(msg_json)).await;
        
        Ok(())
    }

    /// 处理离开房间请求
    async fn handle_leave_room(
        &self,
        client_id: &str,
        room_id: &str,
        tx: &mpsc::Sender<Message>,
    ) -> Result<()> {
        info!("客户端离开房间: client_id={}, room_id={}", client_id, room_id);
        
        // 从房间移除客户端
        self.rooms.leave(room_id, client_id).await;
        
        // 更新客户端->房间映射
        let mut client_rooms = self.client_rooms.lock().await;
        if let Some(rooms) = client_rooms.get_mut(client_id) {
            rooms.remove(&room_id.to_string());
        }
        
        // 发送确认消息
        let response = WsResponse {
            ok: true,
            msg: Some(format!("已离开房间: {}", room_id)),
            payload: None,
        };
        
        let response_msg = WsMessage {
            event: "room_left".to_string(),
            data: Some(serde_json::to_value(response)?),
        };
        
        let msg_json = serde_json::to_string(&response_msg)?;
        let _ = tx.send(Message::Text(msg_json)).await;
        
        Ok(())
    }

    /// 处理重连请求
    async fn handle_reconnect(
        &self,
        client_id: &str,
        old_client_id: &str,
        tx: &mpsc::Sender<Message>,
    ) -> Result<()> {
        info!("处理重连请求: old_id={}, new_id={}", old_client_id, client_id);
        
        // 恢复房间成员资格
        let mut rejoined_rooms = Vec::new();
        
        {
            let client_rooms = self.client_rooms.lock().await;
            if let Some(rooms) = client_rooms.get(old_client_id) {
                for room_id in rooms {
                    self.rooms.join(room_id, client_id.to_string(), tx.clone()).await;
                    rejoined_rooms.push(room_id.clone());
                }
            }
        }
        
        // 更新客户端->房间映射，为新ID创建映射并迁移所有房间
        {
            let mut client_rooms = self.client_rooms.lock().await;
            if let Some(rooms) = client_rooms.remove(old_client_id) {
                client_rooms.insert(client_id.to_string(), rooms);
            }
        }
        
        // 通知客户端重新加入的房间
        if !rejoined_rooms.is_empty() {
            let response = WsResponse {
                ok: true,
                msg: Some("重连成功".to_string()),
                payload: Some(serde_json::json!({
                    "rejoined_rooms": rejoined_rooms
                })),
            };
            
            let response_msg = WsMessage {
                event: "reconnect_success".to_string(),
                data: Some(serde_json::to_value(response)?),
            };
            
            let msg_json = serde_json::to_string(&response_msg)?;
            let _ = tx.send(Message::Text(msg_json)).await;
            
            // 更新统计
            let mut stats = self.stats.lock().await;
            stats.reconnection_count += 1;
        } else {
            // 没有找到以前的房间
            let response = WsResponse {
                ok: true,
                msg: Some("重连成功，但没有找到以前的房间".to_string()),
                payload: None,
            };
            
            let response_msg = WsMessage {
                event: "reconnect_success".to_string(),
                data: Some(serde_json::to_value(response)?),
            };
            
            let msg_json = serde_json::to_string(&response_msg)?;
            let _ = tx.send(Message::Text(msg_json)).await;
        }
        
        Ok(())
    }

    /// 设置断开连接处理器
    pub async fn setup_disconnect_handler<F>(
        &self,
        client_id: &str,
        context: &str,
        handler: F
    ) where
        F: Fn() + Send + Sync + 'static,
    {
        let handler_id = format!("{}:{}", client_id, context);
        info!("设置断开连接处理器: {}", handler_id);
        
        // 清理同一上下文的旧处理器
        self.clean_old_handlers(client_id, context).await;
        
        // 注册新处理器
        let mut handlers = self.disconnect_handlers.lock().await;
        handlers.insert(handler_id, Box::new(handler));
        
        info!("当前处理器总数: {}", handlers.len());
    }

    /// 清理旧处理器
    async fn clean_old_handlers(&self, client_id: &str, context: &str) {
        let handler_id = format!("{}:{}", client_id, context);
        let mut handlers = self.disconnect_handlers.lock().await;
        
        if handlers.remove(&handler_id).is_some() {
            info!("移除旧处理器: {}", handler_id);
        }
    }

    /// 执行断开连接处理器
    async fn execute_disconnect_handlers(&self, client_id: &str) {
        let prefix = format!("{}:", client_id);
        
        // 直接在锁内收集并执行处理器，避免借用错误
        let mut keys_to_remove = Vec::new();
        
        // 找到匹配的处理器并执行
        {
            let handlers = self.disconnect_handlers.lock().await;
            info!("找到 {} 个断开连接处理器", handlers.len());
            
            for (key, handler) in handlers.iter() {
                if key.starts_with(&prefix) {
                    info!("执行断开连接处理器: {}", key);
                    handler();
                    keys_to_remove.push(key.clone());
                }
            }
        }
        
        // 移除已执行的处理器
        if !keys_to_remove.is_empty() {
            let mut handlers = self.disconnect_handlers.lock().await;
            for key in keys_to_remove {
                handlers.remove(&key);
                info!("移除已执行的处理器: {}", key);
            }
        }
    }

    /// 向特定房间广播消息
    pub async fn broadcast_to_room(&self, room_id: &str, event: &str, data: Option<serde_json::Value>) -> Result<usize> {
        let ws_message = WsMessage {
            event: event.to_string(),
            data,
        };
        
        let message_json = serde_json::to_string(&ws_message)?;
        let axum_message = Message::Text(message_json);
        
        let count = self.rooms.broadcast(room_id, axum_message).await;
        if count > 0 {
            // 更新消息计数
            let mut stats = self.stats.lock().await;
            stats.messages_sent += count;
            
            info!("向房间 {} 广播事件 {}, 接收客户端数: {}", room_id, event, count);
        } else {
            warn!("尝试向不存在或空的房间广播: {}", room_id);
        }
        
        Ok(count)
    }

    /// 向特定客户端发送消息
    pub async fn send_to_client(&self, client_id: &str, event: &str, data: Option<serde_json::Value>) -> Result<bool> {
        let ws_message = WsMessage {
            event: event.to_string(),
            data,
        };
        
        let message_json = serde_json::to_string(&ws_message)?;
        let axum_message = Message::Text(message_json);
        
        // 遍历客户端所在的所有房间，寻找客户端
        let client_rooms = self.client_rooms.lock().await;
        if let Some(rooms) = client_rooms.get(client_id) {
            for room_id in rooms {
                if self.rooms.send_to_client(room_id, client_id, axum_message.clone()).await.is_ok() {
                    // 更新消息计数
                    let mut stats = self.stats.lock().await;
                    stats.messages_sent += 1;
                    
                    info!("向客户端 {} 发送事件 {}", client_id, event);
                    return Ok(true);
                }
            }
        }
        
        warn!("客户端 {} 未找到或发送失败", client_id);
        Ok(false)
    }
    
    /// 获取特定房间内的客户端数量
    pub async fn get_room_size(&self, room_id: &str) -> usize {
        self.rooms.get_room_size(room_id).await
    }
    
    /// 获取所有房间及其客户端数量
    pub async fn get_rooms_info(&self) -> HashMap<String, usize> {
        self.rooms.get_all_rooms().await
    }
    
    /// 检查客户端是否在特定房间中
    pub async fn is_client_in_room(&self, client_id: &str, room_id: &str) -> bool {
        let client_rooms = self.client_rooms.lock().await;
        if let Some(rooms) = client_rooms.get(client_id) {
            rooms.contains(&room_id.to_string())
        } else {
            false
        }
    }
}

// 用于存储全局PassportState实例的静态变量
static GLOBAL_PASSPORT_STATE: once_cell::sync::OnceCell<Arc<PassportState>> = once_cell::sync::OnceCell::new();

/// 注册WebSocket路由
pub fn register_ws_routes(app: Router) -> Router {
    // 创建连接管理器
    let connection_manager = Arc::new(ConnectionManager::new());
    
    // 创建用户护照状态
    let passport_state = Arc::new(PassportState::new(connection_manager.clone()));
    
    // 设置全局PassportState实例
    let _ = GLOBAL_PASSPORT_STATE.set(passport_state);
    
    // 创建游戏服务
    let game_service = Arc::new(crate::game::GameService::new());
    
    // 初始化匹配服务
    let match_service = match_game::init_match_service(game_service, connection_manager.clone());
    
    // 添加聊天模块路由
    let app = chat::register_chat_routes(app, connection_manager.clone());
    
    // 添加用户护照模块路由
    let app = passport::register_passport_routes(app, connection_manager.clone());
    
    // 创建WebSocket处理闭包
    let connection_manager_for_handler = connection_manager.clone();
    let handle_ws = move |ws: WebSocketUpgrade| {
        let connection_manager = connection_manager_for_handler.clone();
        async move {
            info!("WebSocket连接请求");
            // 升级连接
            ws.on_upgrade(move |socket| async move {
                // 处理WebSocket连接
                if let Err(e) = connection_manager.handle_socket(socket, None).await {
                    error!("WebSocket处理错误: {}", e);
                }
            })
        }
    };
    
    // 为重连和状态处理克隆connection_manager
    let connection_manager_for_reconnect = connection_manager.clone();
    let connection_manager_for_stats = connection_manager.clone();
    
    // 创建WebSocket重连处理闭包
    let handle_ws_reconnect = move |ws: WebSocketUpgrade, params: axum::extract::Query<HashMap<String, String>>| {
        let connection_manager = connection_manager_for_reconnect.clone();
        async move {
            let client_id = params.get("client_id").cloned();
            
            info!("WebSocket重连请求, client_id: {:?}", client_id);
            
            // 升级连接
            ws.on_upgrade(move |socket| async move {
                // 处理WebSocket连接（使用提供的客户端ID进行重连）
                if let Err(e) = connection_manager.handle_socket(socket, client_id).await {
                    error!("WebSocket重连处理错误: {}", e);
                }
            })
        }
    };
    
    // 创建WebSocket状态处理闭包
    let handle_ws_stats = move || {
        let connection_manager = connection_manager_for_stats.clone();
        async move {
            let stats = connection_manager.get_stats().await;
            let rooms_info = connection_manager.get_rooms_info().await;
            
            let response = serde_json::json!({
                "stats": stats,
                "rooms": rooms_info
            });
            
            axum::Json(response)
        }
    };
    
    // 添加WebSocket路由
    app.route("/ws", get(handle_ws))
       .route("/ws/reconnect", get(handle_ws_reconnect))
       .route("/ws/stats", get(handle_ws_stats))
}
