// Copyright (c), Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/**
 * 类型定义模块
 *
 * 本模块定义了密钥服务器中使用的核心类型，包括：
 * 1. 基于身份的加密(IBE)类型 - 用于密钥加密和分发
 * 2. ElGamal加密类型 - 用于安全通信
 * 3. 网络配置类型 - 支持不同的部署环境
 */
use crypto::elgamal;
use crypto::ibe;

/// 基于身份的加密相关类型
/// IBE主密钥，用于生成用户私钥，应安全存储
pub type IbeMasterKey = ibe::MasterKey;
/// IBE派生密钥，为特定用户生成的私钥
type IbeDerivedKey = ibe::UserSecretKey;
/// IBE公钥，公开发布
type IbePublicKey = ibe::PublicKey;

/// ElGamal加密相关类型
/// ElGamal公钥，用于加密IBE派生密钥
pub type ElGamalPublicKey = elgamal::PublicKey<IbeDerivedKey>;
/// ElGamal加密结果，包含加密后的IBE派生密钥
pub type ElgamalEncryption = elgamal::Encryption<IbeDerivedKey>;
/// ElGamal验证密钥，用于验证加密通信
pub type ElgamalVerificationKey = elgamal::VerificationKey<IbePublicKey>;

/// 主密钥持有证明，证明服务器确实拥有声称的主密钥
pub type MasterKeyPOP = ibe::ProofOfPossession;

/// 最大预算的1%
pub const GAS_BUDGET: u64 = 500_000_000;

/**
 * 网络环境枚举
 * 定义了密钥服务器可以部署和连接的不同网络环境
 */
#[derive(Clone, Debug, PartialEq)]
pub enum Network {
    /// 开发网络，用于开发和初步测试
    Devnet,
    /// 测试网络，用于更广泛的测试和集成
    Testnet,
    /// 主网，生产环境
    Mainnet,
    /// 自定义网络，允许指定自定义节点和GraphQL URL
    Custom {
        node_url: String,
        graphql_url: String,
        explorer_url: Option<String>, // 添加自定义浏览器URL
    },
    /// 测试集群，仅用于单元测试
    #[cfg(test)]
    TestCluster,
}

impl Network {
    /**
     * 获取当前网络的节点URL
     *
     * 返回:
     * 对应网络环境的全节点URL
     */
    pub fn node_url(&self) -> String {
        // 优先使用环境变量中的配置
        if let Ok(url) = std::env::var("NODE_URL") {
            return url;
        }
        match self {
            Network::Devnet => "https://fullnode.devnet.sui.io:443".into(),
            Network::Testnet => "https://fullnode.testnet.sui.io:443".into(),
            Network::Mainnet => "https://fullnode.mainnet.sui.io:443".into(),
            Network::Custom { node_url, .. } => node_url.clone(),
            #[cfg(test)]
            Network::TestCluster => {
                panic!("GraphQL and Explorer services are not available in test cluster")
            }
        }
    }

    /**
     * 获取当前网络的GraphQL URL
     *
     * 返回:
     * 对应网络环境的GraphQL端点URL
     */
    pub fn graphql_url(&self) -> String {
        // 优先使用环境变量中的配置
        if let Ok(url) = std::env::var("GRAPHQL_URL") {
            return url;
        }

        match self {
            Network::Devnet => "https://sui-devnet.mystenlabs.com/graphql".into(),
            Network::Testnet => "https://sui-testnet.mystenlabs.com/graphql".into(),
            Network::Mainnet => "https://sui-mainnet.mystenlabs.com/graphql".into(),
            Network::Custom { graphql_url, .. } => graphql_url.clone(),
            #[cfg(test)]
            Network::TestCluster => {
                panic!("GraphQL and Explorer services are not available in test cluster")
            }
        }
    }

    /**
     * 获取浏览器的基本URL，包含网络名称
     *
     * 返回:
     * 包含网络名称的浏览器基本URL
     */
    pub fn explorer_base_url(&self) -> String {
        // 优先使用环境变量中的配置
        if let Ok(url) = std::env::var("EXPLORER_URL") {
            return url;
        }

        match self {
            Network::Devnet => "https://suiscan.xyz/devnet".into(),
            Network::Testnet => "https://suiscan.xyz/testnet".into(),
            Network::Mainnet => "https://suiscan.xyz/mainnet".into(),
            Network::Custom { explorer_url, .. } => explorer_url
                .clone()
                .unwrap_or_else(|| "https://suiscan.xyz/testnet".into()),
            #[cfg(test)]
            Network::TestCluster => panic!("Explorer URL is not available in test cluster"),
        }
    }

    /**
     * 获取交易的浏览器URL
     *
     * 参数:
     * @param digest - 交易摘要
     *
     * 返回:
     * 交易在浏览器中的URL
     */
    pub fn explorer_tx_url(&self, digest: &str) -> String {
        format!("{}/tx/{}", self.explorer_base_url(), digest)
    }

    /**
     * 获取对象的浏览器URL
     *
     * 参数:
     * @param object_id - 对象ID
     *
     * 返回:
     * 对象在浏览器中的URL
     */
    pub fn explorer_object_url(&self, object_id: &str) -> String {
        format!("{}/object/{}", self.explorer_base_url(), object_id)
    }

    /**
     * 获取用户地址的浏览器URL
     *
     * 参数:
     * @param address - 用户地址
     *
     * 返回:
     * 用户地址在浏览器中的URL
     */
    pub fn explorer_account_url(&self, address: &str) -> String {
        format!("{}/account/{}", self.explorer_base_url(), address)
    }

    /**
     * 从字符串创建网络枚举
     *
     * 参数:
     * @param str - 网络名称字符串
     *
     * 返回:
     * 对应的Network枚举值
     *
     * 对于自定义网络，需要设置NODE_URL和GRAPHQL_URL环境变量
     */
    pub fn from_str(str: &str) -> Self {
        match str.to_ascii_lowercase().as_str() {
            "devnet" => Network::Devnet,
            "testnet" => Network::Testnet,
            "mainnet" => Network::Mainnet,
            "custom" => Network::Custom {
                node_url: std::env::var("NODE_URL").expect("NODE_URL must be set"),
                graphql_url: std::env::var("GRAPHQL_URL").expect("GRAPHQL_URL must be set"),
                explorer_url: std::env::var("EXPLORER_URL").ok(),
            },
            _ => panic!("Unknown network: {}", str),
        }
    }
}
