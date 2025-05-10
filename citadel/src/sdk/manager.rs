use std::collections::HashMap;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use sui_types::base_types::ObjectID;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use std::sync::atomic::{AtomicU64, Ordering};

use super::query::{query_all_table_content, query_object_content};
use crate::cache::{Cache, CACHE_SIZE, CACHE_TTL};
use crate::types::Network;

/// Profile数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: ObjectID,
    pub avatar: String,
    pub rating: u64,
    pub played: u64,
    pub won: u64,
    pub lost: u64,
}

/// 游戏数据管理器
/// 负责管理游戏相关的所有数据，包括但不限于：
/// 1. Profile信息
/// 2. 游戏状态
/// 3. 排行榜数据
/// 等等
pub struct GameManager {
    /// SUI客户端
    client: sui_sdk::SuiClient,
    /// 网络
    network: Network,
    /// Profile缓存
    profile_cache: Arc<RwLock<Cache<ObjectID, Profile>>>,
    /// PassportID到ProfileID的映射
    passport_profile_map: Arc<RwLock<HashMap<ObjectID, ObjectID>>>,
    /// Profile表格ID
    profile_table_id: ObjectID,
    /// 上次更新时间
    last_update: Arc<AtomicU64>,
}

impl GameManager {
    /// 创建新的游戏数据管理器
    pub async fn new(client: sui_sdk::SuiClient, network: Network, manager_store_id: ObjectID) -> Result<Self> {
        // 获取ManagerStore数据
        let store = query_object_content(&network, &manager_store_id).await?;

        // 获取profiles表格ID
        let profile_table_id = store.content["profiles"]["id"]
            .as_str()
            .context("Failed to get profiles table id")?;
        let profile_table_id = ObjectID::from_hex_literal(profile_table_id)
            .context("Failed to parse profiles table id")?;

        Ok(Self {
            client,
            network,
            profile_cache: Arc::new(RwLock::new(Cache::new(CACHE_TTL, CACHE_SIZE))),
            passport_profile_map: Arc::new(RwLock::new(HashMap::new())),
            profile_table_id,
            last_update: Arc::new(AtomicU64::new(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            )),
        })
    }

    /// 获取上次更新时间
    pub fn get_last_update(&self) -> u64 {
        self.last_update.load(Ordering::Relaxed)
    }

    /// 获取Profile数量
    pub async fn get_profile_size(&self) -> Result<u64> {
        // 优先使用映射中的数据
        let map = self.passport_profile_map.read().await;
        let map_size = map.len();
        
        // 如果映射为空，则触发一次完整的数据更新
        if map_size == 0 {
            // 释放读锁
            drop(map);
            // 更新所有profiles数据
            self.update_all_profiles().await?;
            // 重新获取大小
            return Ok(self.passport_profile_map.read().await.len() as u64);
        }
        
        Ok(map_size as u64)
    }

    /// 通过PassportID查询ProfileID
    pub async fn get_profile_id_by_passport(&self, passport_id: &ObjectID) -> Result<ObjectID> {
        // 先检查映射
        if let Some(profile_id) = self.passport_profile_map.read().await.get(passport_id) {
            return Ok(*profile_id);
        }
        info!("get_profile_id_by_passport passport_id: {:?}", passport_id);

        // 查询表格获取所有映射
        let fields = query_all_table_content(&self.network, &self.profile_table_id, None).await?;
        info!("get_profile_id_by_passport fields: {:?}", fields);
        // 更新映射
        let mut map = self.passport_profile_map.write().await;
        for field in fields {
            let passport: ObjectID =
                ObjectID::from_hex_literal(&field.name).context("Failed to parse passport ID")?;
            let profile: ObjectID =
                ObjectID::from_hex_literal(&field.value).context("Failed to parse profile ID")?;
            map.insert(passport, profile);
        }

        // 再次尝试获取
        map.get(passport_id)
            .copied()
            .context("Profile not found for passport")
    }

    /// 获取Profile信息
    pub async fn get_profile(&self, profile_id: &ObjectID) -> Result<Profile> {
        // 先检查缓存
        if let Some(profile) = self.profile_cache.read().await.get(profile_id) {
            return Ok(profile);
        }

        // 查询Profile对象
        let data = query_object_content(&self.network, profile_id).await?;

        // 解析数据
        let profile = Profile {
            id: *profile_id,
            avatar: data.content["avatar"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            rating: data.content["rating"]
                .as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or_default(),
            played: data.content["played"]
                .as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or_default(),
            won: data.content["won"]
                .as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or_default(),
            lost: data.content["lost"]
                .as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or_default(),
        };

        // 更新缓存
        self.profile_cache
            .write()
            .await
            .insert(*profile_id, profile.clone());

        Ok(profile)
    }

    /// 更新所有Profile信息
    pub async fn update_all_profiles(&self) -> Result<()> {
        // 查询表格获取所有映射
        let fields = query_all_table_content(&self.network, &self.profile_table_id, None).await?;
        info!("update_all_profiles fields: {:?}", fields.len());
        // 更新映射
        let mut map = self.passport_profile_map.write().await;
        let profile_cache = self.profile_cache.write().await;

        for field in fields {
            let passport: ObjectID =
                ObjectID::from_hex_literal(&field.name).context("Failed to parse passport ID")?;
            let profile: ObjectID =
                ObjectID::from_hex_literal(&field.value).context("Failed to parse profile ID")?;

            // 更新PassportID到ProfileID的映射
            map.insert(passport, profile);

            // 更新Profile信息
            if let Ok(data) = query_object_content(&self.network, &profile).await {
                let profile_data = Profile {
                    id: profile,
                    avatar: data.content["avatar"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                    rating: data.content["rating"]
                        .as_str()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or_default(),
                    played: data.content["played"]
                        .as_str()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or_default(),
                    won: data.content["won"]
                        .as_str()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or_default(),
                    lost: data.content["lost"]
                        .as_str()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or_default(),
                };
                profile_cache.insert(profile, profile_data);
            }
        }

        // 更新完成后更新时间戳
        self.last_update.store(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            Ordering::Relaxed,
        );

        Ok(())
    }
}
