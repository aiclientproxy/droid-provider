//! WorkOS OAuth 认证模块
//!
//! 实现 Factory.ai Droid 的 WorkOS OAuth 认证流程

#![allow(dead_code)]

use crate::credentials::WorkOSTokenResponse;
use anyhow::Result;
use chrono::{Duration, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// WorkOS OAuth 配置
pub const WORKOS_CLIENT_ID: &str = "client_01HNM792M5G5G1A2THWPXKFMXB";
pub const WORKOS_TOKEN_URL: &str = "https://api.workos.com/user_management/authenticate";
pub const FACTORY_CLI_ORG_URL: &str = "https://app.factory.ai/api/cli/org";
pub const FACTORY_USER_AGENT: &str = "factory-cli/0.32.1";

/// Token 刷新结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRefreshResult {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<chrono::DateTime<Utc>>,
    pub organization_id: Option<String>,
    pub user_id: Option<String>,
    pub owner_email: Option<String>,
}

/// 使用 Refresh Token 刷新 Access Token
pub async fn refresh_workos_token(
    refresh_token: &str,
    organization_id: Option<&str>,
) -> Result<TokenRefreshResult> {
    let client = Client::builder()
        .connect_timeout(std::time::Duration::from_secs(30))
        .timeout(std::time::Duration::from_secs(60))
        .build()?;

    debug!("刷新 WorkOS Token");

    // 构建表单数据
    let mut form = vec![
        ("grant_type", "refresh_token".to_string()),
        ("refresh_token", refresh_token.to_string()),
        ("client_id", WORKOS_CLIENT_ID.to_string()),
    ];

    if let Some(org_id) = organization_id {
        form.push(("organization_id", org_id.to_string()));
    }

    let response = client
        .post(WORKOS_TOKEN_URL)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&form)
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("WorkOS Token 刷新失败: {} - {}", status, body);
    }

    let token_response: WorkOSTokenResponse = response.json().await?;

    // 计算过期时间
    let expires_at = if let Some(expires_at_str) = &token_response.expires_at {
        chrono::DateTime::parse_from_rfc3339(expires_at_str)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    } else if let Some(expires_in) = token_response.expires_in {
        Some(Utc::now() + Duration::seconds(expires_in))
    } else {
        // 默认 8 小时
        Some(Utc::now() + Duration::hours(8))
    };

    info!("WorkOS Token 刷新成功");

    Ok(TokenRefreshResult {
        access_token: token_response.access_token,
        refresh_token: token_response.refresh_token,
        expires_at,
        organization_id: token_response.organization_id,
        user_id: token_response.user.as_ref().and_then(|u| u.id.clone()),
        owner_email: token_response.user.as_ref().and_then(|u| u.email.clone()),
    })
}

/// 获取 Factory 组织 ID 列表
pub async fn fetch_factory_org_ids(access_token: &str) -> Result<Vec<String>> {
    let client = Client::builder()
        .connect_timeout(std::time::Duration::from_secs(15))
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    debug!("获取 Factory 组织信息");

    let response = client
        .get(FACTORY_CLI_ORG_URL)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("x-factory-client", "cli")
        .header("User-Agent", FACTORY_USER_AGENT)
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("获取 Factory 组织信息失败: {} - {}", status, body);
    }

    #[derive(Deserialize)]
    struct OrgResponse {
        #[serde(rename = "workosOrgIds")]
        workos_org_ids: Option<Vec<String>>,
    }

    let org_response: OrgResponse = response.json().await?;

    Ok(org_response.workos_org_ids.unwrap_or_default())
}

/// 验证 Access Token 是否有效
pub async fn validate_access_token(access_token: &str) -> Result<bool> {
    // 尝试获取组织信息来验证 Token
    match fetch_factory_org_ids(access_token).await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert!(!WORKOS_CLIENT_ID.is_empty());
        assert!(WORKOS_TOKEN_URL.starts_with("https://"));
        assert!(FACTORY_CLI_ORG_URL.starts_with("https://"));
    }
}
