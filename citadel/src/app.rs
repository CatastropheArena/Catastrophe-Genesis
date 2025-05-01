// Copyright (c), Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::common::IntentMessage;
use crate::common::{to_signed_response, IntentScope, ProcessDataRequest, ProcessedDataResponse};
use crate::AppState;
use crate::EnclaveError;
use axum::extract::State;
use axum::Json;
use fastcrypto::encoding::{Encoding, Hex};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, error, debug};
use crypto_hash::{hex_digest, Algorithm};
use fastcrypto::traits::Signer;
use fastcrypto::ed25519::{Ed25519KeyPair, Ed25519Signature};
/// ====
/// Core Nautilus server logic, replace it with your own
/// relavant structs and process_data endpoint.
/// ====

/// Inner type for IntentMessage<T>
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserData {
    pub twitter_name: Vec<u8>,
    pub sui_address: Vec<u8>,
}

/// Inner type for ProcessDataRequest<T>
#[derive(Debug, Serialize, Deserialize)]
pub struct UserRequest {
    pub user_url: String,
}

pub async fn process_data(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ProcessDataRequest<UserRequest>>,
) -> Result<Json<ProcessedDataResponse<IntentMessage<UserData>>>, EnclaveError> {
    let user_url = request.payload.user_url.clone();
    info!("Processing user URL: {}", user_url);
    state.metrics.process_data_requests.inc();

    // Check API key
    if state.api_key.is_empty() {
        error!("Twitter API key is empty");
        return Err(EnclaveError::GenericError("Twitter API key not configured".to_string()));
    }
    
    // Calculate MD5 of API key
    let api_key_md5 = hex_digest(Algorithm::MD5, state.api_key.as_bytes());
    debug!("API key length: {}, MD5: {}", state.api_key.len(), api_key_md5);

    let current_timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| {
            error!("Failed to get current timestamp: {}", e);
            EnclaveError::GenericError(format!("Failed to get current timestamp: {}, API key MD5: {}", e, api_key_md5))
        })?
        .as_millis() as u64;
    
    debug!("Current timestamp: {}", current_timestamp);
    
    // Fetch tweet content
    info!("Starting to fetch Twitter content, URL: {}", user_url);
    let (twitter_name, sui_address) = match fetch_tweet_content(&state.api_key, &user_url).await {
        Ok(result) => {
            info!("Successfully retrieved data for user {}, SUI address length: {}", result.0, result.1.len());
            result
        },
        Err(e) => {
            error!("Failed to fetch Twitter content: {:?}, API key MD5: {}", e, api_key_md5);
            return Err(EnclaveError::GenericError(format!("{:?} (API key MD5: {})", e, api_key_md5)));
        }
    };
    
    info!("Preparing to generate signed response");
    let response = to_signed_response(
        &state.eph_kp,
        UserData {
            twitter_name: twitter_name.as_bytes().to_vec(),
            sui_address: sui_address.clone(),
        },
        current_timestamp,
        IntentScope::Tweet,
    );
    
    info!("Request processing completed");
    Ok(Json(response))
}

async fn fetch_tweet_content(
    api_key: &str,
    user_url: &str,
) -> Result<(String, Vec<u8>), EnclaveError> {
    debug!("Creating HTTP client");
    let client = reqwest::Client::new();
    
    let api_key_md5 = hex_digest(Algorithm::MD5, api_key.as_bytes());
    
    if user_url.contains("/status/") {
        info!("Detected status tweet URL: {}", user_url);
        // Extract tweet ID from URL using regex
        let re = Regex::new(r"x\.com/\w+/status/(\d+)")
            .map_err(|e| {
                error!("Failed to create Twitter URL regex: {}", e);
                EnclaveError::GenericError(format!("Invalid Twitter URL format: {}, API key MD5: {}", e, api_key_md5))
            })?;
        
        let tweet_id = re
            .captures(user_url)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str())
            .ok_or_else(|| {
                error!("Unable to extract tweet ID from URL: {}", user_url);
                EnclaveError::GenericError(format!("Invalid Twitter URL: {}, API key MD5: {}", user_url, api_key_md5))
            })?;
        
        info!("Extracted tweet ID: {}", tweet_id);

        // Construct the Twitter API URL
        let url = format!(
            "https://api.twitter.com/2/tweets/{}?expansions=author_id&user.fields=username",
            tweet_id
        );
        
        info!("Requesting Twitter API: {}", url);
        debug!("Authorization header length: {}", format!("Bearer {}", api_key).len());

        // Make the request to Twitter API
        let response_result = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await;
            
        let response = match response_result {
            Ok(resp) => {
                info!("Received Twitter API response: status code {}", resp.status());
                debug!("Response headers: {:?}", resp.headers());
                
                match resp.json::<serde_json::Value>().await {
                    Ok(json) => {
                        debug!("Twitter API response content: {}", json);
                        json
                    },
                    Err(e) => {
                        error!("Failed to parse Twitter API response: {}", e);
                        return Err(EnclaveError::GenericError(format!("Failed to parse Twitter API response: {}, API key MD5: {}", e, api_key_md5)));
                    }
                }
            },
            Err(e) => {
                error!("Failed to send request to Twitter API: {}", e);
                return Err(EnclaveError::GenericError(format!("Failed to send request to Twitter API: {}, API key MD5: {}", e, api_key_md5)));
            }
        };

        // Extract tweet text and author username
        let tweet_text = match response.get("data").and_then(|data| data.get("text")).and_then(|text| text.as_str()) {
            Some(text) => {
                info!("Got tweet content: {}", text);
                text
            },
            None => {
                error!("Unable to extract tweet content from response: {}", response);
                return Err(EnclaveError::GenericError(format!("Unable to extract tweet content: {}, API key MD5: {}", response, api_key_md5)));
            }
        };

        let twitter_name = match response.get("includes")
            .and_then(|includes| includes.get("users"))
            .and_then(|users| users.as_array())
            .and_then(|users| users.first())
            .and_then(|user| user.get("username"))
            .and_then(|username| username.as_str()) {
                Some(name) => {
                    info!("Got username: {}", name);
                    name
                },
                None => {
                    error!("Unable to extract username from response: {}", response);
                    return Err(EnclaveError::GenericError(format!("Unable to extract username, API key MD5: {}", api_key_md5)));
                }
            };

        // Find the position of "#SUI" and extract address before it
        let sui_tag_pos = match tweet_text.find("#SUI") {
            Some(pos) => {
                info!("Found #SUI tag in tweet, position: {}", pos);
                pos
            },
            None => {
                error!("No #SUI tag found in tweet: {}", tweet_text);
                return Err(EnclaveError::GenericError(format!("No #SUI tag found in tweet, API key MD5: {}", api_key_md5)));
            }
        };

        let text_before_tag = &tweet_text[..sui_tag_pos];
        debug!("Text before tag: {}", text_before_tag);
        
        let sui_address_re = Regex::new(r"0x[0-9a-fA-F]{64}")
            .map_err(|e| {
                error!("Failed to create SUI address regex: {}", e);
                EnclaveError::GenericError(format!("Invalid SUI address regex: {}, API key MD5: {}", e, api_key_md5))
            })?;

        let sui_address = match sui_address_re.find(text_before_tag) {
            Some(m) => {
                let addr = m.as_str();
                info!("Found SUI address in tweet: {}", addr);
                addr
            },
            None => {
                error!("No valid SUI address found before tag: {}", text_before_tag);
                return Err(EnclaveError::GenericError(format!("No valid SUI address found before tag, API key MD5: {}", api_key_md5)));
            }
        };

        info!("Decoding SUI address");
        let decoded_address = match Hex::decode(sui_address) {
            Ok(addr) => {
                debug!("Decoded address length: {}", addr.len());
                addr
            },
            Err(e) => {
                error!("Failed to decode SUI address: {}", e);
                return Err(EnclaveError::GenericError(format!("Invalid SUI address: {}, API key MD5: {}", e, api_key_md5)));
            }
        };

        Ok((twitter_name.to_string(), decoded_address))
    } else {
        info!("Detected profile URL: {}", user_url);
        // Handle profile URL
        let re = Regex::new(r"x\.com/(\w+)(?:/)?$")
            .map_err(|e| {
                error!("Failed to create profile URL regex: {}", e);
                EnclaveError::GenericError(format!("Invalid profile URL format: {}, API key MD5: {}", e, api_key_md5))
            })?;
            
        let username = re
            .captures(user_url)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str())
            .ok_or_else(|| {
                error!("Unable to extract username from URL: {}", user_url);
                EnclaveError::GenericError(format!("Invalid profile URL: {}, API key MD5: {}", user_url, api_key_md5))
            })?;
            
        info!("Extracted username: {}", username);

        // Fetch user profile
        let url = format!(
            "https://api.twitter.com/2/users/by/username/{}?user.fields=description",
            username
        );
        
        info!("Requesting Twitter API: {}", url);

        let response_result = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await;
            
        let response = match response_result {
            Ok(resp) => {
                info!("Received Twitter API response: status code {}", resp.status());
                debug!("Response headers: {:?}", resp.headers());
                
                match resp.json::<serde_json::Value>().await {
                    Ok(json) => {
                        debug!("Twitter API response content: {}", json);
                        json
                    },
                    Err(e) => {
                        error!("Failed to parse Twitter API response: {}", e);
                        return Err(EnclaveError::GenericError(format!("Failed to parse Twitter API response: {}, API key MD5: {}", e, api_key_md5)));
                    }
                }
            },
            Err(e) => {
                error!("Failed to send request to Twitter API: {}", e);
                return Err(EnclaveError::GenericError(format!("Failed to send request to Twitter API: {}, API key MD5: {}", e, api_key_md5)));
            }
        };

        // Extract user description
        let description = match response.get("data").and_then(|data| data.get("description")).and_then(|desc| desc.as_str()) {
            Some(desc) => {
                info!("Got user description: {}", desc);
                desc
            },
            None => {
                error!("Unable to extract user description from response: {}", response);
                return Err(EnclaveError::GenericError(format!("Unable to extract user description: {}, API key MD5: {}", response, api_key_md5)));
            }
        };

        let sui_tag_pos = match description.find("#SUI") {
            Some(pos) => {
                info!("Found #SUI tag in user description, position: {}", pos);
                pos
            },
            None => {
                error!("No #SUI tag found in user description: {}", description);
                return Err(EnclaveError::GenericError(format!("No #SUI tag found in user description, API key MD5: {}", api_key_md5)));
            }
        };

        let text_before_tag = &description[..sui_tag_pos];
        debug!("Text before tag: {}", text_before_tag);
        
        let sui_address_re = Regex::new(r"0x[0-9a-fA-F]{64}")
            .map_err(|e| {
                error!("Failed to create SUI address regex: {}", e);
                EnclaveError::GenericError(format!("Invalid SUI address regex: {}, API key MD5: {}", e, api_key_md5))
            })?;

        let sui_address = match sui_address_re.find(text_before_tag) {
            Some(m) => {
                let addr = m.as_str();
                info!("Found SUI address in user description: {}", addr);
                addr
            },
            None => {
                error!("No valid SUI address found before tag: {}", text_before_tag);
                return Err(EnclaveError::GenericError(format!("No valid SUI address found before tag, API key MD5: {}", api_key_md5)));
            }
        };

        info!("Decoding SUI address (skipping '0x' prefix)");
        let decoded_address = match Hex::decode(&sui_address[2..]) {
            Ok(addr) => {
                debug!("Decoded address length: {}", addr.len());
                addr
            },
            Err(e) => {
                error!("Failed to decode SUI address: {}", e);
                return Err(EnclaveError::GenericError(format!("Invalid SUI address: {}, API key MD5: {}", e, api_key_md5)));
            }
        };

        Ok((username.to_string(), decoded_address))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use fastcrypto::traits::{Signer, VerifyingKey, ToFromBytes};
    use fastcrypto::ed25519::{Ed25519KeyPair, Ed25519Signature};
    use fastcrypto::traits::KeyPair;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[tokio::test]
    async fn test_serde() {
        // serialization should be consistent with move test see `fun test_serde` in `enclave.move`.
        use crate::common::IntentMessage;
        let intent_msg = IntentMessage::new(
            UserData {
                twitter_name: "luo_eurax".as_bytes().to_vec(),
                sui_address: Hex::decode(
                    "0x540ba39b0328acd14e100a8af76b7880e336abe08f806ada5643085794bd8aab",
                )
                .unwrap(),
            },
            1745930369515,
            IntentScope::Tweet,
        );
        let signing_payload = bcs::to_bytes(&intent_msg).expect("should not fail");
        let expected_payload = signing_payload.clone();
        println!("signing_payload: {}", Hex::encode(expected_payload));
        
        // 这是从测试输出中获取的实际序列化值
        let expected_serialized = "00eb398f8196010000096c756f5f657572617820540ba39b0328acd14e100a8af76b7880e336abe08f806ada5643085794bd8aab";
        assert_eq!(Hex::encode(&signing_payload), expected_serialized, 
                  "序列化数据与预期不匹配\n实际: {}\n预期: {}", 
                  Hex::encode(&signing_payload), expected_serialized);
    }

    #[tokio::test]
    async fn test_verify_signature() {
        // 测试用户通过curl获取的签名能否验证
        use crate::common::IntentMessage;
        
        let seed = 42u64; // 使用固定的u64值作为种子
        let mut rand = StdRng::seed_from_u64(seed);
        let kp = Ed25519KeyPair::generate(&mut rand);
        // 创建与服务器相同的密钥对
        
        // 创建与test_serde测试相同的数据
        let intent_msg = IntentMessage::new(
            UserData {
                twitter_name: "luo_eurax".as_bytes().to_vec(),
                sui_address: Hex::decode(
                    "0x540ba39b0328acd14e100a8af76b7880e336abe08f806ada5643085794bd8aab",
                )
                .unwrap(),
            },
            1745930369515,
            IntentScope::Tweet,
        );
        
        // 序列化数据
        let signing_payload = bcs::to_bytes(&intent_msg).expect("should not fail");
        println!("序列化数据: {}", Hex::encode(&signing_payload));
        
        // 用户通过curl获取的签名
        let user_signature_bytes = Hex::decode(
            "e74cf6d0843a92a0e571ceafb16a9536928166ccd9f6638ef4e0a605df7e3d6deb6a82726188b795da0665305dea6de188bee3c35c37370ab9b9934646908303"
        ).unwrap();
        
        // 将Vec<u8>转换为Ed25519Signature
        let user_signature = Ed25519Signature::from_bytes(&user_signature_bytes)
            .expect("无法从字节数组创建签名对象");
        
        // 验证签名
        let pk = kp.public();
        let verified = pk.verify(&signing_payload, &user_signature);
        
        // 如果验证失败，输出更多信息用于调试
        if verified.is_err() {
            // 手动签名数据，看看生成的签名是什么
            let our_signature: Ed25519Signature = kp.sign(&signing_payload);
            println!("我们生成的签名: {}", Hex::encode(our_signature.as_ref()));
            println!("用户收到的签名: {}", Hex::encode(&user_signature_bytes));
            
            // 使用我们签名生成的签名验证
            let self_verify = pk.verify(&signing_payload, &our_signature);
            println!("自签名验证结果: {:?}", self_verify);
        }
        
        // 断言签名验证成功
        assert!(verified.is_ok(), "签名验证失败：{:?}", verified.err());
    }
}
