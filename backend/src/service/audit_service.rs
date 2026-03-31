// 审计服务
//
// 提供结构化的审计日志写入接口

use sqlx::PgPool;
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    model::{AuditLog, AuditLogQuery, CreateAuditLog},
    repo::AuditLogRepo,
};

pub struct AuditService;

impl AuditService {
    /// 记录登录成功事件
    pub async fn log_login_success(
        pool: &PgPool,
        user_id: Uuid,
        session_id: &str,
        correlation_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<AuditLog, sqlx::Error> {
        let create_log = CreateAuditLog::success(
            "login",
            "user_session",
            session_id,
        )
        .with_actor(user_id)
        .with_correlation_id(correlation_id.unwrap_or_default())
        .with_ip(ip_address.unwrap_or_else(|| "unknown".to_string()))
        .with_user_agent(user_agent.unwrap_or_else(|| "unknown".to_string()))
        .with_metadata(serde_json::json!({
            "event": "login_success"
        }));

        let log = AuditLogRepo::create(pool, create_log).await?;
        info!(
            "Audit log: login success for user {} (session: {})",
            user_id, session_id
        );
        Ok(log)
    }

    /// 记录登录失败事件
    pub async fn log_login_failure(
        pool: &PgPool,
        username: &str,
        reason_code: &str,
        correlation_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<AuditLog, sqlx::Error> {
        let create_log = CreateAuditLog::failure(
            "login",
            "user_session",
            username,
            reason_code,
        )
        .with_correlation_id(correlation_id.unwrap_or_default())
        .with_ip(ip_address.unwrap_or_else(|| "unknown".to_string()))
        .with_user_agent(user_agent.unwrap_or_else(|| "unknown".to_string()))
        .with_metadata(serde_json::json!({
            "event": "login_failure",
            "username": username
        }));

        let log = AuditLogRepo::create(pool, create_log).await?;
        info!(
            "Audit log: login failure for user {} (reason: {})",
            username, reason_code
        );
        Ok(log)
    }

    /// 记录登出事件
    pub async fn log_logout(
        pool: &PgPool,
        user_id: Option<Uuid>,
        session_id: &str,
        correlation_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<AuditLog, sqlx::Error> {
        let mut create_log = CreateAuditLog::success(
            "logout",
            "user_session",
            session_id,
        )
        .with_correlation_id(correlation_id.unwrap_or_default())
        .with_ip(ip_address.unwrap_or_else(|| "unknown".to_string()))
        .with_user_agent(user_agent.unwrap_or_else(|| "unknown".to_string()));

        if let Some(uid) = user_id {
            create_log = create_log.with_actor(uid);
        }

        let log = AuditLogRepo::create(pool, create_log).await?;
        info!(
            "Audit log: logout for session {} (user: {:?})",
            session_id, user_id
        );
        Ok(log)
    }

    /// 记录 Token 发放事件
    pub async fn log_token_issued(
        pool: &PgPool,
        user_id: Option<Uuid>,
        client_id: Option<Uuid>,
        token_type: &str,
        correlation_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<AuditLog, sqlx::Error> {
        let mut create_log = CreateAuditLog::success(
            "token_issued",
            token_type,
            Uuid::new_v4().to_string(),
        )
        .with_correlation_id(correlation_id.unwrap_or_default())
        .with_ip(ip_address.unwrap_or_else(|| "unknown".to_string()))
        .with_user_agent(user_agent.unwrap_or_else(|| "unknown".to_string()))
        .with_metadata(serde_json::json!({
            "event": "token_issued",
            "token_type": token_type
        }));

        if let Some(uid) = user_id {
            create_log = create_log.with_actor(uid);
        }
        if let Some(cid) = client_id {
            create_log = create_log.with_client(cid);
        }

        let log = AuditLogRepo::create(pool, create_log).await?;
        info!(
            "Audit log: token issued (type: {}, user: {:?}, client: {:?})",
            token_type, user_id, client_id
        );
        Ok(log)
    }

    /// 记录 Token 刷新事件
    pub async fn log_token_refresh(
        pool: &PgPool,
        user_id: Option<Uuid>,
        client_id: Option<Uuid>,
        correlation_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<AuditLog, sqlx::Error> {
        let mut create_log = CreateAuditLog::success(
            "token_refresh",
            "refresh_token",
            Uuid::new_v4().to_string(),
        )
        .with_correlation_id(correlation_id.unwrap_or_default())
        .with_ip(ip_address.unwrap_or_else(|| "unknown".to_string()))
        .with_user_agent(user_agent.unwrap_or_else(|| "unknown".to_string()))
        .with_metadata(serde_json::json!({
            "event": "token_refresh"
        }));

        if let Some(uid) = user_id {
            create_log = create_log.with_actor(uid);
        }
        if let Some(cid) = client_id {
            create_log = create_log.with_client(cid);
        }

        let log = AuditLogRepo::create(pool, create_log).await?;
        Ok(log)
    }

    /// 记录 Refresh Token 重放检测事件
    pub async fn log_replay_detected(
        pool: &PgPool,
        token_hash: &str,
        correlation_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<AuditLog, sqlx::Error> {
        let create_log = CreateAuditLog::failure(
            "token_replay_detected",
            "refresh_token",
            token_hash,
            "token_replay",
        )
        .with_correlation_id(correlation_id.unwrap_or_default())
        .with_ip(ip_address.unwrap_or_else(|| "unknown".to_string()))
        .with_user_agent(user_agent.unwrap_or_else(|| "unknown".to_string()))
        .with_metadata(serde_json::json!({
            "event": "replay_detected"
        }));

        let log = AuditLogRepo::create(pool, create_log).await?;
        error!(
            "Audit log: refresh token replay detected (token: {})",
            token_hash
        );
        Ok(log)
    }

    /// 记录 Authorization Code 重放检测事件
    pub async fn log_auth_code_replay(
        pool: &PgPool,
        code_hash: &str,
        correlation_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<AuditLog, sqlx::Error> {
        let create_log = CreateAuditLog::failure(
            "auth_code_replay_detected",
            "authorization_code",
            code_hash,
            "code_replay",
        )
        .with_correlation_id(correlation_id.unwrap_or_default())
        .with_ip(ip_address.unwrap_or_else(|| "unknown".to_string()))
        .with_user_agent(user_agent.unwrap_or_else(|| "unknown".to_string()))
        .with_metadata(serde_json::json!({
            "event": "auth_code_replay_detected"
        }));

        let log = AuditLogRepo::create(pool, create_log).await?;
        error!(
            "Audit log: authorization code replay detected (code_hash: {})",
            code_hash
        );
        Ok(log)
    }

    /// 记录授权请求事件
    pub async fn log_authorization_request(
        pool: &PgPool,
        user_id: Option<Uuid>,
        client_id: Option<Uuid>,
        scopes: &[String],
        correlation_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<AuditLog, sqlx::Error> {
        let mut create_log = CreateAuditLog::success(
            "authorization_request",
            "authorization_code",
            Uuid::new_v4().to_string(),
        )
        .with_correlation_id(correlation_id.unwrap_or_default())
        .with_ip(ip_address.unwrap_or_else(|| "unknown".to_string()))
        .with_user_agent(user_agent.unwrap_or_else(|| "unknown".to_string()))
        .with_metadata(serde_json::json!({
            "event": "authorization_request",
            "scopes": scopes
        }));

        if let Some(uid) = user_id {
            create_log = create_log.with_actor(uid);
        }
        if let Some(cid) = client_id {
            create_log = create_log.with_client(cid);
        }

        let log = AuditLogRepo::create(pool, create_log).await?;
        Ok(log)
    }

    /// 记录同意授权事件
    pub async fn log_consent_granted(
        pool: &PgPool,
        user_id: Uuid,
        client_id: Uuid,
        scopes: &[String],
        correlation_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<AuditLog, sqlx::Error> {
        let create_log = CreateAuditLog::success(
            "consent_granted",
            "user_consent",
            format!("{}:{}", user_id, client_id),
        )
        .with_actor(user_id)
        .with_client(client_id)
        .with_correlation_id(correlation_id.unwrap_or_default())
        .with_ip(ip_address.unwrap_or_else(|| "unknown".to_string()))
        .with_user_agent(user_agent.unwrap_or_else(|| "unknown".to_string()))
        .with_metadata(serde_json::json!({
            "event": "consent_granted",
            "scopes": scopes
        }));

        let log = AuditLogRepo::create(pool, create_log).await?;
        info!(
            "Audit log: consent granted by user {} for client {}",
            user_id, client_id
        );
        Ok(log)
    }

    /// 记录同意拒绝事件
    pub async fn log_consent_denied(
        pool: &PgPool,
        user_id: Uuid,
        client_id: Uuid,
        correlation_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<AuditLog, sqlx::Error> {
        let create_log = CreateAuditLog::failure(
            "consent_denied",
            "user_consent",
            format!("{}:{}", user_id, client_id),
            "user_denied",
        )
        .with_actor(user_id)
        .with_client(client_id)
        .with_correlation_id(correlation_id.unwrap_or_default())
        .with_ip(ip_address.unwrap_or_else(|| "unknown".to_string()))
        .with_user_agent(user_agent.unwrap_or_else(|| "unknown".to_string()))
        .with_metadata(serde_json::json!({
            "event": "consent_denied"
        }));

        let log = AuditLogRepo::create(pool, create_log).await?;
        info!(
            "Audit log: consent denied by user {} for client {}",
            user_id, client_id
        );
        Ok(log)
    }

    /// 记录账户锁定事件
    pub async fn log_account_locked(
        pool: &PgPool,
        user_id: Uuid,
        reason: &str,
        correlation_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<AuditLog, sqlx::Error> {
        let create_log = CreateAuditLog::failure(
            "account_locked",
            "user_account",
            user_id.to_string(),
            reason,
        )
        .with_actor(user_id)
        .with_correlation_id(correlation_id.unwrap_or_default())
        .with_ip(ip_address.unwrap_or_else(|| "unknown".to_string()))
        .with_user_agent(user_agent.unwrap_or_else(|| "unknown".to_string()))
        .with_metadata(serde_json::json!({
            "event": "account_locked",
            "reason": reason
        }));

        let log = AuditLogRepo::create(pool, create_log).await?;
        info!(
            "Audit log: account locked for user {} (reason: {})",
            user_id, reason
        );
        Ok(log)
    }

    /// 查询审计日志
    pub async fn query_logs(
        pool: &PgPool,
        query: AuditLogQuery,
    ) -> Result<Vec<AuditLog>, sqlx::Error> {
        AuditLogRepo::query(pool, query).await
    }
}
