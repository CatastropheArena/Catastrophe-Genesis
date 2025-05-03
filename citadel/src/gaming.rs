// Copyright (c), Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::game::{GameCache, GameCachePrefix, GameService};
use crate::ws::{ClientId, ConnectionManager, RoomId, WsMessage, WsResponse};
use crate::tool::elo::{self, MatchOutcome}; // 导入 ELO 评分系统
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use rand::seq::SliceRandom;
use rand::thread_rng;

/// 匹配类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MatchType {
    /// 公开游戏
    Public,
    /// 私人游戏
    Private,
}

/// 游戏状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MatchState {
    /// 等待中
    Waiting,
    /// 进行中
    InProgress,
    /// 已完成
    Completed,
}

/// 失败原因枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DefeatReason {
    /// 爆炸
    Explosion,
    /// 超时
    Timeout,
    /// 退出
    Leave,
}

/// 队列常量
pub struct Queue {
    pub name: &'static str,
    pub delay: u64,
}

/// 队列相关常量
pub mod queue_constants {
    use super::Queue;
    
    /// 卡牌动作队列
    pub const CARD_ACTION: Queue = Queue {
        name: "card-action",
        delay: 5000, // 5秒
    };
    
    /// 不活跃队列
    pub mod inactivity {
        use super::Queue;
        
        pub const NAME: &str = "inactivity";
        
        /// 普通延迟
        pub const COMMON: u64 = 30000; // 30秒
        
        /// 对于爆炸卡的延迟
        pub const EXPLOSION: u64 = 15000; // 15秒
    }
}

/// 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub name: String,
    pub rating: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

/// 卡牌类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CardType {
    /// 爆炸猫
    ExplodingKitten,
    /// 拆除
    Defuse,
    /// 跳过
    Skip,
    /// 偷看未来
    SeeTheFuture,
    /// 打乱
    Shuffle,
    /// 攻击
    Attack,
    /// 抢夺
    Favor,
    /// 猫咪卡
    Cat,
    /// 烦人卡
    Nope,
    /// 内爆猫
    ImplodingKitten,
    /// 替换卡
    AlterTheFuture,
    /// 分享未来
    ShareTheFuture,
    /// 掩埋
    BuryCard,
    /// 加速爆炸
    SpeedUpExplosion,
}

/// 卡牌信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: String,
    #[serde(rename = "type")]
    pub card_type: CardType,
    pub variant: Option<String>,
}

/// 游戏玩家
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchPlayer {
    pub user: UserInfo,
    pub hand: Vec<Card>,
    pub is_active: bool,
    pub is_winner: bool,
    pub is_turn: bool,
}

/// 卡牌动作类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CardActionType {
    /// 普通出牌
    Play,
    /// 抽卡
    Draw,
    /// 使用烦人卡
    Nope,
    /// 使用拆除卡
    Defuse,
}

/// 卡牌动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardAction {
    /// 动作类型
    #[serde(rename = "type")]
    pub action_type: CardActionType,
    /// 玩家ID
    pub user_id: String,
    /// 卡牌ID（可选，对于Draw动作可能为空）
    pub card_id: Option<String>,
    /// 卡牌类型（可选）
    pub card_type: Option<CardType>,
    /// 是否被取消
    pub is_canceled: bool,
    /// 创建时间
    pub created_at: u64,
}

/// 游戏房间数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchData {
    pub id: String,
    #[serde(rename = "type")]
    pub match_type: MatchType,
    pub state: MatchState,
    pub players: Vec<MatchPlayer>,
    pub out: Vec<MatchPlayer>,
    pub spectators: Vec<UserInfo>,
    pub deck: Vec<Card>,
    pub discard_pile: Vec<Card>,
    pub turn_index: usize,
    pub created_at: u64,
    pub updated_at: u64,
    pub draw_count: usize,
    pub skip_votes: HashMap<String, bool>,
    /// 动作历史记录
    #[serde(default)]
    pub action_history: Vec<CardAction>,
    /// 当前连锁状态（如果非空，表示有连锁效果在等待反应）
    #[serde(default)]
    pub chain_state: Option<CardAction>,
    /// 连锁响应等待时间（毫秒）
    #[serde(default = "default_chain_wait_time")]
    pub chain_wait_time: u64,
}

/// 卡牌动作队列载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardActionQueuePayload {
    pub match_id: String,
    pub user_id: String,
    pub card_id: String,
}

/// 不活跃队列载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InactivityQueuePayload {
    pub match_id: String,
    pub user_id: String,
}

/// 游戏事件
pub mod events {
    /// 大厅事件
    pub mod lobby {
        pub const JOIN: &str = "lobby:join";
        pub const LEAVE: &str = "lobby:leave";
        pub const UPDATE: &str = "lobby:update";
    }
    
    /// 匹配事件
    pub mod match_events {
        pub const JOIN: &str = "match:join";
        pub const LEAVE: &str = "match:leave";
        pub const START: &str = "match:start";
        pub const END: &str = "match:end";
        pub const DRAW_CARD: &str = "match:draw_card";
        pub const PLAY_CARD: &str = "match:play_card";
        pub const TURN_CHANGE: &str = "match:turn_change";
        pub const DEFUSE: &str = "match:defuse";
        pub const INSERT_EXPLODING_KITTEN: &str = "match:insert_exploding_kitten";
        pub const DEFEAT: &str = "match:defeat";
        pub const VICTORY: &str = "match:victory";
        pub const ALTER_FUTURE: &str = "match:alter_future";
        pub const SPEED_UP_EXPLOSION: &str = "match:speed_up_explosion";
        pub const BURY_CARD: &str = "match:bury_card";
        pub const SHARE_FUTURE: &str = "match:share_future";
        pub const INSERT_IMPLODING_KITTEN: &str = "match:insert_imploding_kitten";
        pub const JOIN_SPECTATORS: &str = "match:join_spectators";
        pub const LEAVE_SPECTATORS: &str = "match:leave_spectators";
    }
}

/// 游戏匹配服务
pub struct MatchService {
    /// 游戏服务，处理缓存
    game_service: Arc<GameService>,
    /// WebSocket连接管理器
    connection_manager: Arc<ConnectionManager>,
    /// 活跃的游戏匹配
    active_matches: Arc<RwLock<HashMap<String, String>>>,
    /// 游戏队列
    queue: Arc<RwLock<Vec<UserInfo>>>,
}

impl MatchService {
    /// 创建新的游戏匹配服务
    pub fn new(
        game_service: Arc<GameService>,
        connection_manager: Arc<ConnectionManager>,
    ) -> Self {
        Self {
            game_service,
            connection_manager,
            active_matches: Arc::new(RwLock::new(HashMap::new())),
            queue: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// 获取游戏
    pub async fn get_match(&self, match_id: &str) -> Option<MatchData> {
        self.game_service.get(GameCachePrefix::MATCH, match_id)
    }
    
    /// 保存游戏
    pub async fn save_match(&self, match_data: &MatchData) -> bool {
        let result = self.game_service.set(GameCachePrefix::MATCH, &match_data.id, match_data);
        
        // 更新活跃游戏列表
        if result {
            let mut active_matches = self.active_matches.write().await;
            active_matches.insert(match_data.id.clone(), match_data.id.clone());
        }
        
        result
    }
    
    /// 删除游戏
    pub async fn delete_match(&self, match_id: &str) -> bool {
        let result = self.game_service.delete(GameCachePrefix::MATCH, match_id);
        
        // 从活跃游戏列表中移除
        if result {
            let mut active_matches = self.active_matches.write().await;
            active_matches.remove(match_id);
        }
        
        result
    }
    
    /// 创建新游戏
    pub async fn create_match(&self, match_type: MatchType, players: Vec<UserInfo>) -> Result<MatchData> {
        let match_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp_millis() as u64;
        
        // 创建游戏玩家
        let match_players = players.iter().map(|user| {
            MatchPlayer {
                user: user.clone(),
                hand: Vec::new(),
                is_active: true,
                is_winner: false,
                is_turn: false,
            }
        }).collect::<Vec<_>>();
        
        // 创建游戏数据
        let match_data = MatchData {
            id: match_id.clone(),
            match_type,
            state: MatchState::Waiting,
            players: match_players,
            out: Vec::new(),
            spectators: Vec::new(),
            deck: Vec::new(), // 初始化空牌组，实际游戏开始前会生成
            discard_pile: Vec::new(),
            turn_index: 0,
            created_at: now,
            updated_at: now,
            draw_count: 0,
            skip_votes: HashMap::new(),
            action_history: Vec::new(),
            chain_state: None,
            chain_wait_time: default_chain_wait_time(),
        };
        
        // 保存游戏数据
        if !self.save_match(&match_data).await {
            return Err(anyhow::anyhow!("保存游戏数据失败"));
        }
        
        Ok(match_data)
    }
    
    /// 加入游戏
    pub async fn join_match(&self, match_id: &str, user_id: &str, client_id: &str) -> Result<()> {
        // 获取游戏数据
        let match_data = self.get_match(match_id).await
            .ok_or_else(|| anyhow::anyhow!("游戏不存在"))?;
        
        // 检查游戏状态
        if match_data.state != MatchState::Waiting {
            return Err(anyhow::anyhow!("游戏已经开始或结束，无法加入"));
        }
        
        // 加入WebSocket房间 - 使用手动实现加入房间
        self.connection_manager.broadcast_to_room(match_id, "system:join", Some(serde_json::json!({
            "client_id": client_id
        }))).await?;
        
        // 广播加入事件
        let response = WsResponse {
            ok: true,
            msg: Some(format!("玩家 {} 加入了游戏", user_id)),
            payload: Some(serde_json::to_value(&match_data)?),
        };
        
        self.connection_manager.broadcast_to_room(
            match_id,
            events::match_events::JOIN,
            Some(serde_json::to_value(response)?),
        ).await?;
        
        Ok(())
    }
    
    /// 离开游戏
    pub async fn leave_match(&self, match_id: &str, user_id: &str, client_id: &str) -> Result<()> {
        // 获取游戏数据
        let mut match_data = self.get_match(match_id).await
            .ok_or_else(|| anyhow::anyhow!("游戏不存在"))?;
        
        // 如果游戏已经开始，将玩家标记为离开
        if match_data.state == MatchState::InProgress {
            // 查找玩家
            let player_index = match_data.players.iter().position(|p| p.user.id == user_id);
            
            if let Some(index) = player_index {
                // 将玩家移到出局列表
                let mut player = match_data.players.remove(index);
                player.is_active = false;
                match_data.out.push(player);
                
                // 更新游戏数据
                match_data.updated_at = chrono::Utc::now().timestamp_millis() as u64;
                self.save_match(&match_data).await;
                
                // 检查游戏是否结束
                if match_data.players.len() <= 1 {
                    {
                        // 使用代码块来限制可变引用的作用域
                        let last_player = match_data.players.first_mut().unwrap();
                        // 标记为胜利者
                        last_player.is_winner = true;
                        
                        // 更新游戏状态
                        match_data.state = MatchState::Completed;
                        match_data.updated_at = chrono::Utc::now().timestamp_millis() as u64;
                    } // last_player的可变引用在这里结束
                    
                    // 克隆数据供后续使用
                    let match_data_clone = match_data.clone();
                    self.save_match(&match_data).await;
                    
                    // 获取胜利者的用户ID用于响应
                    let winner_id = match_data.players.first().unwrap().user.id.clone();
                    
                    // 广播胜利事件
                    let victory_response = WsResponse {
                        ok: true,
                        msg: Some(format!("玩家 {} 获胜", winner_id)),
                        payload: Some(serde_json::json!({
                            "userId": winner_id
                        })),
                    };
                    
                    self.connection_manager.broadcast_to_room(
                        match_id,
                        events::match_events::VICTORY,
                        Some(serde_json::to_value(victory_response)?),
                    ).await?;
                    
                    // 广播游戏结束事件
                    let end_response = WsResponse {
                        ok: true,
                        msg: Some("游戏结束".to_string()),
                        payload: Some(serde_json::to_value(&match_data_clone)?),
                    };
                    
                    self.connection_manager.broadcast_to_room(
                        match_id,
                        events::match_events::END,
                        Some(serde_json::to_value(end_response)?),
                    ).await?;
                    
                    // 更新玩家评分
                    if let Err(e) = self.update_player_ratings(match_id).await {
                        error!("更新玩家评分失败: {}", e);
                    }
                    
                    return Ok(());
                }
            }
        } else if match_data.state == MatchState::Waiting {
            // 如果游戏还在等待中，直接移除玩家
            match_data.players.retain(|p| p.user.id != user_id);
            
            // 如果没有玩家了，删除游戏
            if match_data.players.is_empty() {
                self.delete_match(match_id).await;
            } else {
                // 更新游戏数据
                match_data.updated_at = chrono::Utc::now().timestamp_millis() as u64;
                self.save_match(&match_data).await;
            }
        }
        
        // 离开WebSocket房间 - 使用手动实现离开房间
        self.connection_manager.broadcast_to_room(match_id, "system:leave", Some(serde_json::json!({
            "client_id": client_id
        }))).await?;
        
        // 广播离开事件
        let response = WsResponse {
            ok: true,
            msg: Some(format!("玩家 {} 离开了游戏", user_id)),
            payload: Some(serde_json::to_value(&match_data)?),
        };
        
        self.connection_manager.broadcast_to_room(
            match_id,
            events::match_events::LEAVE,
            Some(serde_json::to_value(response)?),
        ).await?;
        
        Ok(())
    }
    
    /// 开始匹配队列处理
    pub async fn start_matchmaking(&self) {
        let match_service = self.clone();
        
        tokio::spawn(async move {
            loop {
                // 检查队列中的玩家数量，如果达到设定人数则创建游戏
                match_service.process_queue().await;
                
                // 每5秒检查一次
                sleep(Duration::from_secs(5)).await;
            }
        });
    }
    
    /// 处理匹配队列
    async fn process_queue(&self) {
        // 获取队列中的玩家
        let players = {
            let queue = self.queue.read().await;
            if queue.len() < 2 {
                return; // 至少需要2名玩家才能开始游戏
            }
            
            // 复制前4名玩家（或者全部，如果少于4名）
            let player_count = queue.len().min(4);
            queue[0..player_count].to_vec()
        };
        
        if players.len() >= 2 {
            // 创建新游戏
            match self.create_match(MatchType::Public, players.clone()).await {
                Ok(match_data) => {
                    // 从队列中移除这些玩家
                    {
                        let mut queue = self.queue.write().await;
                        for player in &players {
                            if let Some(pos) = queue.iter().position(|p| p.id == player.id) {
                                queue.remove(pos);
                            }
                        }
                    }
                    
                    // 通知所有玩家游戏创建成功
                    for player in &players {
                        // 在实际应用中，这里需要查找玩家的WebSocket连接并发送消息
                        // 这里使用连接管理器向玩家发送消息
                        
                        // 模拟向玩家发送消息，实际应用需要获取玩家的连接ID
                        let response = WsResponse {
                            ok: true,
                            msg: Some(format!("游戏已创建，ID: {}", match_data.id)),
                            payload: Some(serde_json::to_value(&match_data).unwrap_or_default()),
                        };
                        
                        // 这里需要获取玩家的连接ID，这个示例中我们使用玩家ID作为连接ID
                        if let Err(e) = self.connection_manager.send_to_client(
                            &player.id,
                            events::match_events::START,
                            Some(serde_json::to_value(response).unwrap_or_default()),
                        ).await {
                            error!("向玩家 {} 发送游戏创建消息失败: {}", player.id, e);
                        }
                    }
                    
                    info!("已创建新游戏: {}", match_data.id);
                }
                Err(e) => {
                    error!("创建游戏失败: {}", e);
                }
            }
        }
    }
    
    /// 加入匹配队列
    pub async fn join_queue(&self, user: UserInfo) -> Result<()> {
        // 检查玩家是否已在队列中
        {
            let queue = self.queue.read().await;
            if queue.iter().any(|p| p.id == user.id) {
                return Err(anyhow::anyhow!("玩家已在队列中"));
            }
        }
        
        // 将玩家添加到队列
        {
            let mut queue = self.queue.write().await;
            queue.push(user.clone());
        }
        
        info!("玩家 {} 加入匹配队列", user.id);
        Ok(())
    }
    
    /// 离开匹配队列
    pub async fn leave_queue(&self, user_id: &str) -> Result<()> {
        let mut queue = self.queue.write().await;
        let original_len = queue.len();
        
        queue.retain(|p| p.id != user_id);
        
        if queue.len() < original_len {
            info!("玩家 {} 离开匹配队列", user_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("玩家不在队列中"))
        }
    }
    
    /// 获取队列状态
    pub async fn get_queue_status(&self, user_id: &str) -> Option<u64> {
        let queue = self.queue.read().await;
        
        // 查找玩家在队列中的位置
        for (i, player) in queue.iter().enumerate() {
            if player.id == user_id {
                // 实际应用中，可能需要返回更多信息，如等待时间、队列位置等
                return Some(chrono::Utc::now().timestamp_millis() as u64);
            }
        }
        
        None
    }
    
    /// 开始游戏
    pub async fn start_game(&self, match_id: &str) -> Result<()> {
        // 获取游戏数据
        let mut match_data = self.get_match(match_id).await
            .ok_or_else(|| anyhow::anyhow!("游戏不存在"))?;
        
        // 检查游戏状态
        if match_data.state != MatchState::Waiting {
            return Err(anyhow::anyhow!("游戏已经开始或结束"));
        }
        
        // 检查玩家数量
        if match_data.players.len() < 2 {
            return Err(anyhow::anyhow!("玩家数量不足，无法开始游戏"));
        }
        
        // 生成牌组
        match_data.deck = generate_deck(match_data.players.len());
        
        // 发牌
        distribute_cards(&mut match_data);
        
        // 更新游戏状态
        match_data.state = MatchState::InProgress;
        match_data.updated_at = chrono::Utc::now().timestamp_millis() as u64;
        
        // 设置第一个玩家为当前回合
        match_data.turn_index = 0;
        match_data.players[0].is_turn = true;
        
        // 保存游戏数据
        if !self.save_match(&match_data).await {
            return Err(anyhow::anyhow!("保存游戏数据失败"));
        }
        
        // 广播游戏开始事件
        let response = WsResponse {
            ok: true,
            msg: Some("游戏开始".to_string()),
            payload: Some(serde_json::to_value(&match_data)?),
        };
        
        self.connection_manager.broadcast_to_room(
            match_id,
            events::match_events::START,
            Some(serde_json::to_value(response)?),
        ).await?;
        
        Ok(())
    }
    
    /// 抽卡
    pub async fn draw_card(&self, match_id: &str, user_id: &str) -> Result<Option<Card>> {
        // 获取游戏数据
        let mut match_data = self.get_match(match_id).await
            .ok_or_else(|| anyhow::anyhow!("游戏不存在"))?;
        
        // 检查游戏状态
        if match_data.state != MatchState::InProgress {
            return Err(anyhow::anyhow!("游戏未开始或已结束"));
        }
        
        // 查找玩家
        let player_index = match_data.players.iter().position(|p| p.user.id == user_id)
            .ok_or_else(|| anyhow::anyhow!("玩家不在游戏中"))?;
        
        // 检查是否是玩家的回合
        if !match_data.players[player_index].is_turn {
            return Err(anyhow::anyhow!("不是该玩家的回合"));
        }
        
        // 检查牌堆是否为空
        if match_data.deck.is_empty() {
            return Err(anyhow::anyhow!("牌堆已空"));
        }
        
        // 抽卡
        let card = match_data.deck.pop().unwrap();
        
        // 更新抽卡计数
        match_data.draw_count += 1;
        
        // 广播抽卡事件（不含卡牌信息，只通知有人抽卡）
        let draw_response = WsResponse {
            ok: true,
            msg: Some(format!("玩家 {} 抽了一张牌", user_id)),
            payload: Some(serde_json::json!({
                "userId": user_id,
                "deckCount": match_data.deck.len()
            })),
        };
        
        self.connection_manager.broadcast_to_room(
            match_id,
            events::match_events::DRAW_CARD,
            Some(serde_json::to_value(draw_response)?),
        ).await?;
        
        // 处理爆炸猫
        if matches!(card.card_type, CardType::ExplodingKitten) {
            // 给玩家私下消息通知抽到爆炸猫
            let explode_response = WsResponse {
                ok: true,
                msg: Some("你抽到了爆炸猫！".to_string()),
                payload: Some(serde_json::json!({
                    "card": card
                })),
            };
            
            // 检查玩家是否有拆除卡
            let has_defuse = match_data.players[player_index].hand.iter()
                .any(|c| matches!(c.card_type, CardType::Defuse));
            
            // 私下通知玩家
            self.connection_manager.send_to_client(
                &user_id, 
                events::match_events::DRAW_CARD,
                Some(serde_json::to_value(explode_response)?),
            ).await?;
            
            if has_defuse {
                // 玩家有拆除卡，进入拆弹状态
                // 在实际游戏中，需要等待玩家操作
                // 此示例简化为自动使用拆除卡
                
                // 移除一张拆除卡
                let defuse_index = match_data.players[player_index].hand.iter()
                    .position(|c| matches!(c.card_type, CardType::Defuse))
                    .unwrap();
                
                let defuse_card = match_data.players[player_index].hand.remove(defuse_index);
                
                // 将拆除卡放入弃牌堆
                match_data.discard_pile.push(defuse_card);
                
                // 将爆炸猫放回牌堆
                match_data.deck.push(card.clone());
                
                // 广播拆弹成功事件
                let defuse_response = WsResponse {
                    ok: true,
                    msg: Some(format!("玩家 {} 使用拆除卡解除了爆炸猫", user_id)),
                    payload: Some(serde_json::json!({
                        "userId": user_id
                    })),
                };
                
                self.connection_manager.broadcast_to_room(
                    match_id,
                    events::match_events::DEFUSE,
                    Some(serde_json::to_value(defuse_response)?),
                ).await?;
                
                // 进入下一回合
                self.change_turn(match_id).await?;
                
                // 保存游戏数据
                match_data.updated_at = chrono::Utc::now().timestamp_millis() as u64;
                self.save_match(&match_data).await;
                
                // 返回抽到的牌
                return Ok(Some(card));
            } else {
                // 玩家没有拆除卡，淘汰
                
                // 将玩家移到出局列表
                let mut player = match_data.players.remove(player_index);
                player.is_active = false;
                match_data.out.push(player);
                
                // 广播淘汰事件
                let defeat_response = WsResponse {
                    ok: true,
                    msg: Some(format!("玩家 {} 被爆炸猫炸死了", user_id)),
                    payload: Some(serde_json::json!({
                        "userId": user_id,
                        "reason": "explosion"
                    })),
                };
                
                self.connection_manager.broadcast_to_room(
                    match_id,
                    events::match_events::DEFEAT,
                    Some(serde_json::to_value(defeat_response)?),
                ).await?;
                
                // 检查游戏是否结束
                if match_data.players.len() <= 1 {
                    // 使用新的游戏结束处理方法
                    self.handle_game_end(match_id).await?;
                } else {
                    // 游戏继续，切换到下一玩家
                    self.change_turn(match_id).await?;
                    
                    // 保存游戏数据
                    match_data.updated_at = chrono::Utc::now().timestamp_millis() as u64;
                    self.save_match(&match_data).await;
                }
                
                return Ok(Some(card));
            }
        } else {
            // 普通卡牌，加入玩家手牌
            match_data.players[player_index].hand.push(card.clone());
            
            // 私下通知玩家抽到的牌
            let card_response = WsResponse {
                ok: true,
                msg: Some(format!("你抽到了 {:?}", card.card_type)),
                payload: Some(serde_json::json!({
                    "card": card
                })),
            };
            
            self.connection_manager.send_to_client(
                &user_id,
                events::match_events::DRAW_CARD,
                Some(serde_json::to_value(card_response)?),
            ).await?;
            
            // 进入下一回合
            self.change_turn(match_id).await?;
            
            // 保存游戏数据
            match_data.updated_at = chrono::Utc::now().timestamp_millis() as u64;
            self.save_match(&match_data).await;
            
            return Ok(Some(card));
        }
    }
    
    /// 出牌
    pub async fn play_card(&self, match_id: &str, user_id: &str, card_id: &str) -> Result<()> {
        // 获取游戏数据
        let mut match_data = self.get_match(match_id).await
            .ok_or_else(|| anyhow::anyhow!("游戏不存在"))?;
        
        // 检查游戏状态
        if match_data.state != MatchState::InProgress {
            return Err(anyhow::anyhow!("游戏未开始或已结束"));
        }
        
        // 检查是否有连锁状态正在处理
        if match_data.chain_state.is_some() {
            return Err(anyhow::anyhow!("有连锁效果正在处理中，请稍后再试"));
        }
        
        // 查找玩家
        let player_index = match_data.players.iter().position(|p| p.user.id == user_id)
            .ok_or_else(|| anyhow::anyhow!("玩家不在游戏中"))?;
        
        // 检查是否是玩家的回合（烦人卡例外，任何人可以打出）
        let is_nope_card = match_data.players[player_index].hand.iter()
            .any(|c| c.id == card_id && matches!(c.card_type, CardType::Nope));
            
        if !match_data.players[player_index].is_turn && !is_nope_card {
            return Err(anyhow::anyhow!("不是该玩家的回合"));
        }
        
        // 处理烦人卡特殊情况
        if is_nope_card {
            return self.play_nope(match_id, user_id, card_id).await;
        }
        
        // 查找卡牌
        let card_index = match_data.players[player_index].hand.iter()
            .position(|c| c.id == card_id)
            .ok_or_else(|| anyhow::anyhow!("卡牌不存在"))?;
        
        // 获取卡牌
        let card = match_data.players[player_index].hand[card_index].clone();
        
        // 创建卡牌动作
        let card_action = CardAction {
            action_type: CardActionType::Play,
            user_id: user_id.to_string(),
            card_id: Some(card_id.to_string()),
            card_type: Some(card.card_type.clone()),
            is_canceled: false,
            created_at: chrono::Utc::now().timestamp_millis() as u64,
        };
        
        // 移除卡牌（先从手中移除）
        match_data.players[player_index].hand.remove(card_index);
        
        // 将卡牌放入弃牌堆
        match_data.discard_pile.push(card.clone());
        
        // 保存游戏数据（确保卡牌已从手中移除并放入弃牌堆）
        match_data.updated_at = chrono::Utc::now().timestamp_millis() as u64;
        self.save_match(&match_data).await;
        
        // 广播出牌事件
        let play_response = WsResponse {
            ok: true,
            msg: Some(format!("玩家 {} 打出了 {:?}", user_id, card.card_type)),
            payload: Some(serde_json::json!({
                "userId": user_id,
                "card": card
            })),
        };
        
        self.connection_manager.broadcast_to_room(
            match_id,
            events::match_events::PLAY_CARD,
            Some(serde_json::to_value(play_response)?),
        ).await?;
        
        // 启动连锁效果系统
        self.start_card_chain(match_id, card_action).await?;
        
        Ok(())
    }
    
    /// 切换回合
    pub async fn change_turn(&self, match_id: &str) -> Result<()> {
        // 获取游戏数据
        let mut match_data = self.get_match(match_id).await
            .ok_or_else(|| anyhow::anyhow!("游戏不存在"))?;
        
        // 重置当前玩家的回合标志
        if let Some(current_player) = match_data.players.get_mut(match_data.turn_index) {
            current_player.is_turn = false;
        }
        
        // 计算下一个玩家的索引
        match_data.turn_index = (match_data.turn_index + 1) % match_data.players.len();
        
        // 设置下一个玩家的回合标志
        if let Some(next_player) = match_data.players.get_mut(match_data.turn_index) {
            next_player.is_turn = true;
            
            // 广播回合变更事件
            let turn_response = WsResponse {
                ok: true,
                msg: Some(format!("轮到玩家 {} 的回合", next_player.user.id)),
                payload: Some(serde_json::json!({
                    "userId": next_player.user.id,
                    "turnIndex": match_data.turn_index
                })),
            };
            
            self.connection_manager.broadcast_to_room(
                match_id,
                events::match_events::TURN_CHANGE,
                Some(serde_json::to_value(turn_response)?),
            ).await?;
        }
        
        // 保存游戏数据
        match_data.updated_at = chrono::Utc::now().timestamp_millis() as u64;
        self.save_match(&match_data).await;
        
        Ok(())
    }
    
    /// 加入观战
    pub async fn join_spectator(&self, match_id: &str, user_info: UserInfo, client_id: &str) -> Result<()> {
        // 获取游戏数据
        let mut match_data = self.get_match(match_id).await
            .ok_or_else(|| anyhow::anyhow!("游戏不存在"))?;
        
        // 检查用户是否已经在游戏中
        let is_player = match_data.players.iter().any(|p| p.user.id == user_info.id);
        let is_out = match_data.out.iter().any(|p| p.user.id == user_info.id);
        
        if is_player || is_out {
            return Err(anyhow::anyhow!("玩家已在游戏中，不能观战"));
        }
        
        // 检查用户是否已经是观战者
        if match_data.spectators.iter().any(|s| s.id == user_info.id) {
            return Err(anyhow::anyhow!("用户已经在观战"));
        }
        
        // 添加观战者
        match_data.spectators.push(user_info.clone());
        
        // 保存游戏数据
        match_data.updated_at = chrono::Utc::now().timestamp_millis() as u64;
        self.save_match(&match_data).await;
        
        // 加入WebSocket房间 - 使用手动实现加入房间
        self.connection_manager.broadcast_to_room(match_id, "system:join", Some(serde_json::json!({
            "client_id": client_id
        }))).await?;
        
        // 广播有新观战者加入
        let spectator_response = WsResponse {
            ok: true,
            msg: Some(format!("{} 加入观战", user_info.name)),
            payload: Some(serde_json::json!({
                "user": user_info
            })),
        };
        
        self.connection_manager.broadcast_to_room(
            match_id,
            events::match_events::JOIN_SPECTATORS,
            Some(serde_json::to_value(spectator_response)?),
        ).await?;
        
        // 发送当前游戏状态给观战者
        let game_response = WsResponse {
            ok: true,
            msg: Some("游戏状态".to_string()),
            payload: Some(serde_json::to_value(&match_data)?),
        };
        
        self.connection_manager.send_to_client(
            client_id,
            events::match_events::JOIN,
            Some(serde_json::to_value(game_response)?),
        ).await?;
        
        Ok(())
    }
    
    /// 离开观战
    pub async fn leave_spectator(&self, match_id: &str, user_id: &str, client_id: &str) -> Result<()> {
        // 获取游戏数据
        let mut match_data = self.get_match(match_id).await
            .ok_or_else(|| anyhow::anyhow!("游戏不存在"))?;
        
        // 查找并移除观战者
        let spectator_index = match_data.spectators.iter()
            .position(|s| s.id == user_id);
        
        if let Some(index) = spectator_index {
            let spectator = match_data.spectators.remove(index);
            
            // 保存游戏数据
            match_data.updated_at = chrono::Utc::now().timestamp_millis() as u64;
            self.save_match(&match_data).await;
            
            // 离开WebSocket房间 - 使用手动实现离开房间
            self.connection_manager.broadcast_to_room(match_id, "system:leave", Some(serde_json::json!({
                "client_id": client_id
            }))).await?;
            
            // 广播观战者离开
            let spectator_response = WsResponse {
                ok: true,
                msg: Some(format!("{} 离开观战", spectator.name)),
                payload: Some(serde_json::json!({
                    "userId": user_id
                })),
            };
            
            self.connection_manager.broadcast_to_room(
                match_id,
                events::match_events::LEAVE_SPECTATORS,
                Some(serde_json::to_value(spectator_response)?),
            ).await?;
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("用户不是观战者"))
        }
    }
    
    /// 处理玩家超时
    pub async fn handle_player_timeout(&self, match_id: &str, user_id: &str) -> Result<()> {
        // 获取游戏数据
        let mut match_data = self.get_match(match_id).await
            .ok_or_else(|| anyhow::anyhow!("游戏不存在"))?;
        
        // 检查游戏状态
        if match_data.state != MatchState::InProgress {
            return Err(anyhow::anyhow!("游戏未开始或已结束"));
        }
        
        // 查找玩家
        let player_index = match_data.players.iter().position(|p| p.user.id == user_id);
        
        if let Some(index) = player_index {
            // 检查是否是当前玩家的回合
            if match_data.players[index].is_turn {
                // 玩家超时，移到出局列表
                let mut player = match_data.players.remove(index);
                player.is_active = false;
                match_data.out.push(player);
                
                // 广播超时事件
                let timeout_response = WsResponse {
                    ok: true,
                    msg: Some(format!("玩家 {} 因超时而出局", user_id)),
                    payload: Some(serde_json::json!({
                        "userId": user_id,
                        "reason": "timeout"
                    })),
                };
                
                self.connection_manager.broadcast_to_room(
                    match_id,
                    events::match_events::DEFEAT,
                    Some(serde_json::to_value(timeout_response)?),
                ).await?;
                
                // 检查游戏是否结束
                if match_data.players.len() <= 1 {
                    // 使用新的游戏结束处理方法
                    self.handle_game_end(match_id).await?;
                } else {
                    // 游戏继续，切换到下一玩家
                    self.change_turn(match_id).await?;
                    
                    // 保存游戏数据
                    match_data.updated_at = chrono::Utc::now().timestamp_millis() as u64;
                    self.save_match(&match_data).await;
                }
                
                Ok(())
            } else {
                Err(anyhow::anyhow!("不是该玩家的回合"))
            }
        } else {
            Err(anyhow::anyhow!("玩家不在游戏中"))
        }
    }
    
    /// 设置超时处理
    pub async fn setup_inactivity_timer(&self, match_id: &str, user_id: &str, timeout: u64) {
        let match_service = self.clone();
        let match_id_clone = match_id.to_string();
        let user_id_clone = user_id.to_string();
        
        tokio::spawn(async move {
            // 延迟指定时间
            sleep(Duration::from_millis(timeout)).await;
            
            // 检查游戏是否还存在及用户是否还在游戏中
            match match_service.get_match(&match_id_clone).await {
                Some(match_data) => {
                    if match_data.state == MatchState::InProgress {
                        // 找到当前回合的玩家
                        let current_player = match_data.players.get(match_data.turn_index);
                        
                        if let Some(player) = current_player {
                            if player.user.id == user_id_clone && player.is_turn {
                                // 玩家仍然是当前回合，执行超时处理
                                if let Err(e) = match_service.handle_player_timeout(&match_id_clone, &user_id_clone).await {
                                    error!("处理玩家超时失败: {}", e);
                                }
                            }
                        }
                    }
                },
                None => {
                    debug!("游戏 {} 不存在，忽略超时处理", match_id_clone);
                }
            }
        });
    }
    
    /// 更新玩家评分
    /// 使用 ELO 评分系统计算并更新所有玩家的评分
    pub async fn update_player_ratings(&self, match_id: &str) -> Result<()> {
        // 获取游戏数据
        let match_data = self.get_match(match_id).await
            .ok_or_else(|| anyhow::anyhow!("游戏不存在"))?;
        
        // 确保游戏已经结束
        if match_data.state != MatchState::Completed {
            return Err(anyhow::anyhow!("游戏尚未结束，无法更新评分"));
        }
        
        // 获取所有参与玩家的评分（包括胜利者和失败者）
        let mut all_players = Vec::new();
        all_players.extend(match_data.players.iter());
        all_players.extend(match_data.out.iter());
        
        // 找到胜利者
        let winner = match_data.players.iter().find(|p| p.is_winner);
        
        if let Some(winner) = winner {
            info!("计算玩家 {} 的新评分（胜利）", winner.user.id);
            
            // 收集其他玩家的评分
            let opponent_ratings: Vec<i32> = all_players.iter()
                .filter(|p| p.user.id != winner.user.id)
                .map(|p| p.user.rating)
                .collect();
            
            // 计算胜利者的新评分
            let new_rating = elo::if_won(winner.user.rating, &opponent_ratings);
            
            // 记录评分变化
            info!("玩家 {} 的评分从 {} 更新为 {} （+{}）", 
                 winner.user.id, 
                 winner.user.rating, 
                 new_rating,
                 new_rating - winner.user.rating);
            
            // 更新数据库中的玩家评分
            // 注意：在实际实现中，这里应该调用数据库或用户服务来更新永久存储的评分
            // 下面是示意代码
            // await update_user_rating_in_database(winner.user.id, new_rating);
            
            // 计算并更新失败者的评分
            for player in all_players.iter().filter(|p| p.user.id != winner.user.id) {
                // 收集对手评分，包括胜利者
                let opponent_ratings = vec![winner.user.rating];
                
                // 计算新评分
                let new_rating = elo::if_lost(player.user.rating, &opponent_ratings);
                
                // 记录评分变化
                info!("玩家 {} 的评分从 {} 更新为 {} （{}）", 
                     player.user.id, 
                     player.user.rating, 
                     new_rating,
                     new_rating - player.user.rating);
                
                // 更新数据库中的玩家评分
                // await update_user_rating_in_database(player.user.id, new_rating);
            }
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("游戏已结束但未找到胜利者"))
        }
    }
    
    /// 处理游戏结束
    async fn handle_game_end(&self, match_id: &str) -> Result<()> {
        // 获取游戏数据
        let mut match_data = self.get_match(match_id).await
            .ok_or_else(|| anyhow::anyhow!("游戏不存在"))?;
        
        // 确保游戏已处于进行中状态
        if match_data.state != MatchState::InProgress {
            return Err(anyhow::anyhow!("游戏未处于进行中状态"));
        }
        
        // 如果只剩一名玩家，游戏结束
        if match_data.players.len() <= 1 {
            {
                // 使用代码块来限制可变引用的作用域
                let last_player = match_data.players.first_mut().unwrap();
                // 标记为胜利者
                last_player.is_winner = true;
                
                // 更新游戏状态
                match_data.state = MatchState::Completed;
                match_data.updated_at = chrono::Utc::now().timestamp_millis() as u64;
            } // last_player的可变引用在这里结束
            
            // 克隆数据供后续使用
            let match_data_clone = match_data.clone();
            self.save_match(&match_data).await;
            
            // 获取胜利者的用户ID用于响应
            let winner_id = match_data.players.first().unwrap().user.id.clone();
            
            // 广播胜利事件
            let victory_response = WsResponse {
                ok: true,
                msg: Some(format!("玩家 {} 获胜", winner_id)),
                payload: Some(serde_json::json!({
                    "userId": winner_id
                })),
            };
            
            self.connection_manager.broadcast_to_room(
                match_id,
                events::match_events::VICTORY,
                Some(serde_json::to_value(victory_response)?),
            ).await?;
            
            // 广播游戏结束事件
            let end_response = WsResponse {
                ok: true,
                msg: Some("游戏结束".to_string()),
                payload: Some(serde_json::to_value(&match_data_clone)?),
            };
            
            self.connection_manager.broadcast_to_room(
                match_id,
                events::match_events::END,
                Some(serde_json::to_value(end_response)?),
            ).await?;
            
            // 更新玩家评分
            if let Err(e) = self.update_player_ratings(match_id).await {
                error!("更新玩家评分失败: {}", e);
            }
            
            return Ok(());
        }
        
        Err(anyhow::anyhow!("游戏尚未达到结束条件"))
    }

    /// 使用烦人卡（Nope）取消上一个操作
    pub async fn play_nope(&self, match_id: &str, user_id: &str, card_id: &str) -> Result<()> {
        // 获取游戏数据
        let mut match_data = self.get_match(match_id).await
            .ok_or_else(|| anyhow::anyhow!("游戏不存在"))?;
        
        // 检查游戏状态
        if match_data.state != MatchState::InProgress {
            return Err(anyhow::anyhow!("游戏未开始或已结束"));
        }
        
        // 查找玩家
        let player_index = match_data.players.iter().position(|p| p.user.id == user_id)
            .ok_or_else(|| anyhow::anyhow!("玩家不在游戏中"))?;
        
        // 检查是否有连锁状态
        if match_data.chain_state.is_none() {
            return Err(anyhow::anyhow!("没有可以取消的操作"));
        }
        
        // 查找烦人卡
        let card_index = match_data.players[player_index].hand.iter()
            .position(|c| c.id == card_id && matches!(c.card_type, CardType::Nope))
            .ok_or_else(|| anyhow::anyhow!("玩家没有烦人卡或指定卡不是烦人卡"))?;
        
        // 移除烦人卡
        let nope_card = match_data.players[player_index].hand.remove(card_index);
        
        // 将烦人卡放入弃牌堆
        match_data.discard_pile.push(nope_card.clone());
        
        // 标记连锁动作为取消
        if let Some(ref mut chain_action) = match_data.chain_state {
            chain_action.is_canceled = true;
            
            // 记录使用烦人卡的动作
            let nope_action = CardAction {
                action_type: CardActionType::Nope,
                user_id: user_id.to_string(),
                card_id: Some(card_id.to_string()),
                card_type: Some(CardType::Nope),
                is_canceled: false,
                created_at: chrono::Utc::now().timestamp_millis() as u64,
            };
            
            // 添加到动作历史
            match_data.action_history.push(nope_action);
            
            // 广播烦人卡使用事件
            let nope_response = WsResponse {
                ok: true,
                msg: Some(format!("玩家 {} 使用烦人卡取消了上一个操作", user_id)),
                payload: Some(serde_json::json!({
                    "userId": user_id,
                    "cardId": card_id,
                    "canceledAction": chain_action
                })),
            };
            
            self.connection_manager.broadcast_to_room(
                match_id,
                events::match_events::PLAY_CARD,
                Some(serde_json::to_value(nope_response)?),
            ).await?;
            
            // 清除连锁状态
            match_data.chain_state = None;
            
            // 保存游戏数据
            match_data.updated_at = chrono::Utc::now().timestamp_millis() as u64;
            self.save_match(&match_data).await;
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("没有可以取消的操作"))
        }
    }
    
    /// 开始卡牌连锁效果
    async fn start_card_chain(&self, match_id: &str, action: CardAction) -> Result<bool> {
        // 获取游戏数据
        let mut match_data = self.get_match(match_id).await
            .ok_or_else(|| anyhow::anyhow!("游戏不存在"))?;
        
        // 设置连锁状态
        match_data.chain_state = Some(action.clone());
        
        // 添加到动作历史
        match_data.action_history.push(action.clone());
        
        // 保存游戏数据
        match_data.updated_at = chrono::Utc::now().timestamp_millis() as u64;
        self.save_match(&match_data).await;
        
        // 广播连锁开始事件
        let chain_response = WsResponse {
            ok: true,
            msg: Some("开始卡牌连锁效果，可以使用烦人卡取消".to_string()),
            payload: Some(serde_json::json!({
                "action": action,
                "waitTime": match_data.chain_wait_time
            })),
        };
        
        self.connection_manager.broadcast_to_room(
            match_id,
            "match:chain_start",
            Some(serde_json::to_value(chain_response)?),
        ).await?;
        
        // 设置超时处理
        let match_service = self.clone();
        let match_id_clone = match_id.to_string();
        
        tokio::spawn(async move {
            // 等待指定时间
            sleep(Duration::from_millis(match_data.chain_wait_time)).await;
            
            // 尝试结束连锁
            if let Err(e) = match_service.end_card_chain(&match_id_clone).await {
                error!("结束卡牌连锁失败: {}", e);
            }
        });
        
        Ok(true)
    }
    
    /// 结束卡牌连锁效果
    async fn end_card_chain(&self, match_id: &str) -> Result<bool> {
        // 获取游戏数据
        let mut match_data = self.get_match(match_id).await
            .ok_or_else(|| anyhow::anyhow!("游戏不存在"))?;
        
        // 检查是否有连锁状态
        if let Some(chain_action) = match_data.chain_state.clone() {
            // 如果动作没有被取消，则执行
            if !chain_action.is_canceled {
                // 广播连锁结束事件
                let end_response = WsResponse {
                    ok: true,
                    msg: Some("卡牌连锁效果结束，动作有效".to_string()),
                    payload: Some(serde_json::json!({
                        "action": chain_action
                    })),
                };
                
                self.connection_manager.broadcast_to_room(
                    match_id,
                    "match:chain_end",
                    Some(serde_json::to_value(end_response)?),
                ).await?;
                
                // 根据动作类型继续执行效果
                match chain_action.action_type {
                    CardActionType::Play => {
                        if let (Some(card_id), Some(user_id)) = (chain_action.card_id, Some(chain_action.user_id)) {
                            // 执行出牌效果，但跳过连锁处理
                            self.execute_card_effect(match_id, &user_id, &card_id).await?;
                        }
                    },
                    CardActionType::Draw => {
                        // 抽卡动作通常不会被放入连锁
                    },
                    CardActionType::Nope => {
                        // Nope只是取消效果，不需要额外执行
                    },
                    CardActionType::Defuse => {
                        // 拆除卡动作通常不会被放入连锁
                    },
                }
            } else {
                // 动作被取消
                let cancel_response = WsResponse {
                    ok: true,
                    msg: Some("卡牌连锁效果结束，动作被取消".to_string()),
                    payload: Some(serde_json::json!({
                        "action": chain_action
                    })),
                };
                
                self.connection_manager.broadcast_to_room(
                    match_id,
                    "match:chain_end",
                    Some(serde_json::to_value(cancel_response)?),
                ).await?;
            }
            
            // 清除连锁状态
            match_data.chain_state = None;
            
            // 保存游戏数据
            match_data.updated_at = chrono::Utc::now().timestamp_millis() as u64;
            self.save_match(&match_data).await;
            
            Ok(true)
        } else {
            // 没有连锁状态，不需要处理
            Ok(false)
        }
    }
    
    /// 执行卡牌效果（不进入连锁系统）
    async fn execute_card_effect(&self, match_id: &str, user_id: &str, card_id: &str) -> Result<()> {
        // 获取游戏数据
        let mut match_data = self.get_match(match_id).await
            .ok_or_else(|| anyhow::anyhow!("游戏不存在"))?;
        
        // 查找玩家
        let player_index = match_data.players.iter().position(|p| p.user.id == user_id)
            .ok_or_else(|| anyhow::anyhow!("玩家不在游戏中"))?;
        
        // 查找卡牌（在弃牌堆中，因为在开始连锁前已经移除）
        let card = match_data.discard_pile.iter()
            .find(|c| c.id == card_id)
            .ok_or_else(|| anyhow::anyhow!("卡牌不存在"))?
            .clone();
        
        // 处理卡牌效果
        match card.card_type {
            CardType::Skip => {
                // 跳过当前回合
                self.change_turn(match_id).await?;
            },
            CardType::Attack => {
                // 攻击：下一玩家连续抽两张牌
                self.change_turn(match_id).await?;
                
                // 标记下一玩家需要抽两张牌
                // 在实际游戏中，需要更复杂的机制来处理
                // 此示例中简化为记录在游戏状态中
                match_data.draw_count = 2;
            },
            CardType::Shuffle => {
                // 洗牌
                use rand::seq::SliceRandom;
                use rand::thread_rng;
                
                match_data.deck.shuffle(&mut thread_rng());
                
                // 不切换回合，玩家可以继续操作
            },
            CardType::SeeTheFuture => {
                // 偷看未来三张牌
                let future_cards = match_data.deck.iter()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>();
                
                // 私下通知玩家
                let future_response = WsResponse {
                    ok: true,
                    msg: Some("你看到了未来的牌".to_string()),
                    payload: Some(serde_json::json!({
                        "cards": future_cards
                    })),
                };
                
                self.connection_manager.send_to_client(
                    &user_id,
                    events::match_events::PLAY_CARD,
                    Some(serde_json::to_value(future_response)?),
                ).await?;
                
                // 不切换回合，玩家可以继续操作
            },
            CardType::Favor => {
                // 获取其他玩家的一张牌
                // 在实际游戏中，需要等待玩家选择目标
                // 此示例中简化为随机选择一名玩家
                
                let other_players = match_data.players.iter_mut()
                    .enumerate()
                    .filter(|(i, p)| *i != player_index && !p.hand.is_empty())
                    .collect::<Vec<_>>();
                
                if !other_players.is_empty() {
                    use rand::Rng;
                    let random_index = rand::thread_rng().gen_range(0..other_players.len());
                    let (target_index, _) = other_players[random_index];
                    
                    // 随机选择一张牌
                    let random_card_index = rand::thread_rng().gen_range(0..match_data.players[target_index].hand.len());
                    let target_card = match_data.players[target_index].hand.remove(random_card_index);
                    
                    // 获取目标玩家ID（用于消息）
                    let target_player_id = match_data.players[target_index].user.id.clone();
                    
                    // 加入当前玩家手牌
                    match_data.players[player_index].hand.push(target_card.clone());
                    
                    // 广播抢夺事件
                    let favor_response = WsResponse {
                        ok: true,
                        msg: Some(format!("玩家 {} 从玩家 {} 那里获得了一张牌", 
                                        user_id, target_player_id)),
                        payload: Some(serde_json::json!({
                            "userId": user_id,
                            "targetId": target_player_id
                        })),
                    };
                    
                    self.connection_manager.broadcast_to_room(
                        match_id,
                        events::match_events::PLAY_CARD,
                        Some(serde_json::to_value(favor_response)?),
                    ).await?;
                    
                    // 私下通知当前玩家获得的牌
                    let private_response = WsResponse {
                        ok: true,
                        msg: Some(format!("你从玩家 {} 那里获得了 {:?}", 
                                        target_player_id, target_card.card_type)),
                        payload: Some(serde_json::json!({
                            "card": target_card
                        })),
                    };
                    
                    self.connection_manager.send_to_client(
                        &user_id,
                        events::match_events::PLAY_CARD,
                        Some(serde_json::to_value(private_response)?),
                    ).await?;
                }
                
                // 不切换回合，玩家可以继续操作
            },
            CardType::AlterTheFuture => {
                // 查看并重新排列未来三张牌
                if match_data.deck.len() >= 3 {
                    // 取出前三张牌
                    let mut future_cards = Vec::new();
                    for _ in 0..3 {
                        if let Some(card) = match_data.deck.pop() {
                            future_cards.push(card);
                        }
                    }
                    
                    // 显示给玩家
                    let future_response = WsResponse {
                        ok: true,
                        msg: Some("你可以重新排列未来的牌".to_string()),
                        payload: Some(serde_json::json!({
                            "cards": future_cards
                        })),
                    };
                    
                    self.connection_manager.send_to_client(
                        &user_id,
                        events::match_events::ALTER_FUTURE,
                        Some(serde_json::to_value(future_response)?),
                    ).await?;
                    
                    // 这里简化处理，随机排列这些牌
                    use rand::seq::SliceRandom;
                    use rand::thread_rng;
                    future_cards.shuffle(&mut thread_rng());
                    
                    // 放回牌堆顶部
                    for card in future_cards.into_iter().rev() {
                        match_data.deck.push(card);
                    }
                    
                    // 通知玩家已重新排列
                    let alter_response = WsResponse {
                        ok: true,
                        msg: Some("已重新排列未来的牌".to_string()),
                        payload: None,
                    };
                    
                    self.connection_manager.send_to_client(
                        &user_id,
                        events::match_events::ALTER_FUTURE,
                        Some(serde_json::to_value(alter_response)?),
                    ).await?;
                }
                
                // 不切换回合，玩家可以继续操作
            },
            CardType::ShareTheFuture => {
                // 与另一位玩家分享未来三张牌
                let future_cards = match_data.deck.iter()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>();
                
                if !future_cards.is_empty() {
                    // 随机选择一名其他玩家
                    let other_players = match_data.players.iter()
                        .filter(|p| p.user.id != user_id)
                        .collect::<Vec<_>>();
                    
                    if !other_players.is_empty() {
                        use rand::Rng;
                        let random_index = rand::thread_rng().gen_range(0..other_players.len());
                        let target_player = &other_players[random_index];
                        
                        // 向目标玩家分享卡牌
                        let share_response = WsResponse {
                            ok: true,
                            msg: Some(format!("玩家 {} 与你分享了未来的牌", user_id)),
                            payload: Some(serde_json::json!({
                                "cards": future_cards,
                                "fromUserId": user_id
                            })),
                        };
                        
                        self.connection_manager.send_to_client(
                            &target_player.user.id,
                            events::match_events::SHARE_FUTURE,
                            Some(serde_json::to_value(share_response)?),
                        ).await?;
                        
                        // 通知当前玩家已分享
                        let notify_response = WsResponse {
                            ok: true,
                            msg: Some(format!("你与玩家 {} 分享了未来的牌", target_player.user.name)),
                            payload: Some(serde_json::json!({
                                "cards": future_cards,
                                "toUserId": target_player.user.id
                            })),
                        };
                        
                        self.connection_manager.send_to_client(
                            &user_id,
                            events::match_events::SHARE_FUTURE,
                            Some(serde_json::to_value(notify_response)?),
                        ).await?;
                    }
                }
                
                // 不切换回合，玩家可以继续操作
            },
            CardType::BuryCard => {
                // 将一张牌埋入牌堆中间
                if !match_data.deck.is_empty() {
                    // 选择要埋的牌
                    // 这里简化为让玩家埋入最后一张牌堆牌
                    if let Some(card_to_bury) = match_data.deck.pop() {
                        // 计算中间位置
                        let middle_position = match_data.deck.len() / 2;
                        
                        // 插入牌
                        match_data.deck.insert(middle_position, card_to_bury.clone());
                        
                        // 通知玩家
                        let bury_response = WsResponse {
                            ok: true,
                            msg: Some("你将一张牌埋入了牌堆中间".to_string()),
                            payload: Some(serde_json::json!({
                                "buriedCard": card_to_bury
                            })),
                        };
                        
                        self.connection_manager.send_to_client(
                            &user_id,
                            events::match_events::BURY_CARD,
                            Some(serde_json::to_value(bury_response)?),
                        ).await?;
                        
                        // 广播埋牌事件
                        let public_response = WsResponse {
                            ok: true,
                            msg: Some(format!("玩家 {} 将一张牌埋入了牌堆中间", user_id)),
                            payload: None,
                        };
                        
                        self.connection_manager.broadcast_to_room(
                            match_id,
                            events::match_events::BURY_CARD,
                            Some(serde_json::to_value(public_response)?),
                        ).await?;
                    }
                }
                
                // 不切换回合，玩家可以继续操作
            },
            CardType::SpeedUpExplosion => {
                // 加速爆炸猫的爆炸
                // 将一张爆炸猫移到牌堆顶部附近
                
                // 查找牌堆中的爆炸猫
                let exploding_position = match_data.deck.iter().position(|c| 
                    matches!(c.card_type, CardType::ExplodingKitten)
                );
                
                if let Some(pos) = exploding_position {
                    // 取出爆炸猫
                    let exploding_card = match_data.deck.remove(pos);
                    
                    // 放到牌堆顶部附近的随机位置
                    use rand::Rng;
                    let top_range = (match_data.deck.len() / 4).max(1);
                    let new_pos = rand::thread_rng().gen_range(0..top_range);
                    
                    match_data.deck.insert(new_pos, exploding_card);
                    
                    // 广播事件
                    let speed_response = WsResponse {
                        ok: true,
                        msg: Some(format!("玩家 {} 加速了爆炸猫的爆炸", user_id)),
                        payload: None,
                    };
                    
                    self.connection_manager.broadcast_to_room(
                        match_id,
                        events::match_events::SPEED_UP_EXPLOSION,
                        Some(serde_json::to_value(speed_response)?),
                    ).await?;
                }
                
                // 不切换回合，玩家可以继续操作
            },
            CardType::ImplodingKitten => {
                // 内爆猫，类似爆炸猫，但有不同效果
                // 在这里，我们简化为插入一个新的爆炸猫
                
                // 创建一个新的爆炸猫
                let imploding_card = Card {
                    id: format!("imploding-{}", Uuid::new_v4()),
                    card_type: CardType::ExplodingKitten,
                    variant: Some("imploding".to_string()),
                };
                
                // 将其插入牌堆中间
                let middle_position = match_data.deck.len() / 2;
                match_data.deck.insert(middle_position, imploding_card.clone());
                
                // 广播事件
                let implode_response = WsResponse {
                    ok: true,
                    msg: Some(format!("玩家 {} 插入了一只内爆猫", user_id)),
                    payload: Some(serde_json::json!({
                        "position": "middle"
                    })),
                };
                
                self.connection_manager.broadcast_to_room(
                    match_id,
                    events::match_events::INSERT_IMPLODING_KITTEN,
                    Some(serde_json::to_value(implode_response)?),
                ).await?;
                
                // 不切换回合，玩家可以继续操作
            },
            CardType::Cat => {
                // 猫咪卡，需要配对使用
                // 在这个简化版本中，我们只是使用它，没有特殊效果
                // 在实际游戏中，应检查玩家是否有配对所需的其他猫咪卡
                
                // 广播使用猫咪卡
                let cat_response = WsResponse {
                    ok: true,
                    msg: Some(format!("玩家 {} 使用了猫咪卡", user_id)),
                    payload: None,
                };
                
                self.connection_manager.broadcast_to_room(
                    match_id,
                    events::match_events::PLAY_CARD,
                    Some(serde_json::to_value(cat_response)?),
                ).await?;
                
                // 不切换回合，玩家可以继续操作
            },
            CardType::Nope => {
                // Nope卡在连锁系统中单独处理
                // 在正常出牌阶段使用时，没有特殊效果
                
                // 广播使用Nope卡
                let nope_response = WsResponse {
                    ok: true,
                    msg: Some(format!("玩家 {} 使用了烦人卡", user_id)),
                    payload: None,
                };
                
                self.connection_manager.broadcast_to_room(
                    match_id,
                    events::match_events::PLAY_CARD,
                    Some(serde_json::to_value(nope_response)?),
                ).await?;
                
                // 不切换回合，玩家可以继续操作
            },
            // 其他卡牌的默认处理
            _ => {
                // 对于其他未专门处理的卡牌类型，记录日志并跳过特殊效果
                info!("未实现的卡牌效果: {:?}", card.card_type);
                
                // 广播一个通用的出牌消息
                let generic_response = WsResponse {
                    ok: true,
                    msg: Some(format!("玩家 {} 使用了 {:?} 卡牌", user_id, card.card_type)),
                    payload: None,
                };
                
                self.connection_manager.broadcast_to_room(
                    match_id,
                    events::match_events::PLAY_CARD,
                    Some(serde_json::to_value(generic_response)?),
                ).await?;
                
                // 不切换回合，玩家可以继续操作
            }
        }
        
        // 保存游戏数据
        match_data.updated_at = chrono::Utc::now().timestamp_millis() as u64;
        self.save_match(&match_data).await;
        
        Ok(())
    }
}

/// 生成牌组
fn generate_deck(player_count: usize) -> Vec<Card> {
    let mut deck = Vec::new();
    let mut rng = thread_rng();
    
    // 添加爆炸猫卡（玩家数量-1）
    for i in 0..player_count - 1 {
        deck.push(Card {
            id: format!("exploding-{}", i),
            card_type: CardType::ExplodingKitten,
            variant: None,
        });
    }
    
    // 每个玩家添加1张拆除卡
    for i in 0..player_count {
        deck.push(Card {
            id: format!("defuse-{}", i),
            card_type: CardType::Defuse,
            variant: None,
        });
    }
    
    // 添加标准卡牌
    let card_types = [
        CardType::Skip,
        CardType::SeeTheFuture,
        CardType::Shuffle,
        CardType::Attack,
        CardType::Favor,
        CardType::Cat,
        CardType::Nope,
    ];
    
    // 每种卡牌添加4张
    for (type_index, card_type) in card_types.iter().enumerate() {
        for i in 0..4 {
            deck.push(Card {
                id: format!("{}-{}", type_index, i),
                card_type: card_type.clone(),
                variant: None,
            });
        }
    }
    
    // 洗牌
    deck.shuffle(&mut rng);
    
    deck
}

/// 发牌
fn distribute_cards(match_data: &mut MatchData) {
    const INITIAL_CARD_COUNT: usize = 4; // 每个玩家初始卡牌数
    
    for player in &mut match_data.players {
        for _ in 0..INITIAL_CARD_COUNT {
            if let Some(card) = match_data.deck.pop() {
                player.hand.push(card);
            }
        }
        
        // 确保每个玩家有一张拆除卡
        // 检查玩家是否已经有拆除卡
        let has_defuse = player.hand.iter().any(|card| matches!(card.card_type, CardType::Defuse));
        
        if !has_defuse {
            // 从牌堆找一张拆除卡
            if let Some(pos) = match_data.deck.iter().position(|card| matches!(card.card_type, CardType::Defuse)) {
                let defuse_card = match_data.deck.remove(pos);
                player.hand.push(defuse_card);
            } else {
                // 如果牌堆中没有拆除卡，创建一张新的
                player.hand.push(Card {
                    id: format!("defuse-extra-{}", player.user.id),
                    card_type: CardType::Defuse,
                    variant: None,
                });
            }
        }
    }
}

/// 克隆实现
impl Clone for MatchService {
    fn clone(&self) -> Self {
        Self {
            game_service: self.game_service.clone(),
            connection_manager: self.connection_manager.clone(),
            active_matches: self.active_matches.clone(),
            queue: self.queue.clone(),
        }
    }
}

/// 处理WebSocket消息
pub async fn handle_ws_message(
    client_id: &str,
    message: WsMessage,
    match_service: &MatchService,
    user_info: Option<UserInfo>,
) -> Result<bool> {
    // 检查是否有用户信息
    let user = match user_info {
        Some(user) => user,
        None => return Ok(false), // 没有用户信息，无法处理
    };
    
    match message.event.as_str() {
        // 匹配相关事件
        "match:join" => {
            if let Some(data) = message.data {
                if let Some(match_id) = data.get("matchId").and_then(|v| v.as_str()) {
                    match_service.join_match(match_id, &user.id, client_id).await?;
                    return Ok(true);
                }
            }
        }
        "match:leave" => {
            if let Some(data) = message.data {
                if let Some(match_id) = data.get("matchId").and_then(|v| v.as_str()) {
                    match_service.leave_match(match_id, &user.id, client_id).await?;
                    return Ok(true);
                }
            }
        }
        "match:start" => {
            if let Some(data) = message.data {
                if let Some(match_id) = data.get("matchId").and_then(|v| v.as_str()) {
                    match_service.start_game(match_id).await?;
                    return Ok(true);
                }
            }
        }
        "match:draw_card" => {
            if let Some(data) = message.data {
                if let Some(match_id) = data.get("matchId").and_then(|v| v.as_str()) {
                    match_service.draw_card(match_id, &user.id).await?;
                    return Ok(true);
                }
            }
        }
        "match:play_card" => {
            if let Some(data) = message.data {
                if let Some(match_id) = data.get("matchId").and_then(|v| v.as_str()) {
                    if let Some(card_id) = data.get("cardId").and_then(|v| v.as_str()) {
                        match_service.play_card(match_id, &user.id, card_id).await?;
                        return Ok(true);
                    }
                }
            }
        }
        "match:join_spectators" => {
            if let Some(data) = message.data {
                if let Some(match_id) = data.get("matchId").and_then(|v| v.as_str()) {
                    match_service.join_spectator(match_id, user.clone(), client_id).await?;
                    return Ok(true);
                }
            }
        }
        "match:leave_spectators" => {
            if let Some(data) = message.data {
                if let Some(match_id) = data.get("matchId").and_then(|v| v.as_str()) {
                    match_service.leave_spectator(match_id, &user.id, client_id).await?;
                    return Ok(true);
                }
            }
        }
        "queue:join" => {
            match_service.join_queue(user).await?;
            return Ok(true);
        }
        "queue:leave" => {
            match_service.leave_queue(&user.id).await?;
            return Ok(true);
        }
        "queue:status" => {
            // 获取队列状态
            let enqueued_at = match_service.get_queue_status(&user.id).await;
            
            // 创建响应
            let response = WsResponse {
                ok: true,
                msg: None,
                payload: Some(serde_json::json!({
                    "isEnqueued": enqueued_at.is_some(),
                    "enqueuedAt": enqueued_at
                })),
            };
            
            // 发送响应
            match_service.connection_manager.send_to_client(
                client_id,
                "queue:status",
                Some(serde_json::to_value(response)?),
            ).await?;
            
            return Ok(true);
        }
        _ => {
            // 其他事件不处理
            return Ok(false);
        }
    }
    
    Ok(false)
}

/// 初始化游戏匹配服务
pub fn init_match_service(
    game_service: Arc<GameService>,
    connection_manager: Arc<ConnectionManager>,
) -> Arc<MatchService> {
    let match_service = Arc::new(MatchService::new(game_service, connection_manager));
    
    // 启动匹配队列处理
    let match_service_clone = match_service.clone();
    tokio::spawn(async move {
        match_service_clone.start_matchmaking().await;
    });
    
    match_service
}

/// 默认连锁等待时间
fn default_chain_wait_time() -> u64 {
    5000 // 5秒
}
