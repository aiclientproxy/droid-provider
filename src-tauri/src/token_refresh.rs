//! Token 刷新逻辑
//!
//! 支持 WorkOS OAuth Token 刷新

#![allow(dead_code)]

use crate::auth::workos::refresh_workos_token;
use crate::credentials::{AuthType, DroidCredentials};
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Token 刷新结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRefreshResult {
    /// 新的 access_token
    pub access_token: String,
    /// 新的 refresh_token（如果更新了）
    #[serde(default)]
    pub refresh_token: Option<String>,
    /// 过期时间
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
    /// 组织 ID
    #[serde(default)]
    pub organization_id: Option<String>,
}

/// 刷新 Token
pub async fn refresh_token(credential: &mut DroidCredentials) -> Result<TokenRefreshResult> {
    match credential.auth_type {
        AuthType::OAuth => refresh_oauth_token(credential).await,
        AuthType::ApiKey => {
            // API Key 不需要刷新
            anyhow::bail!("API Key 认证不需要刷新 Token")
        }
    }
}

/// 刷新 OAuth Token
async fn refresh_oauth_token(credential: &mut DroidCredentials) -> Result<TokenRefreshResult> {
    let refresh_token = credential
        .refresh_token
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("缺少 refresh_token"))?;

    info!("开始刷新 Droid OAuth Token");

    let result = refresh_workos_token(
        refresh_token,
        credential.organization_id.as_deref(),
    )
    .await?;

    // 更新凭证
    credential.access_token = Some(result.access_token.clone());
    if let Some(ref rt) = result.refresh_token {
        credential.refresh_token = Some(rt.clone());
    }
    credential.expires_at = result.expires_at.map(|dt| dt.to_rfc3339());
    credential.last_refresh = Some(Utc::now().to_rfc3339());
    credential.is_healthy = true;
    credential.last_error = None;

    if let Some(ref org_id) = result.organization_id {
        credential.organization_id = Some(org_id.clone());
    }
    if let Some(ref user_id) = result.user_id {
        credential.user_id = Some(user_id.clone());
    }
    if let Some(ref email) = result.owner_email {
        credential.owner_email = Some(email.clone());
    }

    info!("Droid OAuth Token 刷新成功");

    Ok(TokenRefreshResult {
        access_token: result.access_token,
        refresh_token: result.refresh_token,
        expires_at: result.expires_at,
        organization_id: result.organization_id,
    })
}

/// 检查 Token 是否已过期
pub fn is_token_expired(expires_at: Option<&str>) -> bool {
    if let Some(expires_str) = expires_at {
        if let Ok(expires) = DateTime::parse_from_rfc3339(expires_str) {
            let now = Utc::now();
            // 提前 5 分钟判断为过期
            return expires <= now + Duration::minutes(5);
        }
    }
    // 如果没有过期时间信息，保守地认为可能需要刷新
    true
}

/// 检查 Token 是否即将过期（1 小时内）
pub fn is_token_expiring_soon(expires_at: Option<&str>) -> bool {
    if let Some(expires_str) = expires_at {
        if let Ok(expiry) = DateTime::parse_from_rfc3339(expires_str) {
            let now = Utc::now();
            let threshold = now + Duration::hours(1);
            return expiry < threshold;
        }
    }
    false
}

/// 带重试的 Token 刷新
pub async fn refresh_token_with_retry(
    credential: &mut DroidCredentials,
    max_retries: u32,
) -> Result<TokenRefreshResult> {
    let mut last_error = None;

    for attempt in 0..max_retries {
        match refresh_token(credential).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                warn!(
                    "Token 刷新失败 (尝试 {}/{}): {}",
                    attempt + 1,
                    max_retries,
                    e
                );
                last_error = Some(e);
                // 指数退避
                let delay = std::time::Duration::from_millis(1000 * 2_u64.pow(attempt));
                tokio::time::sleep(delay).await;
            }
        }
    }

    Err(last_error.unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_token_expired() {
        // 已过期
        let expired = (Utc::now() - Duration::hours(1)).to_rfc3339();
        assert!(is_token_expired(Some(&expired)));

        // 即将过期（5分钟内）
        let expiring = (Utc::now() + Duration::minutes(3)).to_rfc3339();
        assert!(is_token_expired(Some(&expiring)));

        // 未过期
        let valid = (Utc::now() + Duration::hours(1)).to_rfc3339();
        assert!(!is_token_expired(Some(&valid)));

        // 无过期时间
        assert!(is_token_expired(None));
    }

    #[test]
    fn test_is_token_expiring_soon() {
        // 1小时内过期
        let expiring = (Utc::now() + Duration::minutes(30)).to_rfc3339();
        assert!(is_token_expiring_soon(Some(&expiring)));

        // 超过1小时
        let valid = (Utc::now() + Duration::hours(2)).to_rfc3339();
        assert!(!is_token_expiring_soon(Some(&valid)));
    }
}
