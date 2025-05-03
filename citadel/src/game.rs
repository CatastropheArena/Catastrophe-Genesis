// Copyright (c), Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/**
 * 游戏缓存系统模块
 *
 * 本模块实现了一个专门用于游戏数据的缓存系统，基于LRU缓存策略，具有以下特点：
 * 1. 提供游戏数据的快速存取
 * 2. 支持基于时间的自动过期机制（TTL）
 * 3. 线程安全实现，支持并发访问
 * 4. 支持游戏会话、用户数据和游戏状态的缓存
 *
 * 基于cache.rs模块重新实现，专为游戏数据优化
 */
use crate::externals::current_epoch_time;
use lru::LruCache;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::num::NonZero;
use std::sync::Arc;

/// 游戏缓存前缀常量，用于区分不同类型的游戏数据
pub enum GameCachePrefix {
    MATCH,   // 游戏匹配数据
    LOBBY,   // 游戏大厅数据
    USER,    // 用户数据
    SESSION, // 会话数据
    STATE,   // 游戏状态数据
}

impl GameCachePrefix {
    pub fn as_str(&self) -> &'static str {
        match self {
            GameCachePrefix::MATCH => "match",
            GameCachePrefix::LOBBY => "lobby",
            GameCachePrefix::USER => "user",
            GameCachePrefix::SESSION => "session",
            GameCachePrefix::STATE => "state",
        }
    }
}

/// 游戏缓存默认设置
pub(crate) const GAME_CACHE_SIZE: usize = 10000; // 默认缓存大小
pub(crate) const GAME_CACHE_TTL: u64 = 30 * 60 * 1000; // 30分钟默认过期时间

/**
 * 游戏缓存条目结构
 *
 * 封装缓存中存储的游戏数据及其过期时间
 *
 * 字段:
 * @field value - 缓存的实际数据
 * @field expiry - 条目过期时间（毫秒时间戳）
 */
#[derive(Clone)]
struct GameCacheEntry<V> {
    pub value: V,    // 缓存数据
    pub expiry: u64, // 过期时间戳
}

/**
 * 游戏缓存结构
 *
 * 实现线程安全的LRU缓存，专门用于游戏数据
 *
 * 字段:
 * @field ttl - 缓存条目的生存时间（毫秒）
 * @field cache - 底层LRU缓存，使用互斥锁保护
 */
pub struct GameCache<K, V> {
    ttl: u64,
    cache: Mutex<LruCache<K, GameCacheEntry<V>>>,
}

impl<K: Hash + Eq + Clone, V: Clone> GameCache<K, V> {
    /**
     * 创建新的游戏缓存实例
     *
     * 使用指定的TTL和大小创建缓存
     *
     * 参数:
     * @param ttl - 缓存条目生存时间（毫秒）
     * @param size - 缓存最大条目数
     *
     * 返回:
     * 新创建的游戏缓存实例
     */
    pub fn new(ttl: u64, size: usize) -> Self {
        assert!(size > 0 && ttl > 0, "TTL和大小必须大于0");
        Self {
            ttl,
            cache: Mutex::new(LruCache::new(
                NonZero::new(size).expect("缓存大小必须大于0"),
            )),
        }
    }

    /**
     * 创建默认配置的游戏缓存
     *
     * 使用预定义的默认TTL和大小创建缓存
     *
     * 返回:
     * 默认配置的游戏缓存实例
     */
    pub fn default() -> Self {
        Self::new(GAME_CACHE_TTL, GAME_CACHE_SIZE)
    }

    /**
     * 获取缓存条目
     *
     * 尝试获取与指定键关联的游戏数据
     * 如果数据已过期，则移除并返回None
     *
     * 参数:
     * @param key - 要查找的键
     *
     * 返回:
     * 如果键存在且未过期，则返回关联的游戏数据，否则返回None
     */
    pub fn get(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.lock();
        match cache.get(key) {
            Some(entry) => {
                if entry.expiry < current_epoch_time() {
                    cache.pop(key);
                    None
                } else {
                    Some(entry.value.clone())
                }
            }
            None => None,
        }
    }

    /**
     * 插入或更新缓存条目
     *
     * 将键值对插入缓存，如果键已存在则更新值
     * 计算并设置条目的过期时间
     *
     * 参数:
     * @param key - 要插入的键
     * @param value - 要存储的游戏数据
     */
    pub fn set(&self, key: K, value: V) {
        let mut cache = self.cache.lock();
        cache.put(
            key,
            GameCacheEntry {
                value,
                expiry: current_epoch_time() + self.ttl,
            },
        );
    }

    /**
     * 更新缓存条目
     *
     * 更新现有缓存条目的部分数据
     * 如果键不存在，则不执行任何操作并返回false
     *
     * 参数:
     * @param key - 要更新的键
     * @param update_fn - 更新函数，接收当前值并返回更新后的值
     *
     * 返回:
     * 更新成功返回true，键不存在返回false
     */
    pub fn update<F>(&self, key: &K, update_fn: F) -> bool
    where
        F: FnOnce(V) -> V,
    {
        let mut cache = self.cache.lock();
        if let Some(entry) = cache.get(key) {
            if entry.expiry < current_epoch_time() {
                cache.pop(key);
                return false;
            }

            let updated_value = update_fn(entry.value.clone());
            cache.put(
                key.clone(),
                GameCacheEntry {
                    value: updated_value,
                    expiry: current_epoch_time() + self.ttl,
                },
            );
            true
        } else {
            false
        }
    }

    /**
     * 删除缓存条目
     *
     * 从缓存中移除指定键的条目
     *
     * 参数:
     * @param key - 要删除的键
     *
     * 返回:
     * 如果键存在并被删除返回true，否则返回false
     */
    pub fn delete(&self, key: &K) -> bool {
        let mut cache = self.cache.lock();
        cache.pop(key).is_some()
    }
}

/**
 * 游戏服务结构
 *
 * 提供游戏相关操作的高级接口，内部使用GameCache进行数据缓存
 */
pub struct GameService {
    cache: Arc<GameCache<String, String>>,
}

impl GameService {
    /**
     * 创建新的游戏服务实例
     *
     * 返回:
     * 新创建的游戏服务实例
     */
    pub fn new() -> Self {
        Self {
            cache: Arc::new(GameCache::default()),
        }
    }

    /**
     * 获取游戏数据
     *
     * 尝试获取指定键的游戏数据，并解析为指定类型
     *
     * 参数:
     * @param prefix - 数据类型前缀
     * @param key - 数据键
     *
     * 返回:
     * 成功返回解析后的数据，否则返回None
     */
    pub fn get<T: for<'de> Deserialize<'de>>(
        &self,
        prefix: GameCachePrefix,
        key: &str,
    ) -> Option<T> {
        let prefixed_key = format!("{}:{}", prefix.as_str(), key);
        self.cache
            .get(&prefixed_key)
            .and_then(|json| serde_json::from_str(&json).ok())
    }

    /**
     * 设置游戏数据
     *
     * 将指定数据序列化并存储到缓存中
     *
     * 参数:
     * @param prefix - 数据类型前缀
     * @param key - 数据键
     * @param value - 要存储的数据
     *
     * 返回:
     * 成功返回true，失败返回false
     */
    pub fn set<T: Serialize>(&self, prefix: GameCachePrefix, key: &str, value: &T) -> bool {
        let prefixed_key = format!("{}:{}", prefix.as_str(), key);
        match serde_json::to_string(value) {
            Ok(json) => {
                self.cache.set(prefixed_key, json);
                true
            }
            Err(_) => false,
        }
    }

    /**
     * 更新游戏数据
     *
     * 更新现有数据的部分字段
     *
     * 参数:
     * @param prefix - 数据类型前缀
     * @param key - 数据键
     * @param partial - 要更新的部分数据
     *
     * 返回:
     * 成功返回true，失败返回false
     */
    pub fn update<T: for<'de> Deserialize<'de> + Serialize>(
        &self,
        prefix: GameCachePrefix,
        key: &str,
        partial: &T,
    ) -> bool {
        let prefixed_key = format!("{}:{}", prefix.as_str(), key);
        self.cache.update(&prefixed_key, |json| {
            if let Ok(mut value) = serde_json::from_str::<serde_json::Value>(&json) {
                if let Ok(partial_value) = serde_json::to_value(partial) {
                    if let Some(obj) = value.as_object_mut() {
                        if let Some(partial_obj) = partial_value.as_object() {
                            for (k, v) in partial_obj {
                                obj.insert(k.clone(), v.clone());
                            }
                        }
                    }
                }
                serde_json::to_string(&value).unwrap_or(json)
            } else {
                json
            }
        })
    }

    /**
     * 删除游戏数据
     *
     * 从缓存中删除指定键的数据
     *
     * 参数:
     * @param prefix - 数据类型前缀
     * @param key - 数据键
     *
     * 返回:
     * 成功返回true，失败返回false
     */
    pub fn delete(&self, prefix: GameCachePrefix, key: &str) -> bool {
        let prefixed_key = format!("{}:{}", prefix.as_str(), key);
        self.cache.delete(&prefixed_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::thread::sleep;
    use std::time::Duration;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestUser {
        id: String,
        name: String,
        score: u32,
    }

    #[test]
    fn test_game_cache_basic() {
        let cache = GameCache::<String, String>::default();
        let key = "test_key".to_string();
        let value = "test_value".to_string();

        cache.set(key.clone(), value.clone());
        assert_eq!(cache.get(&key), Some(value));
    }

    #[test]
    fn test_game_cache_expiry() {
        let cache = GameCache::new(100, 10);
        let key = "test_key".to_string();
        let value = "test_value".to_string();

        cache.set(key.clone(), value);
        sleep(Duration::from_millis(200));
        assert_eq!(cache.get(&key), None);
    }

    #[test]
    fn test_game_cache_update() {
        let cache = GameCache::<String, String>::default();
        let key = "test_key".to_string();
        let value = "test_value".to_string();

        cache.set(key.clone(), value);
        let updated = cache.update(&key, |v| format!("{}_updated", v));
        assert!(updated);
        assert_eq!(cache.get(&key), Some("test_value_updated".to_string()));
    }

    #[test]
    fn test_game_service() {
        let service = GameService::new();
        let user = TestUser {
            id: "1".to_string(),
            name: "玩家1".to_string(),
            score: 100,
        };

        // 测试设置和获取
        assert!(service.set(GameCachePrefix::USER, "1", &user));
        let retrieved: Option<TestUser> = service.get(GameCachePrefix::USER, "1");
        assert_eq!(retrieved, Some(user));

        // 测试更新
        let partial = serde_json::json!({"score": 200});
        assert!(service.update(GameCachePrefix::USER, "1", &partial));
        let updated: Option<TestUser> = service.get(GameCachePrefix::USER, "1");
        assert_eq!(updated.unwrap().score, 200);

        // 测试删除
        assert!(service.delete(GameCachePrefix::USER, "1"));
        let deleted: Option<TestUser> = service.get(GameCachePrefix::USER, "1");
        assert_eq!(deleted, None);
    }
}
