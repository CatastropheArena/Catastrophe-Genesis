// Copyright (c), Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::{env, str::FromStr, sync::Arc};
use tracing::{debug, info, warn};

use anyhow::Context;
use sui_sdk::rpc_types::{SuiObjectDataOptions, SuiTransactionBlockResponse};
use sui_types::{
    base_types::{ObjectID, SuiAddress},
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    transaction::TransactionKind,
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
    name: &str,
    avatar: &str,
) -> Result<SuiTransactionBlockResponse> {
    let package_id_str = app_state.citadel_package_id();
    tracing::debug!("使用Citadel包ID: {}", package_id_str);

    // 解析参数
    let package_id = ObjectID::from_hex_literal(&package_id_str).context("无效的包ID格式")?;
    let passport_id = SuiAddress::from_str(passport_id).context("无效的护照ID格式")?;

    // 使用AppState中的SUI客户端
    let sui_client = &app_state.sui_client;

    // 从环境变量获取密钥对并创建密钥库
    let sk = env::var("WALLET_SK").context("未设置WALLET_SK环境变量")?;
    let (keystore, _, sender) = txb::create_keystore_from_sk(&sk, Some("EnvKeyPair".to_string()))?;
    let admin_cap_id = app_state.config["CITADEL_ADMINCAP_ADDRESS"].clone();
    let manager_store_id = app_state.config["CITADEL_MANAGER_ADDRESS"].clone();
    let friendship_store_id = app_state.config["CITADEL_FRIENDSHIP_ADDRESS"].clone();
    // 打印日志
    info!("开始为护照ID: {} 创建用户档案", passport_id);
    // 构建交易
    let mut ptb = ProgrammableTransactionBuilder::new();
    
    // 先创建所有纯值参数
    let arg_manager_store = ptb.pure(manager_store_id).unwrap();
    let arg_passport_id = ptb.pure(passport_id).unwrap();
    let arg_name = ptb.pure(name).unwrap();
    let arg_avatar = ptb.pure(avatar).unwrap();
    let arg_admin_cap = ptb.pure(admin_cap_id).unwrap();
    
    // 然后在 move_call 中使用这些参数
    ptb.programmable_move_call(
        package_id,
        Identifier::new("citadel").unwrap(),
        Identifier::new("create_profile_for_passport").unwrap(),
        vec![],
        vec![
            arg_manager_store,
            arg_passport_id,
            arg_name,
            arg_avatar,
            arg_admin_cap
        ],
    );
    let ptb = ptb.finish();
    let tx_data = app_state
        .sui_client
        .transaction_builder()
        .tx_data(
            sender,
            TransactionKind::ProgrammableTransaction(ptb),
            crate::types::GAS_BUDGET,
            app_state.reference_gas_price(),
            vec![],
            None,
        )
        .await.unwrap();
    // 执行交易
    let response = crate::txb::execute_transaction(sui_client, tx_data, &keystore, &sender)
        .await
        .context("Failed to execute transaction")?;
    Ok(response)
}
