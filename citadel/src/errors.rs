// Copyright (c), Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/**
 * 错误处理模块
 *
 * 本模块定义了密钥服务器可能返回的各种错误类型，包括：
 * 1. 请求验证错误 - 如无效的PTB、签名或包ID
 * 2. 访问控制错误 - 如无权访问特定密钥
 * 3. 服务器内部错误 - 如系统故障
 * 4. JWT令牌验证错误 - 如无效或过期的令牌
 *
 * 每种错误类型都映射到特定的HTTP状态码和错误消息，以提供清晰的客户端反馈。
 */
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

/**
 * 内部错误枚举
 * 定义了密钥服务器可能遇到的各种错误情况
 */
#[derive(Debug, Serialize, PartialEq)]
pub enum InternalError {
    /// 无效的可编程交易块(PTB)格式
    InvalidPTB,
    /// 无效的包ID，请求的包ID不被服务器识别
    InvalidPackage,
    /// 访问被拒绝，用户没有请求密钥的权限
    NoAccess,
    /// 使用了旧版本的包，需要升级
    OldPackageVersion,
    /// 无效的用户签名，用户身份验证失败
    InvalidSignature,
    /// 无效的会话密钥签名，会话验证失败
    InvalidSessionSignature,
    /// 无效的证书时间或TTL(生存时间)
    InvalidCertificate,
    /// 服务器内部错误，稍后重试
    Failure,
    /// 客户端不是最新版本
    SuiClientNotFresh,
    /// 无效的输入
    InvalidInput,
    /// 解密错误
    DecryptionError,
    /// 序列化错误
    SerializationError,
    // ===== JWT令牌验证错误 =====
    /// JWT令牌无效（签名验证失败、格式错误等）
    InvalidToken,
    /// JWT令牌已过期
    ExpiredToken,
    /// 请求头中缺少Authorization令牌
    MissingAuthToken,
    /// Authorization请求头格式无效
    InvalidAuthHeader,
    /// 用户无权访问请求的资源
    Unauthorized,
}

/**
 * 错误响应结构
 * 包含错误类型和详细错误消息，用于HTTP响应
 */
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    error: InternalError,
    message: String,
}

/**
 * 实现IntoResponse特性
 * 将内部错误转换为HTTP响应
 */
impl IntoResponse for InternalError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            InternalError::InvalidPTB => (StatusCode::FORBIDDEN, "Invalid PTB"),
            InternalError::InvalidPackage => (StatusCode::FORBIDDEN, "Invalid package ID"),
            InternalError::NoAccess => (StatusCode::FORBIDDEN, "Access denied"),
            InternalError::InvalidCertificate => {
                (StatusCode::FORBIDDEN, "Invalid certificate time or ttl")
            }
            InternalError::OldPackageVersion => (
                StatusCode::FORBIDDEN,
                "Package has been upgraded, please use the latest version",
            ),
            InternalError::InvalidSignature => (StatusCode::FORBIDDEN, "Invalid user signature"),
            InternalError::InvalidSessionSignature => {
                (StatusCode::FORBIDDEN, "Invalid session key signature")
            }
            InternalError::Failure => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Internal server error, please try again later",
            ),
            InternalError::SuiClientNotFresh => (
                StatusCode::FORBIDDEN,
                "Client is not up-to-date, please update to the latest version",
            ),
            InternalError::InvalidInput => (StatusCode::FORBIDDEN, "Invalid input"),
            InternalError::DecryptionError => (StatusCode::FORBIDDEN, "Decryption error"),
            InternalError::InvalidToken => {
                (StatusCode::UNAUTHORIZED, "Invalid authentication token")
            }
            InternalError::ExpiredToken => {
                (StatusCode::UNAUTHORIZED, "Authentication token has expired")
            }
            InternalError::MissingAuthToken => {
                (StatusCode::UNAUTHORIZED, "Authentication token is missing")
            }
            InternalError::InvalidAuthHeader => (
                StatusCode::UNAUTHORIZED,
                "Invalid Authorization header format",
            ),
            InternalError::Unauthorized => (
                StatusCode::FORBIDDEN,
                "User is not authorized to access this resource",
            ),
            InternalError::SerializationError => (StatusCode::FORBIDDEN, "Serialization error"),
        };

        let error_response = ErrorResponse {
            error: self,
            message: message.to_string(),
        };

        (status, Json(error_response)).into_response()
    }
}

/**
 * 错误类型字符串表示
 * 提供用于日志和指标的错误标识符
 */
impl InternalError {
    pub fn as_str(&self) -> &'static str {
        match self {
            InternalError::InvalidPTB => "InvalidPTB",
            InternalError::InvalidPackage => "InvalidPackage",
            InternalError::NoAccess => "NoAccess",
            InternalError::InvalidCertificate => "InvalidCertificate",
            InternalError::OldPackageVersion => "OldPackageVersion",
            InternalError::InvalidSignature => "InvalidSignature",
            InternalError::InvalidSessionSignature => "InvalidSessionSignature",
            InternalError::Failure => "Failure",
            InternalError::SuiClientNotFresh => "SuiClientNotFresh",
            InternalError::InvalidInput => "InvalidInput",
            InternalError::DecryptionError => "DecryptionError",
            InternalError::InvalidToken => "InvalidToken",
            InternalError::ExpiredToken => "ExpiredToken",
            InternalError::MissingAuthToken => "MissingAuthToken",
            InternalError::InvalidAuthHeader => "InvalidAuthHeader",
            InternalError::Unauthorized => "Unauthorized",
            InternalError::SerializationError => "SerializationError",
        }
    }
}

impl From<serde_json::Error> for InternalError {
    fn from(_: serde_json::Error) -> Self {
        InternalError::SerializationError
    }
}

impl From<tower_sessions::session::Error> for InternalError {
    fn from(_: tower_sessions::session::Error) -> Self {
        InternalError::Failure
    }
}