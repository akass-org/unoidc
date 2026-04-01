// Me Handler
//
// 用户自助 API 接口（当前登录用户）

use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::{AppError, Result},
    middleware::auth::{require_auth_user, AuthUser},
    model::UpdateUser,
    service::{AuditService, UserService},
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
    let auth_user = require_auth_user(&state.db, &headers).await?;
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
    let auth_user = require_auth_user(&state.db, &headers).await?;

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
    let auth_user = require_auth_user(&state.db, &headers).await?;

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

#[derive(Debug, Deserialize)]
pub struct UploadAvatarRequest {
    // 在实际实现中，这里应该处理 multipart/form-data
    // 为简化，先使用 base64 编码的图片数据
    pub avatar: String,
}

#[derive(Debug, Serialize)]
pub struct UploadAvatarResponse {
    pub picture: String,
}

/// 上传用户头像
pub async fn upload_avatar(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(_req): Json<UploadAvatarRequest>,
) -> Result<Json<ProfileResponse>> {
    let auth_user = require_auth_user(&state.db, &headers).await?;

    // TODO: 实现头像上传逻辑
    // 1. 验证图片格式和大小
    // 2. 保存图片到存储（本地或云存储）
    // 3. 更新用户 picture 字段

    // 暂时返回当前用户信息
    let is_admin = check_is_admin(&state.db, &auth_user).await?;
    Ok(Json(ProfileResponse::from((auth_user.user, is_admin))))
}

// ============================================================================
// 已授权应用
// ============================================================================

#[derive(Debug, Serialize)]
pub struct AppResponse {
    pub client_id: String,
    pub client_name: String,
    pub description: Option<String>,
    pub granted_at: String,
    pub scopes: Vec<String>,
}

/// 获取当前用户已授权的应用列表
pub async fn get_apps(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<AppResponse>>> {
    let auth_user = require_auth_user(&state.db, &headers).await?;

    // 获取用户的所有同意记录
    let consents = crate::repo::ConsentRepo::find_user_consents(&state.db, auth_user.user.id)
        .await
        .map_err(|e| AppError::InternalServerError {
            error_code: Some(format!("DB_ERROR: {}", e)),
        })?;

    let mut apps = Vec::new();
    for consent in consents {
        // 获取客户端信息
        if let Ok(Some(client)) =
            crate::repo::ClientRepo::find_by_id(&state.db, consent.client_id).await
        {
            let scopes: Vec<String> = consent.scope.split_whitespace().map(|s| s.to_string()).collect();
            apps.push(AppResponse {
                client_id: client.client_id,
                client_name: client.name,
                description: client.description,
                granted_at: consent.granted_at.to_string(),
                scopes,
            });
        }
    }

    Ok(Json(apps))
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
    let auth_user = require_auth_user(&state.db, &headers).await?;

    let consents = crate::repo::ConsentRepo::find_user_consents(&state.db, auth_user.user.id)
        .await
        .map_err(|e| AppError::InternalServerError {
            error_code: Some(format!("DB_ERROR: {}", e)),
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
    let auth_user = require_auth_user(&state.db, &headers).await?;

    // 查找客户端
    let client = crate::repo::ClientRepo::find_by_client_id(&state.db, &client_id)
        .await
        .map_err(|e| AppError::InternalServerError {
            error_code: Some(format!("DB_ERROR: {}", e)),
        })?
        .ok_or(AppError::ClientNotFound {
            client_id: Some(client_id),
        })?;

    // 删除同意记录
    crate::repo::ConsentRepo::revoke(&state.db, auth_user.user.id, client.id)
        .await
        .map_err(|e| AppError::InternalServerError {
            error_code: Some(format!("DB_ERROR: {}", e)),
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
// 辅助函数
// ============================================================================

/// 检查用户是否为管理员
async fn check_is_admin(pool: &sqlx::PgPool, auth_user: &AuthUser) -> Result<bool> {
    // 获取 admin 组
    let admin_group = crate::repo::GroupRepo::find_by_name(pool, "admin")
        .await
        .map_err(|e| AppError::InternalServerError {
            error_code: Some(format!("DB_ERROR: {}", e)),
        })?;

    if let Some(group) = admin_group {
        // 检查用户是否在 admin 组
        let user_groups = crate::repo::GroupRepo::find_user_groups(pool, auth_user.user.id)
            .await
            .map_err(|e| AppError::InternalServerError {
                error_code: Some(format!("DB_ERROR: {}", e)),
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
