// Copyright (c), Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/**
 * 事务工具模块 (Transaction Builder)
 *
 * 本模块提供了一组事务构建和执行的工具函数，简化了与Sui区块链
 * 交互所需的常见操作。
 */
use anyhow::{Context, Result};
use fastcrypto::encoding::Base64;
use fastcrypto::encoding::Encoding;
use shared_crypto::intent::{Intent, IntentMessage};
use sui_keys::keystore::{AccountKeystore, InMemKeystore};
use sui_sdk::rpc_types::SuiTransactionBlockResponseOptions;
use sui_sdk::SuiClient;
use sui_types::{
    base_types::SuiAddress, crypto::Signature, crypto::SuiKeyPair, transaction::Transaction,
    transaction::TransactionData,
};

/**
 * 从私钥字符串创建内存密钥库
 *
 * 此函数接收sui私钥字符串，解码为SuiKeyPair，
 * 并创建一个包含该密钥对的内存密钥库，同时返回对应的Sui地址。
 *
 * @param sk_sui_privkey - sui私钥字符串
 * @param alias - 密钥别名，可选
 * @return 返回包含密钥库和对应地址的元组
 */
pub fn create_keystore_from_sk(
    sk_base64: &str,
    alias: Option<String>,
) -> Result<(InMemKeystore, SuiKeyPair, SuiAddress)> {
    // 解码私钥,先进行Base64解码
    let sk: String = String::from_utf8(
        Base64::decode(sk_base64).map_err(|e| anyhow::anyhow!("Base64 解码失败: {}", e))?,
    )
    .unwrap();
    // 将解码后的字节转换为字符串
    let keypair =
        SuiKeyPair::decode(&sk).map_err(|e| anyhow::anyhow!("解码 SuiKeyPair 失败: {}", e))?;
    // 创建内存密钥库
    let mut keystore = InMemKeystore::new_insecure_for_tests(0);
    // 使用提供的别名或默认别名
    let key_alias = alias.unwrap_or_else(|| "DefaultKeyPair".to_string());
    // 添加密钥到密钥库
    keystore
        .add_key(Some(key_alias.clone()), keypair.copy())
        .unwrap();
    // 获取对应的地址
    let sender = keystore
        .get_address_by_alias(key_alias)
        .context("从密钥库获取地址失败")?;
    // 获取地址的所有权，避免引用本地变量
    let sender_owned = *sender;
    Ok((keystore, keypair, sender_owned))
}

/**
 * 执行事务
 *
 * 此函数接收事务数据、密钥库和发送者地址，签名并执行事务，
 * 返回事务执行的响应结果。
 *
 * @param client - Sui客户端
 * @param tx_data - 事务数据
 * @param keystore - 密钥库
 * @param sender - 发送者地址
 * @return 返回事务执行响应
 */
pub async fn execute_transaction(
    client: &SuiClient,
    tx_data: TransactionData,
    keystore: &impl AccountKeystore,
    sender: &SuiAddress,
) -> Result<sui_sdk::rpc_types::SuiTransactionBlockResponse> {
    // 签名事务
    let sig = keystore
        .sign_secure(sender, &tx_data, Intent::sui_transaction())
        .context("KeyStore Sign Failed")?;
    let response = client
    .quorum_driver_api()
    .execute_transaction_block(
        Transaction::from_data(tx_data,vec![sig]),
        SuiTransactionBlockResponseOptions::new()
            .with_effects()
            .with_input()
            .with_events()
            .with_object_changes()
            .with_balance_changes(),
        Some(sui_types::quorum_driver_types::ExecuteTransactionRequestType::WaitForLocalExecution),
    )
    .await?;
    Ok(response)
}

/**
 * 使用密钥对直接执行事务
 *
 * 此函数接收事务数据和密钥对，直接签名并执行事务，
 * 无需创建中间的密钥库。
 *
 * @param client - Sui客户端
 * @param tx_data - 事务数据
 * @param keypair - Sui密钥对
 * @return 返回事务执行响应
 */
pub async fn execute_transaction_with_keypair(
    client: &SuiClient,
    tx_data: TransactionData,
    keypair: &SuiKeyPair,
) -> Result<sui_sdk::rpc_types::SuiTransactionBlockResponse> {
    let intent_msg = IntentMessage::new(Intent::sui_transaction(), tx_data.clone());
    let sig = Signature::new_secure(&intent_msg, keypair);
    let response = client
    .quorum_driver_api()
    .execute_transaction_block(
        Transaction::from_data(tx_data,vec![sig]),
        SuiTransactionBlockResponseOptions::new()
            .with_effects()
            .with_input()
            .with_events()
            .with_object_changes()
            .with_balance_changes(),
        Some(sui_types::quorum_driver_types::ExecuteTransactionRequestType::WaitForLocalExecution),
    )
    .await?;
    Ok(response)
}
