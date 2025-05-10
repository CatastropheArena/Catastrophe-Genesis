use anyhow::{Context, Result};
use serde_json::Value;
use sui_types::base_types::{ObjectID};
use reqwest::Client;
use tracing::info;
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
    let dynamic_fields = &response["data"]["owner"]["dynamicFields"];
    
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

    let fields = nodes_array
        .iter()
        .map(|node| TableField {
            name: node["name"]["json"].as_str().unwrap_or_default().to_string(),
            value: node["value"]["json"].as_str().unwrap_or_default().to_string(),
        })
        .collect();

    Ok(TableQueryResult {
        fields,
        has_next_page,
        end_cursor,
    })
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