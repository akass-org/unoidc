// Me Handler
//
// 用户自助 API 接口（当前登录用户）

use axum::{
    extract::{Multipart, Path, State},
    http::HeaderMap,
    response::IntoResponse,
    Json,
};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use time::format_description::well_known::Rfc3339;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::{AppError, Result},
    middleware::auth::{require_auth_user, AuthUser},
    model::UpdateUser,
    repo::RefreshTokenRepo,
    service::{AuditService, AuthService, EmailVerificationService, UserService},
    AppState,
};

// ============================================================================
// 用户资料
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ProfileResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
    pub email_verified: bool,
    pub is_admin: bool,
}

impl From<(crate::model::User, bool)> for ProfileResponse {
    fn from((user, is_admin): (crate::model::User, bool)) -> Self {
        Self {
            id: user.id.to_string(),
            username: user.username,
            email: user.email,
            display_name: user.display_name.unwrap_or_default(),
            given_name: user.given_name,
            family_name: user.family_name,
            picture: user.picture,
            email_verified: user.email_verified,
            is_admin,
        }
    }
}

/// 获取当前用户信息
pub async fn get_profile(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<ProfileResponse>> {
    let auth_user = require_auth_user(&state.db, &headers, &state.config.session_secret).await?;
    let is_admin = check_is_admin(&state.db, &auth_user).await?;

    Ok(Json(ProfileResponse::from((auth_user.user, is_admin))))
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProfileRequest {
    #[validate(length(max = 100, message = "显示名称不能超过100个字符"))]
    pub display_name: Option<String>,
    #[validate(email(message = "邮箱格式不正确"))]
    pub email: Option<String>,
}

/// 更新当前用户信息
pub async fn update_profile(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<ProfileResponse>> {
    let auth_user = require_auth_user(&state.db, &headers, &state.config.session_secret).await?;

    req.validate().map_err(|e| AppError::ValidationError {
        field: "request".to_string(),
        message: e.to_string(),
    })?;

    let update = UpdateUser {
        display_name: req.display_name,
        given_name: None,
        family_name: None,
        picture: None,
        email_verified: None,
        enabled: None,
    };

    let user = UserService::update_user(&state.db, auth_user.user.id, update)
        .await
        .map_err(|e| AppError::BusinessError {
            code: "PROFILE_UPDATE_FAILED".to_string(),
            message: e.to_string(),
        })?;

    // TODO: 更新邮箱（需要验证流程）

    let is_admin = check_is_admin(&state.db, &auth_user).await?;
    Ok(Json(ProfileResponse::from((user, is_admin))))
}

// ============================================================================
// 密码修改
// ============================================================================

#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 1, message = "当前密码不能为空"))]
    pub current_password: String,
    #[validate(length(min = 8, max = 128, message = "新密码长度必须在 8-128 之间"))]
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct ChangePasswordResponse {
    pub success: bool,
    pub message: String,
}

/// 修改当前用户密码
pub async fn change_password(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<ChangePasswordResponse>> {
    let auth_user = require_auth_user(&state.db, &headers, &state.config.session_secret).await?;

    req.validate().map_err(|e| AppError::ValidationError {
        field: "request".to_string(),
        message: e.to_string(),
    })?;

    UserService::change_password(
        &state.db,
        auth_user.user.id,
        &req.current_password,
        &req.new_password,
    )
    .await
    .map_err(|e| {
        if e.to_string().contains("Invalid old password") {
            AppError::InvalidCredentials
        } else {
            AppError::BusinessError {
                code: "PASSWORD_CHANGE_FAILED".to_string(),
                message: e.to_string(),
            }
        }
    })?;

    // 密码修改后，强制所有客户端重新登录：清理会话并撤销刷新令牌
    AuthService::logout_all_sessions(&state.db, auth_user.user.id)
        .await
        .map_err(|_| AppError::InternalServerError {
            error_code: Some("SESSION_REVOKE_FAILED".to_string()),
        })?;

    RefreshTokenRepo::revoke_all_for_user(&state.db, auth_user.user.id)
        .await
        .map_err(|_| AppError::InternalServerError {
            error_code: Some("TOKEN_REVOKE_FAILED".to_string()),
        })?;

    // 记录审计日志
    let ip_address = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let _ = AuditService::log_password_change(
        &state.db,
        auth_user.user.id,
        None,
        ip_address,
        user_agent,
    )
    .await;

    Ok(Json(ChangePasswordResponse {
        success: true,
        message: "密码修改成功".to_string(),
    }))
}

// ============================================================================
// 头像上传
// ============================================================================

/// 上传用户头像
pub async fn upload_avatar(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<ProfileResponse>> {
    let auth_user = require_auth_user(&state.db, &headers, &state.config.session_secret).await?;

    // 提取 avatar 字段
    let field: axum::extract::multipart::Field = multipart
        .next_field()
        .await
        .map_err(|_| AppError::InvalidRequest("Failed to parse upload".to_string()))?
        .ok_or_else(|| AppError::InvalidRequest("No avatar file provided".to_string()))?;

    let content_type = field
        .content_type()
        .unwrap_or("application/octet-stream")
        .to_string();

    let data: Vec<u8> = field
        .bytes()
        .await
        .map_err(|_| AppError::InvalidRequest("Failed to read file data".to_string()))?
        .to_vec();

    // 验证大小（最大 1MB）
    const MAX_SIZE: usize = 1024 * 1024;
    if data.len() > MAX_SIZE {
        tracing::warn!(
            user = %auth_user.user.username,
            size_kb = data.len() / 1024,
            ct = %content_type,
            "Avatar upload rejected: image too large"
        );
        return Err(AppError::ValidationError {
            field: "avatar".to_string(),
            message: format!("图片过大 ({}KB)，请选择小于 1MB 的图片", data.len() / 1024),
        });
    }

    // 验证格式
    let valid_types = ["image/jpeg", "image/png", "image/webp", "image/gif"];
    if !valid_types.contains(&content_type.as_str()) {
        return Err(AppError::ValidationError {
            field: "avatar".to_string(),
            message: "仅支持 JPEG、PNG、WebP、GIF 格式".to_string(),
        });
    }

    // 解码、缩放至 256x256、重新编码为 JPEG
    let picture = {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let img = image::load_from_memory(&data).map_err(|_| {
                AppError::ValidationError {
                    field: "avatar".to_string(),
                    message: "无效的图片文件".to_string(),
                }
            })?;

            let resized = img.resize(256, 256, image::imageops::FilterType::Lanczos3);
            let mut buffer = Vec::new();
            resized
                .write_to(
                    &mut std::io::Cursor::new(&mut buffer),
                    image::ImageFormat::Jpeg,
                )
                .map_err(|e| AppError::InternalServerError {
                    error_code: Some(format!("IMAGE_ENCODE_ERROR: {}", e)),
                })?;

            let b64_data = base64::engine::general_purpose::STANDARD.encode(&buffer);
            Ok::<_, AppError>(format!("data:image/jpeg;base64,{}", b64_data))
        }))
    };

    let picture = match picture {
        Ok(Ok(pic)) => pic,
        Ok(Err(e)) => return Err(e),
        Err(_) => {
            return Err(AppError::ValidationError {
                field: "avatar".to_string(),
                message: "图片处理失败，可能是文件损坏".to_string(),
            });
        }
    };

    // 更新用户记录
    crate::repo::UserRepo::update_picture(&state.db, auth_user.user.id, &picture)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update avatar: {}", e);
            AppError::InternalServerError {
                error_code: Some("AVATAR_UPDATE_ERROR".to_string()),
            }
        })?;

    // 返回更新后的用户信息
    let updated_user = crate::repo::UserRepo::find_by_id(&state.db, auth_user.user.id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch updated user: {}", e);
            AppError::InternalServerError {
                error_code: Some("USER_FETCH_ERROR".to_string()),
            }
        })?
        .ok_or(AppError::UserNotFound {
            identifier: Some(auth_user.user.id.to_string()),
        })?;

    let is_admin = check_is_admin(&state.db, &auth_user).await?;
    Ok(Json(ProfileResponse::from((updated_user, is_admin))))
}

// ============================================================================
// 已授权应用
// ============================================================================

#[derive(Debug, Serialize)]
pub struct AppResponse {
    pub client_id: String,
    pub client_name: String,
    pub description: Option<String>,
    pub granted_at: Option<String>,
    pub scopes: Vec<String>,
    pub access_source: String,
}

/// 获取当前用户已授权的应用列表
pub async fn get_apps(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<AppResponse>>> {
    let auth_user = require_auth_user(&state.db, &headers, &state.config.session_secret).await?;

    // 获取用户的所有同意记录
    let consents = crate::repo::ConsentRepo::find_user_consents(&state.db, auth_user.user.id)
        .await
        .map_err(|e| {
            tracing::error!("Database error while fetching user apps consents: {}", e);
            AppError::InternalServerError {
                error_code: Some("CONSENTS_FETCH_ERROR".to_string()),
            }
        })?;

    let mut apps_by_client_id: HashMap<String, AppResponse> = HashMap::new();

    for consent in consents {
        // 获取客户端信息
        if let Ok(Some(client)) =
            crate::repo::ClientRepo::find_by_id(&state.db, consent.client_id).await
        {
            let scopes: Vec<String> = consent.scope.split_whitespace().map(|s| s.to_string()).collect();
            apps_by_client_id.insert(client.client_id.clone(), AppResponse {
                client_id: client.client_id,
                client_name: client.name,
                description: client.description,
                granted_at: Some(consent.granted_at.to_string()),
                scopes,
                access_source: "consent".to_string(),
            });
        }
    }

    let visible_clients = crate::repo::ClientRepo::find_accessible_clients_for_user(&state.db, auth_user.user.id)
        .await
        .map_err(|e| {
            tracing::error!("Database error while fetching visible apps: {}", e);
            AppError::InternalServerError {
                error_code: Some("VISIBLE_APPS_FETCH_ERROR".to_string()),
            }
        })?;

    for client in visible_clients {
        apps_by_client_id.entry(client.client_id.clone()).or_insert_with(|| AppResponse {
            client_id: client.client_id,
            client_name: client.name,
            description: client.description,
            granted_at: None,
            scopes: Vec::new(),
            access_source: "group".to_string(),
        });
    }

    let mut apps: Vec<AppResponse> = apps_by_client_id.into_values().collect();
    apps.sort_by(|left, right| match (&left.granted_at, &right.granted_at) {
        (Some(left_granted_at), Some(right_granted_at)) => right_granted_at.cmp(left_granted_at),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => left.client_name.cmp(&right.client_name),
    });

    Ok(Json(apps))
}

// ============================================================================
// 审计日志
// ============================================================================

#[derive(Debug, sqlx::FromRow)]
struct AuditLogRow {
    id: uuid::Uuid,
    actor_user_id: Option<uuid::Uuid>,
    username: Option<String>,
    client_id: Option<uuid::Uuid>,
    client_name: Option<String>,
    action: String,
    outcome: String,
    reason_code: Option<String>,
    ip_address: Option<String>,
    user_agent: Option<String>,
    created_at: time::OffsetDateTime,
}

#[derive(Debug, Serialize)]
pub struct AuditLogResponse {
    pub id: String,
    pub event_type: String,
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub client_id: Option<String>,
    pub client_name: Option<String>,
    pub ip_address: String,
    pub user_agent: String,
    pub outcome: String,
    pub reason: Option<String>,
    pub created_at: String,
}

/// 获取当前用户的审计日志
pub async fn get_audit_logs(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<AuditLogResponse>>> {
    let auth_user = require_auth_user(&state.db, &headers, &state.config.session_secret).await?;

    let logs = sqlx::query_as::<_, AuditLogRow>(
        r#"
        SELECT
            al.id,
            al.actor_user_id,
            u.username,
            al.client_id,
            c.name as client_name,
            al.action,
            al.outcome,
            al.reason_code,
            al.ip_address,
            al.user_agent,
            al.created_at
        FROM audit_logs al
        LEFT JOIN users u ON al.actor_user_id = u.id
        LEFT JOIN clients c ON al.client_id = c.id
        WHERE al.actor_user_id = $1
           OR (al.action = 'login' AND al.target_id = $2)
           OR (al.action = 'registration_failure' AND al.target_id = $2)
        ORDER BY al.created_at DESC
        LIMIT 200
        "#,
    )
    .bind(auth_user.user.id)
    .bind(&auth_user.user.username)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error while querying user audit logs: {}", e);
        AppError::InternalServerError {
            error_code: Some("AUDIT_LOGS_FETCH_ERROR".to_string()),
        }
    })?;

    let responses: Vec<AuditLogResponse> = logs
        .into_iter()
        .map(|row| {
            let event_type = match (row.action.as_str(), row.outcome.as_str()) {
                ("login", "success") => "login_success",
                ("login", _) => "login_failure",
                ("logout", _) => "logout",
                ("token_issued", _) => "token_issued",
                ("token_refresh", _) => "token_refresh",
                ("consent_granted", _) => "consent_granted",
                ("consent_denied", _) => "consent_revoked",
                ("user_created", _) => "user_created",
                ("password_reset", _) => "password_reset",
                ("registration_failure", _) => "registration_failure",
                ("email_changed", _) => "email_changed",
                _ => row.action.as_str(),
            }
            .to_string();

            AuditLogResponse {
                id: row.id.to_string(),
                event_type,
                user_id: row.actor_user_id.map(|id| id.to_string()),
                username: row.username,
                client_id: row.client_id.map(|id| id.to_string()),
                client_name: row.client_name,
                ip_address: row.ip_address.unwrap_or_else(|| "unknown".to_string()),
                user_agent: row.user_agent.unwrap_or_else(|| "unknown".to_string()),
                outcome: row.outcome,
                reason: row.reason_code,
                created_at: row.created_at.format(&Rfc3339).unwrap_or_else(|_| row.created_at.to_string()),
            }
        })
        .collect();

    Ok(Json(responses))
}

// ============================================================================
// 授权管理
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ConsentResponse {
    pub client_id: String,
    pub client_name: String,
    pub scopes: Vec<String>,
    pub granted_at: String,
}

/// 获取当前用户的所有授权记录
pub async fn get_consents(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<ConsentResponse>>> {
    let auth_user = require_auth_user(&state.db, &headers, &state.config.session_secret).await?;

    let consents = crate::repo::ConsentRepo::find_user_consents(&state.db, auth_user.user.id)
        .await
        .map_err(|e| {
            tracing::error!("Database error while fetching user consents: {}", e);
            AppError::InternalServerError {
                error_code: Some("CONSENTS_FETCH_ERROR".to_string()),
            }
        })?;

    let mut responses = Vec::new();
    for consent in consents {
        if let Ok(Some(client)) =
            crate::repo::ClientRepo::find_by_id(&state.db, consent.client_id).await
        {
            let scopes: Vec<String> = consent.scope.split_whitespace().map(|s| s.to_string()).collect();
            responses.push(ConsentResponse {
                client_id: client.client_id,
                client_name: client.name,
                scopes,
                granted_at: consent.granted_at.to_string(),
            });
        }
    }

    Ok(Json(responses))
}

/// 撤销对某个客户端的授权
pub async fn revoke_consent(
    State(state): State<Arc<AppState>>,
    Path(client_id): Path<String>,
    headers: HeaderMap,
) -> Result<impl IntoResponse> {
    let auth_user = require_auth_user(&state.db, &headers, &state.config.session_secret).await?;

    // 查找客户端
    let client = crate::repo::ClientRepo::find_by_client_id(&state.db, &client_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error while finding client by client_id: {}", e);
            AppError::InternalServerError {
                error_code: Some("CLIENT_FETCH_ERROR".to_string()),
            }
        })?
        .ok_or(AppError::ClientNotFound {
            client_id: Some(client_id),
        })?;

    // 删除同意记录
    crate::repo::ConsentRepo::revoke(&state.db, auth_user.user.id, client.id)
        .await
        .map_err(|e| {
            tracing::error!("Database error while revoking consent: {}", e);
            AppError::InternalServerError {
                error_code: Some("CONSENT_REVOKE_ERROR".to_string()),
            }
        })?;

    // 记录审计日志
    let ip_address = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let _ = AuditService::log_consent_revoked(
        &state.db,
        auth_user.user.id,
        client.id,
        None, // correlation_id
        ip_address,
        user_agent,
    )
    .await;

    Ok(Json(serde_json::json!({ "success": true })))
}

// ============================================================================
// 邮箱验证
// ============================================================================

#[derive(Debug, Deserialize, Validate)]
pub struct RequestEmailChangeRequest {
    #[validate(email(message = "邮箱格式不正确"))]
    pub new_email: String,
}

#[derive(Debug, Serialize)]
pub struct RequestEmailChangeResponse {
    pub success: bool,
    pub message: String,
}

/// 请求邮箱修改 - 向新邮箱发送验证链接
pub async fn request_email_change(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<RequestEmailChangeRequest>,
) -> Result<Json<RequestEmailChangeResponse>> {
    let auth_user = require_auth_user(&state.db, &headers, &state.config.session_secret).await?;

    req.validate().map_err(|e| AppError::ValidationError {
        field: "request".to_string(),
        message: e.to_string(),
    })?;

    // 验证新邮箱是否已被其他用户使用
    if let Ok(Some(existing)) = crate::repo::UserRepo::find_by_email(&state.db, &req.new_email).await {
        if existing.id != auth_user.user.id {
            return Err(AppError::BusinessError {
                code: "EMAIL_ALREADY_EXISTS".to_string(),
                message: "该邮箱已被其他用户使用".to_string(),
            });
        }
    }

    // 生成验证 token
    let plain_token = EmailVerificationService::request_email_change(
        &state.db,
        &auth_user.user,
        &req.new_email,
    )
    .await
    .map_err(|e| AppError::BusinessError {
        code: "EMAIL_CHANGE_REQUEST_FAILED".to_string(),
        message: e.to_string(),
    })?;

    // 发送验证邮件到新邮箱
    if let Some(email_svc) = &state.email_service {
        let verify_url = format!(
            "{}/profile?email_verify_token={}",
            state.config.app_base_url, plain_token
        );
        if let Err(e) = email_svc
            .send_email_change_verification(
                &req.new_email,
                &auth_user.user.username,
                &verify_url,
            )
            .await
        {
            tracing::error!("Failed to send email verification: {}", e);
            // 仍然返回成功，避免泄露邮箱是否存在的信息
        }
    } else {
        // SMTP 未配置时，降级为日志打印（开发用）
        tracing::info!(
            "Email verification token for user {}: {}*** (expires in 24 hours)",
            auth_user.user.username,
            &plain_token[..8]
        );
    }

    Ok(Json(RequestEmailChangeResponse {
        success: true,
        message: "验证链接已发送到新邮箱，请在 24 小时内点击链接进行验证".to_string(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct VerifyEmailChangeRequest {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyEmailChangeResponse {
    pub success: bool,
    pub message: String,
    pub new_email: String,
}

/// 验证邮箱修改 - 确认邮箱变更
pub async fn verify_email_change(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<VerifyEmailChangeRequest>,
) -> Result<Json<VerifyEmailChangeResponse>> {
    let auth_user = require_auth_user(&state.db, &headers, &state.config.session_secret).await?;

    // 验证 token 并获取新邮箱
    let new_email = EmailVerificationService::verify_email_change(&state.db, &req.token)
        .await
        .map_err(|e| AppError::BusinessError {
            code: "EMAIL_VERIFICATION_FAILED".to_string(),
            message: e.to_string(),
        })?;

    // 更新用户邮箱
    crate::repo::UserRepo::update_email(&state.db, auth_user.user.id, &new_email)
        .await
        .map_err(|e| {
            tracing::error!("Database error while updating user email: {}", e);
            AppError::InternalServerError {
                error_code: Some("EMAIL_UPDATE_ERROR".to_string()),
            }
        })?;

    // 记录审计日志
    let ip_address = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let _ = AuditService::log_email_changed(
        &state.db,
        auth_user.user.id,
        &new_email,
        None,
        ip_address,
        user_agent,
    )
    .await;

    Ok(Json(VerifyEmailChangeResponse {
        success: true,
        message: "邮箱验证成功，邮箱已更新".to_string(),
        new_email,
    }))
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 检查用户是否为管理员
async fn check_is_admin(pool: &sqlx::PgPool, auth_user: &AuthUser) -> Result<bool> {
    // 获取 admin 组
    let admin_group = crate::repo::GroupRepo::find_by_name(pool, "admin")
        .await
        .map_err(|e| {
            tracing::error!("Database error while finding admin group: {}", e);
            AppError::InternalServerError {
                error_code: Some("ADMIN_CHECK_ERROR".to_string()),
            }
        })?;

    if let Some(group) = admin_group {
        // 检查用户是否在 admin 组
        let user_groups = crate::repo::GroupRepo::find_user_groups(pool, auth_user.user.id)
            .await
            .map_err(|e| {
                tracing::error!("Database error while finding user groups: {}", e);
                AppError::InternalServerError {
                    error_code: Some("ADMIN_CHECK_ERROR".to_string()),
                }
            })?;

        Ok(user_groups.iter().any(|g| g.id == group.id))
    } else {
        Ok(false)
    }
}

// 为 AuditService 添加密码修改和撤销授权日志方法
impl AuditService {
    pub async fn log_password_change(
        pool: &sqlx::PgPool,
        user_id: Uuid,
        correlation_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> std::result::Result<crate::model::AuditLog, sqlx::Error> {
        let create_log = crate::model::CreateAuditLog::success(
            "password_change",
            "user_account",
            user_id.to_string(),
        )
        .with_actor(user_id)
        .with_correlation_id(correlation_id.unwrap_or_default())
        .with_ip(ip_address.unwrap_or_else(|| "unknown".to_string()))
        .with_user_agent(user_agent.unwrap_or_else(|| "unknown".to_string()))
        .with_metadata(serde_json::json!({
            "event": "password_change"
        }));

        crate::repo::AuditLogRepo::create(pool, create_log).await
    }

    pub async fn log_consent_revoked(
        pool: &sqlx::PgPool,
        user_id: Uuid,
        client_id: Uuid,
        correlation_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> std::result::Result<crate::model::AuditLog, sqlx::Error> {
        let create_log = crate::model::CreateAuditLog::success(
            "consent_revoked",
            "user_consent",
            format!("{}:{}", user_id, client_id),
        )
        .with_actor(user_id)
        .with_client(client_id)
        .with_correlation_id(correlation_id.unwrap_or_default())
        .with_ip(ip_address.unwrap_or_else(|| "unknown".to_string()))
        .with_user_agent(user_agent.unwrap_or_else(|| "unknown".to_string()))
        .with_metadata(serde_json::json!({
            "event": "consent_revoked"
        }));

        crate::repo::AuditLogRepo::create(pool, create_log).await
    }
}
