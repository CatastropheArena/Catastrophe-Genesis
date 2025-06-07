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

/// 好友关系状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelationshipStatus {
    /// 待确认
    Pending = 1,
    /// 已接受
    Friends = 2,
}

/// 好友关系数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub initiator: ObjectID,
    pub receiver: ObjectID,
    pub status: RelationshipStatus,
    pub created_at: u64,
}

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

/// Profile详细信息(包含关系)
#[derive(Debug, Clone, Serialize)]
pub struct ProfileWithRelationship {
    #[serde(flatten)]
    pub profile: Profile,
    pub relationship: Option<Relationship>,
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
    /// 好友关系缓存
    relationship_cache: Arc<RwLock<Cache<(ObjectID, ObjectID), Relationship>>>,
    /// PassportID到ProfileID的映射
    passport_profile_map: Arc<RwLock<HashMap<ObjectID, ObjectID>>>,
    /// Profile表格ID
    profile_table_id: ObjectID,
    /// 好友关系存储ID
    friendship_table_id: ObjectID,
    /// Profile上次更新时间
    last_profile_update: Arc<AtomicU64>,
    /// 关系上次更新时间
    last_relationship_update: Arc<AtomicU64>,
}

impl GameManager {
    /// 创建新的游戏数据管理器
    pub async fn new(client: sui_sdk::SuiClient, network: Network, manager_store_id: ObjectID,friendship_store_id: ObjectID) -> Result<Self> {
        let profile_table_id = match network {
            #[cfg(test)]
            Network::TestCluster => ObjectID::ZERO, // 在测试环境中使用一个固定的ID
            _ => {
                let store = query_object_content(&network, &manager_store_id).await?;
                // 获取profiles表格ID
                let profile_table_id = store.content["profiles"]["id"]
                    .as_str()
                    .context("Failed to get profiles table id")?;
                ObjectID::from_hex_literal(profile_table_id)
                    .context("Failed to parse profiles table id")?
            }
        };
        let friendship_table_id = match network {
            #[cfg(test)]
            Network::TestCluster => ObjectID::ZERO, // 在测试环境中使用一个固定的ID
            _ => {
                let store = query_object_content(&network, &friendship_store_id).await?;
                // 获取profiles表格ID
                let friendship_table_id = store.content["relations"]["id"]
                    .as_str()
                    .context("Failed to get friendship table id")?;
                ObjectID::from_hex_literal(friendship_table_id)
                    .context("Failed to parse friendship table id")?
            }
        };

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(Self {
            client,
            network,
            profile_cache: Arc::new(RwLock::new(Cache::new(CACHE_TTL, CACHE_SIZE))),
            relationship_cache: Arc::new(RwLock::new(Cache::new(CACHE_TTL, CACHE_SIZE))),
            passport_profile_map: Arc::new(RwLock::new(HashMap::new())),
            profile_table_id,
            friendship_table_id,
            last_profile_update: Arc::new(AtomicU64::new(current_time)),
            last_relationship_update: Arc::new(AtomicU64::new(current_time)),
        })
    }

    /// 获取Profile上次更新时间
    pub fn get_last_profile_update(&self) -> u64 {
        self.last_profile_update.load(Ordering::Relaxed)
    }

    /// 获取关系上次更新时间
    pub fn get_last_relationship_update(&self) -> u64 {
        self.last_relationship_update.load(Ordering::Relaxed)
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
        match self.network {
            #[cfg(test)]
            Network::TestCluster => {
                // 在测试环境中，直接返回一个固定的 ProfileID
                Ok(ObjectID::ZERO)
            }
            _ => {
                // 先检查映射
                if let Some(profile_id) = self.passport_profile_map.read().await.get(passport_id) {
                    return Ok(*profile_id);
                }
                info!("get_profile_id_by_passport passport_id: {:?}", passport_id);

                // 更新所有profiles数据
                self.update_all_profiles().await?;

                // 再次尝试获取
                self.passport_profile_map
                    .read()
                    .await
                    .get(passport_id)
                    .copied()
                    .context("Profile not found for passport")
            }
        }
    }

    /// 获取Profile信息
    pub async fn get_profile(&self, profile_id: &ObjectID) -> Result<Profile> {
        match self.network {
            #[cfg(test)]
            Network::TestCluster => {
                // 在测试环境中返回一个模拟的 Profile
                Ok(Profile {
                    id: *profile_id,
                    avatar: "test_avatar".to_string(),
                    rating: 1000,
                    played: 0,
                    won: 0,
                    lost: 0,
                })
            }
            _ => {
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
        }
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
        self.last_profile_update.store(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            Ordering::Relaxed,
        );

        Ok(())
    }

    /// 更新PassportID到ProfileID的映射缓存
    pub async fn update_passport_profile_mapping(&self, passport_id: ObjectID, profile_id: ObjectID) {
        let mut map = self.passport_profile_map.write().await;
        map.insert(passport_id, profile_id);
    }

    /// 更新Profile缓存
    pub async fn update_profile_cache(&self, profile: Profile) {
        let mut cache = self.profile_cache.write().await;
        cache.insert(profile.id, profile);
    }

    /// 获取带关系信息的Profile
    pub async fn get_profile_with_relationship(
        &self,
        profile_id: &ObjectID,
        current_user_id: Option<&ObjectID>,
    ) -> Result<ProfileWithRelationship> {
        // 获取基础Profile信息
        let profile = self.get_profile(profile_id).await?;
        
        // 如果提供了当前用户ID,则查询关系
        let relationship = if let Some(user_id) = current_user_id {
            self.get_relationship(user_id, profile_id).await?
        } else {
            None
        };

        Ok(ProfileWithRelationship {
            profile,
            relationship,
        })
    }


    /// 获取用户关系
    pub async fn get_relationship(&self, a: &ObjectID, b: &ObjectID) -> Result<Option<Relationship>> {
        // 先检查缓存
        let cache_key = (*a, *b);
        if let Some(relationship) = self.relationship_cache.read().await.get(&cache_key) {
            return Ok(Some(relationship.clone()));
        }

        // 缓存未命中，执行全量更新
        debug!("关系缓存未命中，执行全量更新 a: {:?}, b: {:?}", a, b);
        self.update_all_relationships().await?;

        // 再次尝试从缓存获取
        Ok(self.relationship_cache.read().await.get(&cache_key).map(|r| r.clone()))
    }


    /// 获取关系缓存大小
    pub async fn get_relationship_cache_size(&self) -> u64 {
        self.relationship_cache.read().await.len() as u64
    }

    /// 更新所有好友关系缓存
    pub async fn update_all_relationships(&self) -> Result<()> {
        info!("开始更新所有好友关系缓存");
        
        // 查询所有好友关系
        let fields = query_all_table_content(&self.network, &self.friendship_table_id, None).await?;
        info!("获取到 {} 个关系记录", fields.len());
        
        // 获取缓存写锁
        let mut cache = self.relationship_cache.write().await;
        
        // 清空现有缓存
        cache.clear();
        
        // 更新缓存
        for field in fields {
            // 解析关系键
            let key: serde_json::Value = serde_json::from_str(&field.name)?;
            let a = ObjectID::from_hex_literal(key["a"].as_str().unwrap_or_default())?;
            let b = ObjectID::from_hex_literal(key["b"].as_str().unwrap_or_default())?;
            
            // 解析关系数据
            let relation_data: serde_json::Value = serde_json::from_str(&field.value)?;
            let from_a = relation_data["from_a"].as_bool().unwrap_or_default();
            
            // 根据 from_a 确定发起者和接收者
            let (initiator, receiver) = if from_a {
                (a, b)
            } else {
                (b, a)
            };
            
            let relationship = Relationship {
                initiator,
                receiver,
                status: match relation_data["status"].as_u64().unwrap_or_default() as u8 {
                    1 => RelationshipStatus::Pending,
                    2 => RelationshipStatus::Friends,
                    _ => continue,
                },
                created_at: relation_data["created_at"]
                    .as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or_default(),
            };
            
            // 双向缓存关系
            cache.insert((initiator, receiver), relationship.clone());
            cache.insert((receiver, initiator), relationship);
        }
        
        // 更新完成后更新时间戳
        self.last_relationship_update.store(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            Ordering::Relaxed,
        );
        
        info!("好友关系缓存更新完成，共更新 {} 条记录", cache.len());
        Ok(())
    }

}
