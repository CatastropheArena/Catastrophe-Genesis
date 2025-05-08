// Copyright (c), Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/**
 * Seal命令行接口 (Seal CLI)
 * 
 * 本模块实现了Seal密钥管理系统的命令行界面，提供了一套完整的工具来管理
 * 与Seal密码系统交互所需的密钥和加密操作。通过此CLI，用户可以：
 * 
 * - 生成IBE密钥对
 * - 提取用户私钥
 * - 验证用户私钥
 * - 使用Seal进行加密和解密操作
 * - 解析和查看加密对象的结构
 * - 发布Move模块
 * - 注册密钥服务器
 */

use clap::{Parser, Subcommand};
use crypto::dem::{Aes256Gcm, Hmac256Ctr};
use crypto::EncryptionInput::Plain;
use crypto::{
    create_full_id, ibe, seal_decrypt, seal_encrypt, Ciphertext, EncryptedObject, EncryptionInput,
    IBEEncryptions, IBEPublicKeys, IBEUserSecretKeys, ObjectID,
};
use fastcrypto::encoding::Encoding;
use fastcrypto::encoding::{Base64, Hex};
use fastcrypto::error::{FastCryptoError, FastCryptoResult};
use fastcrypto::groups::bls12381::{G1Element, G2Element, Scalar};
use fastcrypto::serde_helpers::ToFromByteArray;
// use fastcrypto::si
use rand::thread_rng;
use serde::Deserialize;
use serde::Serialize;
use sui_sdk::rpc_types::SuiTransactionBlockResponseOptions;
use sui_sdk::SuiClientBuilder;
use sui_types::object::Owner;
use sui_types::transaction::Transaction;
use tracing::info;
use std::env;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::str::FromStr;
use sui_sdk::rpc_types::{ObjectChange, SuiData, SuiObjectDataOptions};

use sui_types::base_types::SuiAddress;
use sui_types::crypto::{SuiKeyPair, KeypairTraits};
// 导入所需的依赖
use anyhow::Context;
use std::path::Path;
use sui_move_build::BuildConfig;
use sui_sdk::SuiClient;
use sui_sdk::wallet_context::WalletContext;
use dotenv::dotenv;
pub use fastcrypto::traits::Signer;
pub use fastcrypto::traits::{
    AggregateAuthenticator, Authenticator, EncodeDecodeBase64, SigningKey, ToFromBytes,
    VerifyingKey,
};
use sui_keys::keystore::{AccountKeystore,Keystore,InMemKeystore};
use shared_crypto::intent::{Intent, IntentMessage};
use crate::AppState;
use serde_json::json;
use sui_sdk::json::SuiJsonValue;

// 导入txb模块
use crate::txb;

/// 密钥长度常量（字节）
const KEY_LENGTH: usize = 32;

/// 默认编码方式，用于序列化和反序列化值
type DefaultEncoding = Hex;

/**
 * CLI参数结构体
 * 
 * 使用clap库定义的命令行参数结构，包含所有可能的命令和选项
 */
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    #[command(subcommand)]
    command: Command,
}

/**
 * CLI支持的命令枚举
 * 
 * 定义了所有可用的子命令及其各自的参数
 * 
 * >>> cargo run cli register-key-server -p 0x73df4c06b9b9d4a165bf61a66225cc197d8c7b82dd490bf704ae18937d023186 -d 本地调试 -u http://localhost:3000 -k ae4f0608b74840bc0bd928047ce5029553374c071fd7887944858e376308cda4a648093557e9193bf3f8daddd7e7a42013db21156f7fb91cc08ee336b7c9dd8e076d6937eb09847113c28193d9e1790df568a93572a9a81cc611db121cf89473
 */
#[derive(Subcommand, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Command {
    /// 生成新的主密钥和公钥对
    /// 
    /// 此命令创建一个新的Boneh-Franklin IBE主密钥对，包括一个随机生成的
    /// 主密钥（私钥）和对应的公钥。主密钥应保密存储，公钥可以公开分发。
    Genkey,
    
    /// 从ID和主密钥提取用户私钥
    /// 
    /// 使用主密钥和用户ID提取对应的用户私钥。这个私钥允许用户解密
    /// 使用相应公钥和ID加密的消息。
    Extract {
        /// Sui网络上处理此密钥的KMS包的地址
        #[arg(long)]
        package_id: ObjectID,
        
        /// 应派生密钥的ID
        #[arg(long)]
        id: EncodedBytes,
        
        /// 主密钥。BLS12-381标量的Hex编码
        #[arg(long, value_parser = parse_serializable::<Scalar, DefaultEncoding>)]
        master_key: Scalar,
    },
    
    /// 验证用户私钥是否与公钥匹配
    /// 
    /// 检查给定的用户私钥是否对应于特定公钥和用户ID的有效私钥
    Verify {
        /// Sui网络上处理此密钥的KMS包的地址
        #[arg(long)]
        package_id: ObjectID,
        
        /// 应验证密钥的ID
        #[arg(long)]
        id: EncodedBytes,
        
        /// 用户私钥。压缩的BLS12-381 G1Element的Hex编码
        #[arg(long, value_parser = parse_serializable::<G1Element, DefaultEncoding>)]
        user_secret_key: G1Element,
        
        /// 公钥。压缩的BLS12-381 G2Element的Hex编码
        #[arg(long, value_parser = parse_serializable::<G2Element, DefaultEncoding>)]
        public_key: G2Element,
    },
    
    /// 使用Seal派生密钥（明文模式）
    /// 
    /// 使用基于身份的密钥封装机制(IBKEM)派生密钥，具体使用BLS12381上的Boneh-Franklin方案。
    /// 该命令输出可以公开共享的加密对象（以Hex编码的BCS序列化形式）和应私密保存的派生对称密钥。
    Plain {
        /// Sui网络上处理此密钥的KMS包的地址
        #[arg(long)]
        package_id: ObjectID,
        
        /// 应派生密钥的ID
        #[arg(long)]
        id: EncodedBytes,
        
        /// 解密所需的密钥服务器最小数量（阈值）
        #[arg(long)]
        threshold: u8,
        
        /// 密钥服务器的Hex编码公钥列表
        #[arg(value_parser = parse_serializable::<G2Element, DefaultEncoding>, num_args = 1..)]
        public_keys: Vec<G2Element>,
        
        /// 表示密钥服务器的Move对象地址列表
        #[arg(num_args = 1.., last = true)]
        object_ids: Vec<ObjectID>,
    },
    
    /// 使用Seal和AES-256-GCM加密消息
    /// 
    /// 使用基于身份的密钥封装机制(IBKEM)派生密钥，然后使用AES-256-GCM加密消息。
    /// 该命令输出可以公开共享的加密对象和应私密保存的派生对称密钥。
    EncryptAes {
        /// 要加密的消息（Hex编码字节）
        #[arg(long)]
        message: EncodedBytes,
        
        /// 可选的额外认证数据（Hex编码字节）
        #[arg(long)]
        aad: Option<EncodedBytes>,
        
        /// Sui网络上处理此加密的KMS包的地址
        #[arg(long)]
        package_id: ObjectID,
        
        /// 用于此加密的密钥ID
        #[arg(long)]
        id: EncodedBytes,
        
        /// 解密所需的密钥服务器最小数量（阈值）
        #[arg(long)]
        threshold: u8,
        
        /// 密钥服务器的Hex编码公钥列表
        #[arg(value_parser = parse_serializable::<G2Element, DefaultEncoding>, num_args = 1..)]
        public_keys: Vec<G2Element>,
        
        /// 表示密钥服务器的Move对象地址列表
        #[arg(num_args = 1.., last = true)]
        object_ids: Vec<ObjectID>,
    },
    
    /// 使用Seal和HMAC-256-CTR加密消息
    /// 
    /// 使用基于身份的密钥封装机制(IBKEM)派生密钥，然后使用计数器模式和hmac-sha3-256作为PRF加密消息。
    /// 该命令输出可以公开共享的加密对象和应私密保存的派生对称密钥。
    EncryptHmac {
        /// 要加密的消息（Hex编码字节）
        #[arg(long)]
        message: EncodedBytes,
        
        /// 可选的额外认证数据（Hex编码字节）
        #[arg(long)]
        aad: Option<EncodedBytes>,
        
        /// Sui网络上处理此加密的KMS包的地址
        #[arg(long)]
        package_id: ObjectID,
        
        /// 用于此加密的密钥ID
        #[arg(long)]
        id: EncodedBytes,
        
        /// 解密所需的密钥服务器最小数量（阈值）
        #[arg(long)]
        threshold: u8,
        
        /// 密钥服务器的Hex编码公钥列表
        #[arg(value_parser = parse_serializable::<G2Element, DefaultEncoding>, num_args = 1..)]
        public_keys: Vec<G2Element>,
        
        /// 表示密钥服务器的Move对象地址列表
        #[arg(num_args = 1.., last = true)]
        object_ids: Vec<ObjectID>,
    },
    
    /// 解密Seal加密对象
    /// 
    /// 使用提供的密钥服务器私钥解密加密对象。如果加密对象包含消息，则返回该消息。
    /// 如果使用了Plain模式，则返回派生的加密密钥。
    Decrypt {
        /// 加密对象（Hex编码字节）
        #[arg(value_parser = parse_serializable::<EncryptedObject, DefaultEncoding>)]
        encrypted_object: EncryptedObject,
        
        /// 密钥服务器的私钥列表。私钥顺序必须与object_ids字段中的密钥服务器顺序匹配
        #[arg(value_parser = parse_serializable::<G1Element, DefaultEncoding>, num_args = 1..)]
        secret_keys: Vec<G1Element>,
        
        /// 用于此解密的密钥服务器Move对象地址列表
        #[arg(num_args = 1.., last = true)]
        object_ids: Vec<ObjectID>,
    },
    
    /// 解析Seal加密对象
    /// 
    /// 解析并显示加密对象的各个组成部分，包括版本、包ID、加密份额等详细信息
    Parse {
        /// 加密对象（Hex编码字节）
        #[arg(value_parser = parse_serializable::<EncryptedObject, DefaultEncoding>)]
        encrypted_object: EncryptedObject,
    },
    
    /// 使用对称密钥直接解密加密对象
    /// 
    /// 当已知派生的对称密钥时，可以直接解密加密对象而无需使用私钥重建密钥
    SymmetricDecrypt {
        /// 加密对象（Hex编码字节）
        #[arg(value_parser = parse_serializable::<EncryptedObject, DefaultEncoding>)]
        encrypted_object: EncryptedObject,
        
        /// 加密时派生的对称密钥
        #[arg(long)]
        key: EncodedBytes,
    },

    /// 发布Move模块
    /// 
    /// 在Sui网络上发布指定路径的Move模块
    Publish {
        /// 要发布的模块路径
        #[arg(short = 'm')]
        module: String
    },
    
    /// 注册密钥服务器
    /// 
    /// 在Sui网络上注册一个密钥服务器，并返回注册后的服务器对象ID
    RegisterKeyServer {
        /// Seal包ID
        #[arg(long, short = 'p')]
        package_id: ObjectID,
        
        /// 服务器描述
        #[arg(long, short = 'd')]
        description: String,
        
        /// 服务器URL
        #[arg(long, short = 'u')]
        url: String,
        
        /// 服务器IBE公钥
        #[arg(long, short = 'k', value_parser = parse_serializable::<G2Element, DefaultEncoding>)]
        public_key: G2Element,
    },

    /// 解码为十六进制 (Decode from Base64)
    /// 
    /// 将数据解码为十六进制格式
    DeB64 {
        /// 十六进制字符串输入
        #[arg(long, short = 'x', group = "input")]
        hex: Option<String>,
        
        /// 字符串输入
        #[arg(long, short = 's', group = "input")]
        string: Option<String>,
    },

    /// 编码为Base64 (Encode to Base64)
    /// 
    /// 将数据编码为Base64格式
    EnB64 {
        /// 十六进制字符串输入
        #[arg(long, short = 'x', group = "input")]
        hex: Option<String>,
        
        /// 字符串输入
        #[arg(long, short = 's', group = "input")]
        string: Option<String>,
    },
}

/// 生成密钥命令的输出结构
struct GenkeyOutput((Scalar, G2Element));

/// 提取用户私钥命令的输出结构
struct ExtractOutput(G1Element);

/// 验证命令的输出结构
struct VerifyOutput(FastCryptoResult<()>);

/// 加密命令的输出结构
struct EncryptionOutput((EncryptedObject, [u8; KEY_LENGTH]));

/// 解密命令的输出结构
struct DecryptionOutput(Vec<u8>);

/// 解析命令的输出结构
struct ParseOutput(EncryptedObject);

/// 对称解密命令的输出结构
struct SymmetricDecryptOutput(Vec<u8>);

/// 用于CLI二进制输入的类型
/// 
/// 包装了一个字节向量，用于处理Hex编码的输入参数
#[derive(Debug, Clone)]
pub struct EncodedBytes(pub Vec<u8>);

impl FromStr for EncodedBytes {
    type Err = FastCryptoError;

    /// 从字符串解析EncodedBytes
    /// 
    /// 将Hex编码的字符串解码为字节向量
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        DefaultEncoding::decode(s).map(EncodedBytes)
    }
}

//
// 输出格式化
//

/**
 * 将可序列化对象转换为字符串
 * 
 * 使用BCS序列化对象，然后使用默认编码转换为字符串
 */
fn serializable_to_string<T: Serialize>(t: &T) -> String {
    DefaultEncoding::encode(bcs::to_bytes(t).expect("序列化失败"))
}

/**
 * 解析可序列化对象
 * 
 * 将编码的字符串解析为指定类型的对象
 */
pub fn parse_serializable<T: for<'a> Deserialize<'a>, E: Encoding>(s: &str) -> Result<T, String> {
    let bytes = E::decode(s).map_err(|e| format!("{}", e))?;
    bcs::from_bytes(&bytes).map_err(|e| format!("{}", e))
}

// 各命令输出的格式化实现

impl Display for GenkeyOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "主密钥: {}\n公钥: {}",
            serializable_to_string(&self.0 .0),
            serializable_to_string(&self.0 .1),
        )
    }
}

impl Display for ExtractOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "用户私钥: {}", serializable_to_string(&self.0))
    }
}

impl Display for VerifyOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            if self.0.is_ok() {
                "验证成功"
            } else {
                "验证失败"
            }
        )
    }
}

impl Display for EncryptionOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "加密对象 (BCS编码): {}\n对称密钥: {}",
            DefaultEncoding::encode(bcs::to_bytes(&self.0 .0).unwrap()),
            Hex::encode(self.0 .1)
        )
    }
}

impl Display for DecryptionOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "解密消息: {}", DefaultEncoding::encode(&self.0))
    }
}

impl Display for ParseOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "版本: {}", self.0.version)?;
        writeln!(f, "包ID: {}", self.0.package_id)?;
        writeln!(f, "ID: {}", DefaultEncoding::encode(&self.0.id))?;
        writeln!(f, "服务器列表及共享索引:")?;
        for (id, index) in &self.0.services {
            writeln!(f, "  {}: {}", id, index)?;
        }
        writeln!(f, "阈值: {}", self.0.threshold)?;
        writeln!(f, "密文:")?;
        match &self.0.ciphertext {
            Ciphertext::Aes256Gcm { blob, aad } => {
                writeln!(f, "  类型: AES-256-GCM")?;
                writeln!(f, "  数据: {}", DefaultEncoding::encode(blob))?;
                writeln!(
                    f,
                    "  额外认证数据: {}\n",
                    aad.as_ref()
                        .map_or("无".to_string(), DefaultEncoding::encode)
                )?;
            }
            Ciphertext::Hmac256Ctr { blob, aad, mac } => {
                writeln!(f, "  类型: HMAC-256-CTR")?;
                writeln!(f, "  数据: {}", DefaultEncoding::encode(blob))?;
                writeln!(
                    f,
                    "  额外认证数据: {}",
                    aad.as_ref()
                        .map_or("无".to_string(), DefaultEncoding::encode)
                )?;
                writeln!(f, "  MAC: {}", DefaultEncoding::encode(mac))?;
            }
            Ciphertext::Plain => {
                writeln!(f, "  类型: 明文")?;
            }
        }
        writeln!(f, "加密份额:")?;
        match &self.0.encrypted_shares {
            IBEEncryptions::BonehFranklinBLS12381 {
                encrypted_shares: shares,
                nonce: encapsulation,
                encrypted_randomness,
            } => {
                writeln!(f, "  类型: Boneh-Franklin BLS12-381")?;
                writeln!(f, "  份额列表:")?;
                for share in shares.iter() {
                    writeln!(f, "    {}", DefaultEncoding::encode(share))?;
                }
                writeln!(
                    f,
                    "  封装值: {}",
                    serializable_to_string(&encapsulation)
                )?;
                writeln!(
                    f,
                    "  加密随机性: {}",
                    DefaultEncoding::encode(encrypted_randomness)
                )?;
            }
        };
        Ok(())
    }
}

impl Display for SymmetricDecryptOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "解密消息: {}", DefaultEncoding::encode(&self.0))
    }
}

/// Base64编解码命令的输出结构
struct Base64Output {
    input_type: String,
    output_type: String,
    value: String,
}

impl Display for Base64Output {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}转换为{}: {}", self.input_type, self.output_type, self.value)
    }
}

/// 运行CLI命令
/// 
/// 处理来自主程序的CLI命令，执行相应的操作并返回结果
pub async fn run_cli_command(command: Command) -> anyhow::Result<()> {
    // 初始化环境变量
    dotenv().ok();
    // 根据命令执行相应的操作并格式化输出
    let output = match command {
        // 生成新的IBE密钥对
        Command::Genkey => GenkeyOutput(ibe::generate_key_pair(&mut thread_rng())).to_string(),
        
        // 从主密钥和ID提取用户私钥
        Command::Extract {
            package_id,
            id,
            master_key,
        } => ExtractOutput(ibe::extract(
            &master_key,
            &create_full_id(&package_id, &id.0),
        ))
        .to_string(),
        
        // 验证用户私钥是否与公钥匹配
        Command::Verify {
            package_id,
            id,
            user_secret_key,
            public_key,
        } => VerifyOutput(ibe::verify_user_secret_key(
            &user_secret_key,
            &create_full_id(&package_id, &id.0),
            &public_key,
        ))
        .to_string(),
        
        // 使用Seal派生密钥（明文模式）
        Command::Plain {
            package_id,
            id,
            threshold,
            public_keys,
            object_ids,
        } => EncryptionOutput(seal_encrypt(
            package_id,
            id.0,
            object_ids,
            &IBEPublicKeys::BonehFranklinBLS12381(public_keys),
            threshold,
            Plain,
        )?)
        .to_string(),
        
        // 使用Seal和AES-256-GCM加密消息
        Command::EncryptAes {
            message,
            aad,
            package_id,
            id,
            threshold,
            public_keys,
            object_ids,
        } => EncryptionOutput(seal_encrypt(
            package_id,
            id.0,
            object_ids,
            &IBEPublicKeys::BonehFranklinBLS12381(public_keys),
            threshold,
            EncryptionInput::Aes256Gcm {
                data: message.0,
                aad: aad.map(|a| a.0),
            },
        )?)
        .to_string(),
        
        // 使用Seal和HMAC-256-CTR加密消息
        Command::EncryptHmac {
            message,
            aad,
            package_id,
            id,
            threshold,
            public_keys,
            object_ids,
        } => EncryptionOutput(seal_encrypt(
            package_id,
            id.0,
            object_ids,
            &IBEPublicKeys::BonehFranklinBLS12381(public_keys),
            threshold,
            EncryptionInput::Hmac256Ctr {
                data: message.0,
                aad: aad.map(|a| a.0),
            },
        )?)
        .to_string(),
        
        // 解密Seal加密对象
        Command::Decrypt {
            encrypted_object,
            secret_keys,
            object_ids,
        } => DecryptionOutput(seal_decrypt(
            &encrypted_object,
            &IBEUserSecretKeys::BonehFranklinBLS12381(
                object_ids.into_iter().zip(secret_keys).collect(),
            ),
            None,
        )?)
        .to_string(),
        
        // 解析Seal加密对象
        Command::Parse { encrypted_object } => ParseOutput(encrypted_object).to_string(),
        
        // 使用对称密钥直接解密加密对象
        Command::SymmetricDecrypt {
            encrypted_object,
            key,
        } => {
            // 转换输入密钥为正确的格式
            let dem_key = key
                .0
                .try_into()
                .map_err(|_| FastCryptoError::InvalidInput)?;
            let EncryptedObject { ciphertext, .. } = encrypted_object;

            // 根据加密模式选择相应的解密方法
            match ciphertext {
                Ciphertext::Aes256Gcm { blob, aad } => {
                    Aes256Gcm::decrypt(&blob, &aad.unwrap_or(vec![]), &dem_key)
                }
                Ciphertext::Hmac256Ctr { blob, aad, mac } => {
                    Hmac256Ctr::decrypt(&blob, &mac, &aad.unwrap_or(vec![]), &dem_key)
                }
                _ => Err(FastCryptoError::InvalidInput),
            }
            .map(SymmetricDecryptOutput)?
            .to_string()
        },
        
        // 发布Move模块
        Command::Publish {
            module,
        } => {
            // 读取发布的环境
            let network = AppState::init_network();
            // 构建模块路径
            let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            path.extend(["move", &module]);
            info!("Model Path: {:?}",path);
            // 编译Move包
            let compiled_package = BuildConfig::new_for_testing()
                .build(&path)
                .context("Compile Move package failed")?;
            
            // 初始化SUI客户端
            let sui_client = SuiClientBuilder::default()
                .build(network.node_url())
                .await
                .expect("Sui client build failed");
            
            // 创建事务构建器
            let tx_builder = sui_client.transaction_builder();
            
            // 从环境变量获取密钥对并创建密钥库
            let sk = env::var("WALLET_SK").context("未设置WALLET_SK环境变量")?;
            let (keystore, _, sender) = txb::create_keystore_from_sk(&sk, Some("EnvKeyPair".to_string()))?;
            
            // 创建发布事务
            let tx_data = tx_builder
                .publish(
                    sender,
                    compiled_package.get_package_bytes(true),
                    compiled_package.get_dependency_storage_package_ids(),
                    None,
                    crate::types::GAS_BUDGET,
                )
                .await
                .context("创建发布事务失败")?;
            
            // 使用txb模块执行事务
            let response = txb::execute_transaction(
                &sui_client,
                tx_data,
                &keystore,
                &sender
            )
            .await
            .context("执行事务失败")?;
            
            if !response.status_ok().unwrap() {
                anyhow::bail!("Transaction failed: {:?}", response.effects.as_ref().unwrap());
            }
            
            let changes = response.object_changes.unwrap();

            // 返回第一个（也是唯一一个）发布的包的ID
            let package_id = changes
                .iter()
                .find_map(|d| match d {
                    ObjectChange::Published { package_id, .. } => Some(*package_id),
                    _ => None,
                })
                .unwrap();

            // 找到升级能力ID
            let upgrade_cap = changes
                .iter()
                .find_map(|d| match d {
                    ObjectChange::Created { object_id, .. } => Some(*object_id),
                    _ => None,
                })
                .unwrap();

            // 找到并汇总创建的所有对象
            let mut created_objects = Vec::new();
            let mut admin_cap_objects = Vec::new();
            
            for change in changes.iter() {
                match change {
                    ObjectChange::Created { object_id, object_type, owner, .. } => {
                        // 检查是否是共享对象
                        let is_shared = match owner {
                            Owner::Shared { .. } => true,
                            _ => false,
                        };
                        
                        if is_shared {
                            created_objects.push((object_id, object_type));
                        }
                        
                        // 检查是否是AdminCap对象（直接转移给发送者的）
                        if object_type.to_string().ends_with("::citadel::AdminCap") {
                            match owner {
                                Owner::AddressOwner(addr) if *addr == sender => {
                                    admin_cap_objects.push((object_id, object_type));
                                },
                                _ => {}
                            }
                        }
                    },
                    _ => {}
                }
            }

            let mut result = format!("发布成功！\n包ID: {}\n升级能力ID: {}", package_id, upgrade_cap);
            
            // 如果有AdminCap对象，打印它们
            if !admin_cap_objects.is_empty() {
                result.push_str("\n\nAdminCap对象:");
                for (id, type_info) in admin_cap_objects {
                    result.push_str(&format!("\n对象ID: {}\n类型: {}", id, type_info));
                }
            }
            
            // 如果有共享对象，打印它们
            if !created_objects.is_empty() {
                result.push_str("\n\n创建的共享对象:");
                for (id, type_info) in created_objects {
                    result.push_str(&format!("\n对象ID: {}\n类型: {}", id, type_info));
                }
            }

            result
        },
        
        // 注册密钥服务器
        Command::RegisterKeyServer {
            package_id,
            description,
            url,
            public_key,
        } => {
            // 初始化环境变量
            dotenv().ok();
            // 读取网络配置
            let network = AppState::init_network();
            
            // 初始化SUI客户端
            let sui_client = SuiClientBuilder::default()
                .build(network.node_url())
                .await
                .expect("Sui client build failed");
            
            // 从环境变量获取密钥对并创建密钥库
            let sk = env::var("WALLET_SK").context("未设置WALLET_SK环境变量")?;
            let (keystore, _, sender) = txb::create_keystore_from_sk(&sk, Some("EnvKeyPair".to_string()))?;
            
            // 构建注册事务
            let tx_builder = sui_client.transaction_builder();
            let tx_data = tx_builder
                .move_call(
                    sender,
                    package_id,
                    "key_server",
                    "register_and_transfer",
                    vec![],
                    vec![
                        SuiJsonValue::from_str(&description).context("序列化描述失败")?,
                        SuiJsonValue::from_str(&url).context("序列化URL失败")?,
                        SuiJsonValue::from_str(&0u8.to_string()).context("序列化算法类型失败")?,
                        SuiJsonValue::new(json!(public_key.to_byte_array().to_vec())).context("序列化公钥失败")?,
                    ],
                    None,
                    crate::types::GAS_BUDGET,
                    None,
                )
                .await
                .context("创建注册事务失败")?;
            
            // 使用txb模块执行事务
            let response = txb::execute_transaction(
                &sui_client,
                tx_data,
                &keystore,
                &sender
            )
            .await
            .context("执行交易失败")?;
            
            // 检查交易是否成功
            if !response.status_ok().unwrap_or(false) {
                anyhow::bail!("交易执行失败: {:?}", response.effects.as_ref().unwrap());
            }
            
            // 从响应中查找创建的KeyServer对象
            let changes = response.object_changes.unwrap();
            let service_objects = changes
                .iter()
                .filter_map(|change| match change {
                    ObjectChange::Created { object_type, object_id, .. } if object_type.to_string().ends_with("::key_server::KeyServer") => {
                        Some(*object_id)
                    },
                    _ => None,
                })
                .collect::<Vec<_>>();
            
            if service_objects.is_empty() {
                anyhow::bail!("未找到创建的KeyServer对象");
            }
            
            format!("密钥服务器注册成功！\n服务器对象ID: {}", service_objects[0])
        },

        // 解码为十六进制
        Command::DeB64 { hex, string } => {
            if let Some(base64_str) = hex {
                // 解码Base64字符串到十六进制
                let bytes = Base64::decode(&base64_str)
                    .map_err(|e| anyhow::anyhow!("无效的Base64字符串: {}", e))?;
                let hex_str = Hex::encode(&bytes);
                
                Base64Output {
                    input_type: "Base64".to_string(),
                    output_type: "Hex".to_string(),
                    value: hex_str,
                }.to_string()
            } else if let Some(str_input) = string {
                let bytes = Base64::decode(&str_input)
                    .map_err(|e| anyhow::anyhow!("无效的Base64字符串: {}", e))?;
                // 将字符串解码为十六进制
                let decode_str= String::from_utf8(bytes).unwrap();
                
                Base64Output {
                    input_type: "字符串".to_string(),
                    output_type: "字符串".to_string(),
                    value: decode_str,
                }.to_string()
            } else {
                anyhow::bail!("必须提供-x或-s参数");
            }
        },
        
        // 编码为Base64
        Command::EnB64 { hex, string } => {
            if let Some(hex_str) = hex {
                // 解码Hex字符串到Base64
                let bytes = Hex::decode(&hex_str)
                    .map_err(|e| anyhow::anyhow!("无效的16进制字符串: {}", e))?;
                let base64_str = Base64::encode(&bytes);
                
                Base64Output {
                    input_type: "Hex".to_string(),
                    output_type: "Base64".to_string(),
                    value: base64_str,
                }.to_string()
            } else if let Some(str_input) = string {
                // 将字符串编码为Base64
                let base64_str = Base64::encode(str_input.as_bytes());
                
                Base64Output {
                    input_type: "字符串".to_string(),
                    output_type: "Base64".to_string(),
                    value: base64_str,
                }.to_string()
            } else {
                anyhow::bail!("必须提供-x或-s参数");
            }
        },
    };
    
    // 输出结果
    println!("{}", output);
    Ok(())
}