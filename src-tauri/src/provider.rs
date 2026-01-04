//! Droid Provider 核心实现
//!
//! 实现凭证管理、模型支持检查等核心功能。

use crate::auth::encryption::{decrypt_sensitive_data, encrypt_sensitive_data, hash_api_key};
use crate::credentials::{
    AcquiredCredential, ApiKeyEntry, AuthType, DroidCredentials, EndpointType, ValidationResult,
};
use crate::token_refresh::TokenRefreshResult;
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Factory.ai API 基础 URL
pub const FACTORY_API_BASE_URL: &str = "https://api.factory.ai/api/llm";

/// 端点路径
pub const ENDPOINT_ANTHROPIC: &str = "/a/v1/messages";
pub const ENDPOINT_OPENAI: &str = "/o/v1/responses";
pub const ENDPOINT_COMM: &str = "/o/v1/chat/completions";

/// 模型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
    pub family: Option<String>,
    pub context_length: Option<u32>,
    pub supports_vision: bool,
    pub supports_tools: bool,
}

/// Provider 错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderError {
    pub error_type: String,
    pub message: String,
    pub status_code: Option<u16>,
    pub retryable: bool,
    pub cooldown_seconds: Option<u64>,
}

lazy_static::lazy_static! {
    static ref CREDENTIALS: Arc<RwLock<HashMap<String, DroidCredentials>>> =
        Arc::new(RwLock::new(HashMap::new()));
    static ref ENCRYPTION_KEY: String = std::env::var("DROID_ENCRYPTION_KEY")
        .unwrap_or_else(|_| "default-droid-encryption-key".to_string());
}

/// 列出支持的模型
pub fn list_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: "claude-opus-4-1-20250805".to_string(),
            display_name: "Claude Opus 4.1".to_string(),
            family: Some("opus".to_string()),
            context_length: Some(200000),
            supports_vision: true,
            supports_tools: true,
        },
        ModelInfo {
            id: "claude-sonnet-4-5-20250929".to_string(),
            display_name: "Claude Sonnet 4.5".to_string(),
            family: Some("sonnet".to_string()),
            context_length: Some(200000),
            supports_vision: true,
            supports_tools: true,
        },
        ModelInfo {
            id: "claude-sonnet-4-20250514".to_string(),
            display_name: "Claude Sonnet 4".to_string(),
            family: Some("sonnet".to_string()),
            context_length: Some(200000),
            supports_vision: true,
            supports_tools: true,
        },
        ModelInfo {
            id: "gpt-5-2025-08-07".to_string(),
            display_name: "GPT-5".to_string(),
            family: Some("gpt".to_string()),
            context_length: Some(128000),
            supports_vision: true,
            supports_tools: true,
        },
    ]
}

/// 检查是否支持某个模型
pub fn supports_model(model: &str) -> bool {
    model.starts_with("claude-") || model.starts_with("gpt-")
}

/// 获取端点路径
fn get_endpoint_path(endpoint_type: EndpointType) -> &'static str {
    match endpoint_type {
        EndpointType::Anthropic => ENDPOINT_ANTHROPIC,
        EndpointType::OpenAI => ENDPOINT_OPENAI,
        EndpointType::Comm => ENDPOINT_COMM,
    }
}

/// 获取凭证
pub async fn acquire_credential(model: &str) -> Result<AcquiredCredential> {
    if !supports_model(model) {
        anyhow::bail!("不支持的模型: {}", model);
    }

    let creds = CREDENTIALS.read().await;

    // 查找健康的凭证
    let healthy_creds: Vec<_> = creds.iter().filter(|(_, c)| c.is_healthy).collect();

    if healthy_creds.is_empty() {
        anyhow::bail!("没有可用的健康凭证");
    }

    // 选择第一个健康凭证
    let (id, credential) = healthy_creds.first().unwrap();

    let endpoint_path = get_endpoint_path(credential.endpoint_type);
    let base_url = format!("{}{}", FACTORY_API_BASE_URL, endpoint_path);

    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert(
        "User-Agent".to_string(),
        "factory-cli/0.32.1".to_string(),
    );
    headers.insert("x-factory-client".to_string(), "cli".to_string());

    match credential.auth_type {
        AuthType::OAuth => {
            let token = credential
                .access_token
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("凭证没有有效的 access_token"))?;

            headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        }
        AuthType::ApiKey => {
            // 选择一个可用的 API Key
            let active_keys: Vec<_> = credential
                .api_keys
                .iter()
                .filter(|k| k.status == "active")
                .collect();

            if active_keys.is_empty() {
                anyhow::bail!("没有可用的 API Key");
            }

            // 随机选择一个
            let selected = &active_keys[rand::random::<usize>() % active_keys.len()];
            let api_key = decrypt_sensitive_data(&selected.encrypted_key, &ENCRYPTION_KEY)?;

            headers.insert("Authorization".to_string(), format!("Bearer {}", api_key));
        }
    }

    Ok(AcquiredCredential {
        id: (*id).clone(),
        name: credential.name.clone(),
        auth_type: credential.auth_type.to_string(),
        base_url: Some(base_url),
        headers,
        metadata: HashMap::new(),
    })
}

/// 释放凭证
pub async fn release_credential(credential_id: &str, result: serde_json::Value) -> Result<()> {
    let mut creds = CREDENTIALS.write().await;

    if let Some(credential) = creds.get_mut(credential_id) {
        credential.usage_count += 1;

        if let Some(error) = result.get("error") {
            credential.error_count += 1;
            credential.last_error = error
                .get("message")
                .and_then(|m| m.as_str())
                .map(String::from);

            if error
                .get("mark_unhealthy")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                credential.is_healthy = false;
                warn!("凭证标记为不健康: {}", credential_id);
            }
        } else {
            credential.is_healthy = true;
            credential.last_error = None;
            debug!("凭证使用成功: {}", credential_id);
        }
    }

    Ok(())
}

/// 验证凭证
pub async fn validate_credential(credential_id: &str) -> Result<ValidationResult> {
    let creds = CREDENTIALS.read().await;

    if let Some(credential) = creds.get(credential_id) {
        let is_valid = match credential.auth_type {
            AuthType::OAuth => credential.access_token.is_some() || credential.refresh_token.is_some(),
            AuthType::ApiKey => {
                credential
                    .api_keys
                    .iter()
                    .any(|k| k.status == "active")
            }
        };

        Ok(ValidationResult {
            valid: is_valid && credential.is_healthy,
            message: if is_valid {
                Some("凭证有效".to_string())
            } else {
                Some("凭证配置不完整".to_string())
            },
            details: HashMap::new(),
        })
    } else {
        Ok(ValidationResult {
            valid: false,
            message: Some("凭证不存在".to_string()),
            details: HashMap::new(),
        })
    }
}

/// 刷新 Token
pub async fn refresh_token(credential_id: &str) -> Result<TokenRefreshResult> {
    let mut creds = CREDENTIALS.write().await;

    if let Some(credential) = creds.get_mut(credential_id) {
        let result = crate::token_refresh::refresh_token(credential).await?;
        info!("Token 刷新成功: {}", credential_id);
        Ok(result)
    } else {
        anyhow::bail!("凭证不存在: {}", credential_id)
    }
}

/// 创建凭证
pub async fn create_credential(auth_type: &str, config: serde_json::Value) -> Result<String> {
    let auth_type_enum = match auth_type {
        "oauth" => AuthType::OAuth,
        "api_key" => AuthType::ApiKey,
        _ => anyhow::bail!("不支持的认证类型: {}", auth_type),
    };

    let mut droid_config: DroidCredentials = serde_json::from_value(config.clone())?;
    droid_config.auth_type = auth_type_enum;

    // 处理 API Key 加密
    if auth_type_enum == AuthType::ApiKey {
        if let Some(api_keys) = config.get("api_keys").and_then(|v| v.as_array()) {
            let mut entries = Vec::new();
            for key in api_keys {
                if let Some(key_str) = key.as_str() {
                    let hash = hash_api_key(key_str);
                    let encrypted = encrypt_sensitive_data(key_str, &ENCRYPTION_KEY)?;
                    entries.push(ApiKeyEntry {
                        id: uuid::Uuid::new_v4().to_string(),
                        hash,
                        encrypted_key: encrypted,
                        created_at: Utc::now().to_rfc3339(),
                        last_used_at: None,
                        usage_count: 0,
                        status: "active".to_string(),
                        error_message: None,
                    });
                }
            }
            droid_config.api_keys = entries;
        }
    }

    // 验证必要字段
    match auth_type_enum {
        AuthType::OAuth => {
            if droid_config.refresh_token.is_none() && droid_config.access_token.is_none() {
                anyhow::bail!("OAuth 认证需要 access_token 或 refresh_token");
            }
        }
        AuthType::ApiKey => {
            if droid_config.api_keys.is_empty() {
                anyhow::bail!("API Key 认证需要至少一个 API Key");
            }
        }
    }

    // 生成凭证 ID
    let credential_id = uuid::Uuid::new_v4().to_string();

    // 存储凭证
    let mut creds = CREDENTIALS.write().await;
    creds.insert(credential_id.clone(), droid_config);

    info!("创建凭证成功: {} (类型: {})", credential_id, auth_type);
    Ok(credential_id)
}

/// 转换请求
pub async fn transform_request(request: serde_json::Value) -> Result<serde_json::Value> {
    // Droid 直接转发，无需转换
    Ok(request)
}

/// 转换响应
pub async fn transform_response(response: serde_json::Value) -> Result<serde_json::Value> {
    Ok(response)
}

/// 应用风控
pub async fn apply_risk_control(
    _request: &mut serde_json::Value,
    _credential_id: &str,
) -> Result<()> {
    // Droid 暂不需要特殊风控
    Ok(())
}

/// 解析错误
pub fn parse_error(status: u16, body: &str) -> Option<ProviderError> {
    match status {
        401 => Some(ProviderError {
            error_type: "authentication".to_string(),
            message: "Token 已过期或无效".to_string(),
            status_code: Some(status),
            retryable: true,
            cooldown_seconds: Some(0),
        }),
        403 => Some(ProviderError {
            error_type: "authorization".to_string(),
            message: "权限不足".to_string(),
            status_code: Some(status),
            retryable: false,
            cooldown_seconds: None,
        }),
        429 => Some(ProviderError {
            error_type: "rate_limit".to_string(),
            message: "请求过于频繁".to_string(),
            status_code: Some(status),
            retryable: true,
            cooldown_seconds: Some(60),
        }),
        500..=599 => Some(ProviderError {
            error_type: "server_error".to_string(),
            message: format!("服务器错误: {}", body),
            status_code: Some(status),
            retryable: true,
            cooldown_seconds: Some(10),
        }),
        _ => None,
    }
}
