// Copyright (c), Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::{env, str::FromStr, sync::Arc};
use tracing::{debug, info, warn};

use anyhow::Context;
use sui_sdk::{json::SuiJsonValue, rpc_types::{SuiObjectDataOptions, SuiTransactionBlockResponse}};
use serde_json::Value;
use sui_types::{
    base_types::{ObjectID, SuiAddress},
    transaction::{TransactionData, TransactionKind, CallArg, ObjectArg},
    Identifier,
};
use anyhow::Result;

use crate::{app, txb};

/**
 * 为护照创建用户档案
 *
 * 调用Citadel合约中的create_profile_for_passport函数，为指定的护照ID创建用户档案
 *
 * 参数:
 * @param app_state - 应用状态，包含网络配置和SUI客户端
 * @param package_id - Citadel包ID（可选，如果提供则使用该值，否则使用app_state中的最新值）
 * @param passport_id - 护照ID (SuiAddress)
 * @param name - 用户名
 * @param avatar - 头像URL
 * @param private_key - 管理员私钥 (Base64编码)
 *
 * 返回:
 * 交易执行结果，包含创建的档案信息
 */
pub async fn create_profile_for_passport(
    app_state: &Arc<crate::AppState>,
    passport_id: &str,
    avatar: &str,
) -> Result<SuiTransactionBlockResponse> {
    let package_id_str = app_state.citadel_package_id();
    tracing::debug!("使用Citadel包ID: {}", package_id_str);

    // 解析参数
    let package_id = ObjectID::from_hex_literal(&package_id_str).context("无效的包ID格式")?;
    let passport_id = ObjectID::from_hex_literal(&passport_id).context("无效的护照ID格式")?;

    // 使用AppState中的SUI客户端
    let sui_client = &app_state.sui_client;

    // 从环境变量获取密钥对并创建密钥库
    let sk = env::var("WALLET_SK").context("未设置WALLET_SK环境变量")?;
    let (keystore, _, sender) = txb::create_keystore_from_sk(&sk, Some("EnvKeyPair".to_string()))?;
    let admin_cap_id = ObjectID::from_hex_literal(&app_state.config["CITADEL_ADMINCAP_ADDRESS"].clone()).context("无效的admin_cap_id格式")?;
    let manager_store_id = ObjectID::from_hex_literal(&app_state.config["CITADEL_MANAGER_ADDRESS"].clone()).context("无效的manager_store_id格式")?   ;
    let friendship_store_id = ObjectID::from_hex_literal(&app_state.config["CITADEL_FRIENDSHIP_ADDRESS"].clone()).context("无效的friendship_store_id格式")?;
    
    // 打印日志
    info!("开始为护照ID: {} 创建用户档案", passport_id);
    
    // 构建参数列表
    let args = vec![
        SuiJsonValue::from_object_id(manager_store_id),
        SuiJsonValue::from_object_id(friendship_store_id),
        SuiJsonValue::from_object_id(passport_id),
        SuiJsonValue::new(Value::String(avatar.to_string()))?,
        SuiJsonValue::from_object_id(admin_cap_id),
    ];

    // 使用SuiClient的transaction_builder直接构建Move调用交易
    let tx_data = sui_client
        .transaction_builder()
        .move_call(
            sender,
            package_id,
            "citadel",
            "create_profile_for_passport",
            vec![],  // 类型参数为空
            args,
            None,  // gas object参数
            crate::types::GAS_BUDGET,
            None,  // gas price参数
        )
        .await
        .context("构建Move调用交易失败")?;
    
    // 执行交易
    let response = crate::txb::execute_transaction(sui_client, tx_data, &keystore, &sender)
        .await
        .context("执行交易失败")?;
    
    Ok(response)
}
