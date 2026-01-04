//! 凭证数据结构

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 认证类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    /// WorkOS OAuth 认证
    OAuth,
    /// API Key 认证
    ApiKey,
}

impl Default for AuthType {
    fn default() -> Self {
        Self::OAuth
    }
}

impl std::fmt::Display for AuthType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthType::OAuth => write!(f, "oauth"),
            AuthType::ApiKey => write!(f, "api_key"),
        }
    }
}

/// 端点类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndpointType {
    /// Anthropic Messages API
    Anthropic,
    /// OpenAI Responses API
    OpenAI,
    /// OpenAI Chat Completions API
    Comm,
}

impl Default for EndpointType {
    fn default() -> Self {
        Self::Anthropic
    }
}

impl std::fmt::Display for EndpointType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EndpointType::Anthropic => write!(f, "anthropic"),
            EndpointType::OpenAI => write!(f, "openai"),
            EndpointType::Comm => write!(f, "comm"),
        }
    }
}

/// API Key 条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyEntry {
    /// 条目 ID
    pub id: String,
    /// API Key 哈希（用于去重）
    pub hash: String,
    /// 加密的 API Key
    pub encrypted_key: String,
    /// 创建时间
    pub created_at: String,
    /// 最后使用时间
    #[serde(default)]
    pub last_used_at: Option<String>,
    /// 使用次数
    #[serde(default)]
    pub usage_count: u64,
    /// 状态 (active/error)
    #[serde(default = "default_status")]
    pub status: String,
    /// 错误信息
    #[serde(default)]
    pub error_message: Option<String>,
}

fn default_status() -> String {
    "active".to_string()
}

/// Droid 凭证
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DroidCredentials {
    /// 凭证名称
    #[serde(default)]
    pub name: Option<String>,
    /// 认证类型
    #[serde(default)]
    pub auth_type: AuthType,
    /// 端点类型
    #[serde(default)]
    pub endpoint_type: EndpointType,

    // OAuth 字段
    /// Access Token
    pub access_token: Option<String>,
    /// Refresh Token
    pub refresh_token: Option<String>,
    /// 过期时间 (RFC3339 格式)
    pub expires_at: Option<String>,
    /// 组织 ID
    pub organization_id: Option<String>,
    /// 用户 ID
    pub user_id: Option<String>,
    /// 所有者邮箱
    pub owner_email: Option<String>,
    /// 所有者名称
    pub owner_name: Option<String>,
    /// Token 类型
    #[serde(default = "default_token_type")]
    pub token_type: String,

    // API Key 字段
    /// API Key 条目列表
    #[serde(default)]
    pub api_keys: Vec<ApiKeyEntry>,

    // 通用字段
    /// 最后刷新时间
    pub last_refresh: Option<String>,
    /// 是否健康
    #[serde(default = "default_true")]
    pub is_healthy: bool,
    /// 使用次数
    #[serde(default)]
    pub usage_count: u64,
    /// 错误次数
    #[serde(default)]
    pub error_count: u64,
    /// 最后错误信息
    #[serde(default)]
    pub last_error: Option<String>,
}

fn default_token_type() -> String {
    "Bearer".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for DroidCredentials {
    fn default() -> Self {
        Self {
            name: None,
            auth_type: AuthType::OAuth,
            endpoint_type: EndpointType::Anthropic,
            access_token: None,
            refresh_token: None,
            expires_at: None,
            organization_id: None,
            user_id: None,
            owner_email: None,
            owner_name: None,
            token_type: default_token_type(),
            api_keys: Vec::new(),
            last_refresh: None,
            is_healthy: true,
            usage_count: 0,
            error_count: 0,
            last_error: None,
        }
    }
}

/// 获取的凭证
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcquiredCredential {
    /// 凭证 ID
    pub id: String,
    /// 凭证名称
    #[serde(default)]
    pub name: Option<String>,
    /// 认证方式
    pub auth_type: String,
    /// Base URL
    #[serde(default)]
    pub base_url: Option<String>,
    /// 请求头
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// 额外元数据
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 凭证验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// 是否有效
    pub valid: bool,
    /// 消息
    #[serde(default)]
    pub message: Option<String>,
    /// 额外信息
    #[serde(default)]
    pub details: HashMap<String, serde_json::Value>,
}

/// WorkOS Token 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkOSTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<String>,
    pub expires_in: Option<i64>,
    pub token_type: Option<String>,
    pub organization_id: Option<String>,
    pub user: Option<WorkOSUser>,
    pub authentication_method: Option<String>,
}

/// WorkOS 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkOSUser {
    pub id: Option<String>,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}
