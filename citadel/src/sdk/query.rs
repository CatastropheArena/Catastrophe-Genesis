use anyhow::{Context, Result};
use serde_json::Value;
use sui_types::base_types::{ObjectID};
use reqwest::Client;
use tracing::{info, warn};
use crate::types::Network;

/// GraphQL客户端封装
#[derive(Debug, Clone)]
pub struct GraphQLClient {
    client: Client,
    url: String,
}

impl GraphQLClient {
    pub fn new(network: &Network) -> Self {
        Self {
            client: Client::new(),
            url: network.graphql_url().to_string(),
        }
    }

    async fn execute_query(&self, query: &str) -> Result<Value> {
        self.client
            .post(&self.url)
            .json(&serde_json::json!({ "query": query }))
            .send()
            .await
            .context("Failed to execute GraphQL query")?
            .json()
            .await
            .context("Failed to parse GraphQL response")
    }
}

/// 对象数据结构
#[derive(Debug, Clone)]
pub struct ObjectData {
    pub address: ObjectID,
    pub content: Value,
}

/// 表格字段数据
#[derive(Debug, Clone)]
pub struct TableField {
    pub name: String,
    pub value: String,
}

/// 表格查询结果
#[derive(Debug, Clone)]
pub struct TableQueryResult {
    pub fields: Vec<TableField>,
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

/// 好友关系查询结果
#[derive(Debug, Clone)]
pub struct RelationshipQueryResult {
    pub user_id: ObjectID,
    pub friend_id: ObjectID,
    pub status: u8,
    pub created_at: u64,
}

/// 查询所有好友关系的结果
#[derive(Debug, Clone)]
pub struct AllRelationshipsQueryResult {
    pub user_id: ObjectID,
    pub relationships: Vec<RelationshipQueryResult>,
}

/// 默认每页大小
const DEFAULT_PAGE_SIZE: u32 = 50;

/// 查询对象内容
pub async fn query_object_content(network: &Network, object_id: &ObjectID) -> Result<ObjectData> {
    let client = GraphQLClient::new(network);
    let query = format!(
        r#"
        query GetObjectContent {{
            object(address: "{}") {{
                address
                asMoveObject {{
                    contents {{
                        json
                    }}
                }}  
            }}
        }}
        "#,
        object_id
    );

    let response = client.execute_query(&query).await?;
    let content = response["data"]["object"]["asMoveObject"]["contents"]["json"].clone();

    Ok(ObjectData {
        address: *object_id,
        content,
    })
}

/// 查询好友关系
pub async fn query_relationship(
    network: &Network,
    friendship_table_id: &ObjectID,
    user_id: &ObjectID,
    friend_id: &ObjectID,
) -> Result<Option<RelationshipQueryResult>> {
    info!("开始查询好友关系...");
    info!("friendship_table_id: {:?}", friendship_table_id);
    info!("user_id: {:?}", user_id);
    info!("friend_id: {:?}", friend_id);
    
    // 使用 query_all_table_content 获取所有表数据
    info!("正在获取外层表的所有数据...");
    let fields = query_all_table_content(network, friendship_table_id, None).await?;
    info!("获取到 {} 个外层表记录", fields.len());
    info!("外层表数据: {:#?}", fields);
    
    // 遍历所有字段，找到目标用户的表
    for field in fields {
        info!("处理外层表记录: name={}", field.name);
        let profile_id = match ObjectID::from_hex_literal(&field.name) {
            Ok(id) => id,
            Err(e) => {
                warn!("解析 profile_id 失败: {}, error: {}", field.name, e);
                continue;
            }
        };
        
        // 如果找到了其中一个用户的表
        if profile_id == *user_id || profile_id == *friend_id {
            info!("找到目标用户的表: profile_id={}", profile_id);
            
            // 解析表信息
            let table_info: serde_json::Value = match serde_json::from_str(&field.value) {
                Ok(info) => info,
                Err(e) => {
                    warn!("解析表信息失败: {}, error: {}", field.value, e);
                    continue;
                }
            };
            info!("表信息: {:#?}", table_info);

            if let Some(table_id) = table_info["id"].as_str() {
                info!("找到内层表ID: {}", table_id);
                
                // 获取该表中的所有关系数据
                let table_object_id = match ObjectID::from_hex_literal(table_id) {
                    Ok(id) => id,
                    Err(e) => {
                        warn!("解析内层表ID失败: {}, error: {}", table_id, e);
                        continue;
                    }
                };
                
                info!("正在获取内层表的关系数据...");
                let relation_fields = query_all_table_content(
                    network, 
                    &table_object_id,
                    None
                ).await?;
                info!("获取到 {} 个关系记录", relation_fields.len());
                info!("关系数据: {:#?}", relation_fields);
                
                // 遍历关系数据
                for relation_field in relation_fields {
                    info!("处理关系记录: name={}", relation_field.name);
                    let target_id = match ObjectID::from_hex_literal(&relation_field.name) {
                        Ok(id) => id,
                        Err(e) => {
                            warn!("解析目标ID失败: {}, error: {}", relation_field.name, e);
                            continue;
                        }
                    };
                    
                    // 检查是否是我们要找的关系
                    if (profile_id == *user_id && target_id == *friend_id) ||
                       (profile_id == *friend_id && target_id == *user_id) {
                        info!("找到目标关系: profile_id={}, target_id={}", profile_id, target_id);
                        
                        // 解析关系数据
                        let relation_data: serde_json::Value = match serde_json::from_str(&relation_field.value) {
                            Ok(data) => data,
                            Err(e) => {
                                warn!("解析关系数据失败: {}, error: {}", relation_field.value, e);
                                continue;
                            }
                        };
                        info!("关系详细数据: {:#?}", relation_data);
                        
                        let result = RelationshipQueryResult {
                            user_id: relation_data["user_id"].as_str()
                                .and_then(|s| ObjectID::from_hex_literal(s).ok())
                                .unwrap_or(profile_id),
                            friend_id: relation_data["friend_id"].as_str()
                                .and_then(|s| ObjectID::from_hex_literal(s).ok())
                                .unwrap_or(target_id),
                            status: relation_data["status"]
                                .as_u64()  // 直接获取数字类型
                                .unwrap_or_default() as u8,
                            created_at: relation_data["created_at"]
                                .as_str()
                                .and_then(|s| s.parse::<u64>().ok())
                                .unwrap_or_default(),
                        };
                        info!("查询完成，返回关系结果: {:#?}", result);
                        return Ok(Some(result));
                    }
                }
            }
        }
    }
    
    info!("未找到目标关系");
    Ok(None)
}

/// 查询表格内容
pub async fn query_table_content(
    network: &Network,
    table_id: &ObjectID,
    cursor: Option<String>,
    page_size: Option<u32>,
) -> Result<TableQueryResult> {
    let client = GraphQLClient::new(network);
    
    // 构建参数字符串
    let mut args = Vec::new();
    
    // 总是添加页面大小，如果未指定则使用默认值
    args.push(format!("first: {}", page_size.unwrap_or(DEFAULT_PAGE_SIZE)));
    
    // 如果有cursor，添加到参数列表
    if let Some(c) = cursor {
        args.push(format!(r#"after: "{}""#, c));
    }
    
    // 将所有参数用逗号连接
    let args_str = args.join(", ");

    let query = format!(
        r#"
        query GetTableContent {{
            owner(address: "{}") {{
                address
                dynamicFields({}) {{
                    pageInfo {{
                        hasNextPage
                        endCursor
                    }}
                    nodes {{
                        name {{ json }}
                        value {{
                            ... on MoveValue {{
                                json
                            }}
                            ... on MoveObject {{
                                contents {{
                                    json
                                }}
                            }}
                        }}
                    }}
                }}
            }}
        }}
        "#,
        table_id, args_str
    );
    info!("query_table_content query: {:?}", query);
    let response = client.execute_query(&query).await?;
    info!("完整的响应数据: {:#?}", response);
    
    let dynamic_fields = &response["data"]["owner"]["dynamicFields"];
    info!("dynamic_fields: {:#?}", dynamic_fields);
    
    let has_next_page = dynamic_fields["pageInfo"]["hasNextPage"]
        .as_bool()
        .unwrap_or(false);

    let end_cursor = dynamic_fields["pageInfo"]["endCursor"]
        .as_str()
        .map(String::from);

    // 创建一个拥有所有权的 Vec 来存储节点数据
    let nodes_array = dynamic_fields["nodes"]
        .as_array()
        .unwrap_or(&Vec::new())
        .to_vec();
    info!("nodes_array: {:#?}", nodes_array);

    let fields = nodes_array
        .iter()
        .map(|node| {
            info!("处理节点: {:#?}", node);
            let name = node["name"]["json"].as_str().unwrap_or_default().to_string();
            info!("节点 name: {}", name);
            
            // 详细记录 value 的结构
            info!("node[\"value\"]: {:#?}", node["value"]);
            info!("node[\"value\"][\"json\"]: {:#?}", node["value"]["json"]);
            info!("node[\"value\"][\"contents\"]: {:#?}", node["value"]["contents"]);
            
            // 处理不同类型的 value
            let value = if let Some(json_value) = node["value"]["json"].as_object() {
                info!("找到 json 对象: {:#?}", json_value);
                serde_json::to_string(json_value).unwrap_or_default()
            } else if let Some(json_value) = node["value"]["contents"]["json"].as_object() {
                info!("找到 contents.json 对象: {:#?}", json_value);
                serde_json::to_string(json_value).unwrap_or_default()
            } else if let Some(json_str) = node["value"]["json"].as_str() {
                info!("找到 json 字符串: {}", json_str);
                json_str.to_string()
            } else {
                info!("尝试直接序列化 json 值");
                serde_json::to_string(&node["value"]["json"]).unwrap_or_default()
            };
            
            info!("最终 value 值: {}", value);
            
            TableField { name, value }
        })
        .collect();

    let result = TableQueryResult {
        fields,
        has_next_page,
        end_cursor,
    };
    info!("查询结果: {:#?}", result);
    
    Ok(result)
}

/// 查询所有表格内容
pub async fn query_all_table_content(
    network: &Network,
    table_id: &ObjectID,
    page_size: Option<u32>,
) -> Result<Vec<TableField>> {
    let mut all_fields = Vec::new();
    let mut cursor = None;
    loop {
        let result = query_table_content(network, table_id, cursor, page_size).await?;
        all_fields.extend(result.fields);

        if !result.has_next_page {
            break;
        }
        cursor = result.end_cursor;
    }

    Ok(all_fields)
}

/// 查询所有好友关系
/// 这个函数会获取relations表中的所有数据，支持分页
pub async fn query_all_relationships(
    network: &Network,
    friendship_table_id: &ObjectID,
) -> Result<Vec<AllRelationshipsQueryResult>> {
    let client = GraphQLClient::new(network);
    
    // 先获取外层表的所有内表ID
    let query = format!(
        r#"
        query GetFriendshipStore {{
            owner(address: "{}") {{
                address
                dynamicFields {{
                    nodes {{
                        name {{ json }}
                        value {{ json }}
                    }}
                }}
            }}
        }}
        "#,
        friendship_table_id
    );
    info!("query_all_relationships: 获取外层表的所有内表ID");
    let response = client.execute_query(&query).await?;
    
    let mut all_relationships = Vec::new();
    
    // 获取所有内表ID
    let nodes = response["data"]["owner"]["dynamicFields"]["nodes"]
        .as_array()
        .unwrap_or(&Vec::new())
        .to_vec();  // 创建一个拥有所有权的副本
    
    // 遍历每个内表
    for node in nodes {
        let user_id = ObjectID::from_hex_literal(
            node["name"]["json"].as_str().unwrap_or_default()
        )?;
        
        // 获取内表的关系数据
        let mut user_relationships = Vec::new();
        
        // 如果内表为空，跳过
        if node["value"]["json"].as_array().map_or(true, |arr| arr.is_empty()) {
            continue;
        }
        
        // 查询内表的具体数据
        let inner_table_query = format!(
            r#"
            query GetInnerTable {{
                owner(address: "{}") {{
                    dynamicFields {{
                        nodes {{
                            name {{ json }}
                            value {{ json }}
                        }}
                    }}
                }}
            }}
            "#,
            user_id
        );
        
        info!("query_all_relationships: 获取用户 {} 的关系数据", user_id);
        if let Ok(inner_response) = client.execute_query(&inner_table_query).await {
            if let Some(inner_nodes) = inner_response["data"]["owner"]["dynamicFields"]["nodes"].as_array() {
                let inner_nodes = inner_nodes.to_vec();  // 创建一个拥有所有权的副本
                for inner_node in inner_nodes {
                    if let (Some(friend_id_str), Some(relation_data)) = (
                        inner_node["name"]["json"].as_str(),
                        inner_node["value"]["json"].as_object()
                    ) {
                        if let Ok(friend_id) = ObjectID::from_hex_literal(friend_id_str) {
                            user_relationships.push(RelationshipQueryResult {
                                user_id,
                                friend_id,
                                status: relation_data["status"].as_u64().unwrap_or_default() as u8,
                                created_at: relation_data["created_at"].as_u64().unwrap_or_default(),
                            });
                        }
                    }
                }
            }
        }
        
        if !user_relationships.is_empty() {
            all_relationships.push(AllRelationshipsQueryResult {
                user_id,
                relationships: user_relationships,
            });
        }
    }
    
    info!("query_all_relationships: 完成查询，共获取 {} 个用户的关系数据", all_relationships.len());
    Ok(all_relationships)
} 