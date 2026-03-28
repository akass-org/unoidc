// 认证服务
//
// 处理用户登录、登出、会话管理等认证相关业务逻辑

use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    crypto::password,
    error::{AppError, Result},
    model::{CreateSession, Session, User},
    repo::{SessionRepo, UserRepo},
};

/// 登录配置常量
pub const MAX_FAILED_ATTEMPTS: i32 = 5;          // 最大失败次数
pub const LOCKOUT_DURATION_SECONDS: i64 = 1800;  // 锁定时长（30分钟）
pub const DEFAULT_SESSION_DURATION: i64 = 86400; // 默认会话时长（24小时）

/// 认证服务
pub struct AuthService;

impl AuthService {
    /// 用户登录
    ///
    /// 验证用户凭证并创建会话
    /// 如果登录失败，增加失败计数
    /// 如果失败次数达到阈值，锁定账户
    pub async fn login(
        pool: &PgPool,
        username: &str,
        password: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(User, Session)> {
        // 1. 查找用户
        let user = UserRepo::find_by_username(pool, username)
            .await
            .map_err(|e| {
                warn!("Database error while finding user: {}", e);
                AppError::InternalServerError
            })?
            .ok_or_else(|| {
                info!("Login failed: user not found - {}", username);
                AppError::InvalidCredentials
            })?;

        // 2. 检查账户是否被锁定
        if user.is_locked() {
            warn!("Login attempt on locked account: {}", username);
            return Err(AppError::Forbidden);
        }

        // 3. 检查账户是否启用
        if !user.enabled {
            warn!("Login attempt on disabled account: {}", username);
            return Err(AppError::Forbidden);
        }

        // 4. 验证密码
        let password_valid = password::verify_password(password, &user.password_hash)
            .map_err(|e| {
                warn!("Password verification error: {}", e);
                AppError::InternalServerError
            })?;

        if !password_valid {
            // 密码错误，增加失败计数
            let failed_attempts = UserRepo::increment_failed_login(pool, user.id)
                .await
                .map_err(|e| {
                    warn!("Failed to increment login attempts: {}", e);
                    AppError::InternalServerError
                })?;

            info!(
                "Login failed for user {}: attempt {}/{}",
                username, failed_attempts, MAX_FAILED_ATTEMPTS
            );

            // 检查是否需要锁定账户
            if failed_attempts >= MAX_FAILED_ATTEMPTS {
                let lockout_until = OffsetDateTime::now_utc()
                    + time::Duration::seconds(LOCKOUT_DURATION_SECONDS);

                UserRepo::lock_account(pool, user.id, lockout_until)
                    .await
                    .map_err(|e| {
                        warn!("Failed to lock account: {}", e);
                        AppError::InternalServerError
                    })?;

                warn!(
                    "Account locked for user {} until {:?}",
                    username, lockout_until
                );

                return Err(AppError::Forbidden);
            }

            return Err(AppError::InvalidCredentials);
        }

        // 5. 登录成功！重置失败计数并更新最后登录时间
        UserRepo::reset_failed_login(pool, user.id)
            .await
            .map_err(|e| {
                warn!("Failed to reset login attempts: {}", e);
                AppError::InternalServerError
            })?;

        UserRepo::update_last_login(pool, user.id)
            .await
            .map_err(|e| {
                warn!("Failed to update last login time: {}", e);
                AppError::InternalServerError
            })?;

        info!("User logged in successfully: {}", username);

        // 6. 创建会话
        let session_input = CreateSession::new(user.id, ip_address, user_agent);
        let session = SessionRepo::create(pool, session_input)
            .await
            .map_err(|e| {
                warn!("Failed to create session: {}", e);
                AppError::InternalServerError
            })?;

        info!("Session created for user {}: {}", username, session.session_id);

        Ok((user, session))
    }

    /// 用户登出
    ///
    /// 删除指定会话
    pub async fn logout(pool: &PgPool, session_id: &str) -> Result<()> {
        SessionRepo::delete(pool, session_id)
            .await
            .map_err(|e| {
                warn!("Failed to delete session: {}", e);
                AppError::InternalServerError
            })?;

        info!("Session logged out: {}", session_id);

        Ok(())
    }

    /// 验证会话
    ///
    /// 检查会话是否有效并更新最后访问时间
    pub async fn validate_session(pool: &PgPool, session_id: &str) -> Result<Option<(User, Session)>> {
        // 查找会话
        let session = match SessionRepo::find_by_session_id(pool, session_id).await {
            Ok(Some(s)) if s.is_valid() => s,
            Ok(Some(_)) => {
                // 会话已过期，删除它
                warn!("Session expired: {}", session_id);
                let _ = SessionRepo::delete(pool, session_id).await;
                return Ok(None);
            }
            Ok(None) => {
                // 会话不存在
                return Ok(None);
            }
            Err(e) => {
                warn!("Database error while finding session: {}", e);
                return Err(AppError::InternalServerError);
            }
        };

        // 查找用户
        let user = match UserRepo::find_by_id(pool, session.user_id).await {
            Ok(Some(u)) if u.can_login() => u,
            Ok(Some(u)) => {
                // 用户无法登录（被禁用或锁定）
                warn!("User cannot login: enabled={}, locked={}", u.enabled, u.is_locked());
                return Ok(None);
            }
            Ok(None) => {
                // 用户不存在
                warn!("User not found for session: {}", session_id);
                return Ok(None);
            }
            Err(e) => {
                warn!("Database error while finding user: {}", e);
                return Err(AppError::InternalServerError);
            }
        };

        // 更新会话最后访问时间
        if let Err(e) = SessionRepo::touch(pool, session_id).await {
            warn!("Failed to update session last seen time: {}", e);
            // 不返回错误，因为会话验证本身是成功的
        }

        Ok(Some((user, session)))
    }

    /// 登出用户的所有会话
    ///
    /// 删除指定用户的所有活跃会话（用于强制登出或密码重置等场景）
    pub async fn logout_all_sessions(pool: &PgPool, user_id: Uuid) -> Result<u64> {
        let count = SessionRepo::delete_user_sessions(pool, user_id)
            .await
            .map_err(|e| {
                warn!("Failed to delete user sessions: {}", e);
                AppError::InternalServerError
            })?;

        info!("Logged out {} sessions for user {}", count, user_id);

        Ok(count)
    }
}
