// Admin Handler
//
// 管理后台 API 接口

use axum::{
    extract::{ConnectInfo, Path, State},
    http::HeaderMap,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use time::format_description::well_known::Rfc3339;
use uuid::Uuid;
use validator::Validate;

use crate::{
    crypto,
    error::{AppError, Result},
    middleware::auth::{require_auth_user, AuthUser},
    model::{CreateClient, CreateGroup, UpdateClient, UpdateGroup, UpdateUser},
    repo::{AuditLogRepo, ClientRepo, GroupRepo, RefreshTokenRepo, SettingsRepo, UserRepo},
    service::{AuditService, AuthService, ClientService, GroupService, KeyService, UserService},
    AppState,
};

async fn ensure_admin_group(pool: &sqlx::PgPool) -> Result<crate::model::Group> {
    match GroupRepo::find_by_name(pool, "admin").await {
        Ok(Some(group)) => Ok(group),
        Ok(None) => GroupRepo::create(
            pool,
            CreateGroup {
                name: "admin".to_string(),
                description: Some("System administrators".to_string()),
            },
        )
        .await
        .map_err(|e| {
            tracing::error!("Database error while creating admin group: {}", e);
            AppError::InternalServerError {
                error_code: Some("ADMIN_GROUP_ERROR".to_string()),
            }
        }),
        Err(e) => {
            tracing::error!("Database error while finding admin group: {}", e);
            Err(AppError::InternalServerError {
                error_code: Some("ADMIN_GROUP_ERROR".to_string()),
            })
        }
    }
}

/// 检查指定用户是否为管理员
async fn is_user_admin(pool: &sqlx::PgPool, user_id: Uuid) -> Result<bool> {
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
        let user_groups = crate::repo::GroupRepo::find_user_groups(pool, user_id)
            .await
            .map_err(|e| {
                tracing::error!("Database error while finding user groups: {}", e);
                AppError::InternalServerError {
                    error_code: Some("ADMIN_CHECK_ERROR".to_string()),
                }
            })?;

        return Ok(user_groups.iter().any(|g| g.id == group.id));
    }

    Ok(false)
}

/// 检查当前请求用户是否为管理员
async fn require_admin(
    pool: &sqlx::PgPool,
    headers: &HeaderMap,
    session_secret: &str,
) -> Result<AuthUser> {
    let auth_user = require_auth_user(pool, headers, session_secret).await?;

    if is_user_admin(pool, auth_user.user.id).await? {
        return Ok(auth_user);
    }

    Err(AppError::Forbidden {
        reason: Some("Admin access required".to_string()),
    })
}

// ============================================================================
// 用户管理
// ============================================================================

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
    pub groups: Vec<String>,
    pub is_admin: bool,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 100, message = "用户名长度必须在 3-100 之间"))]
    pub username: String,
    #[validate(email(message = "邮箱格式不正确"))]
    pub email: String,
    #[validate(length(min = 1, message = "显示名称不能为空"))]
    pub display_name: String,
    pub password: String,
    pub is_admin: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub is_admin: Option<bool>,
    pub is_active: Option<bool>,
}

/// 异步将 User 转换为 UserResponse，检查管理员权限
async fn user_to_response(pool: &sqlx::PgPool, user: crate::model::User) -> Result<UserResponse> {
    let is_admin = is_user_admin(pool, user.id).await.unwrap_or(false);
    let groups = GroupRepo::find_user_groups(pool, user.id)
        .await
        .map_err(|e| {
            tracing::error!("Database error while finding user groups: {}", e);
            AppError::InternalServerError {
                error_code: Some("USER_GROUPS_FETCH_ERROR".to_string()),
            }
        })?
        .into_iter()
        .map(|group| group.name)
        .collect();

    Ok(UserResponse {
        id: user.id.to_string(),
        username: user.username,
        email: user.email,
        display_name: user.display_name.unwrap_or_default(),
        given_name: user.given_name,
        family_name: user.family_name,
        picture: user.picture,
        groups,
        is_admin,
        is_active: user.enabled,
        created_at: user
            .created_at
            .format(&Rfc3339)
            .unwrap_or_else(|_| user.created_at.to_string()),
    })
}

/// 获取所有用户
pub async fn get_users(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<UserResponse>>> {
    let _auth_user = require_admin(&state.db, &headers, &state.config.session_secret).await?;

    let users = UserService::list_users(&state.db, 1000, 0)
        .await
        .map_err(|e| {
            tracing::error!("Database error while listing users: {}", e);
            AppError::InternalServerError {
                error_code: Some("USERS_FETCH_ERROR".to_string()),
            }
        })?;

    let mut responses = Vec::new();
    for user in users {
        responses.push(user_to_response(&state.db, user).await?);
    }

    Ok(Json(responses))
}

/// 创建用户（管理员）
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>> {
    let _auth_user = require_admin(&state.db, &headers, &state.config.session_secret).await?;

    req.validate().map_err(|e| AppError::ValidationError {
        field: "request".to_string(),
        message: e.to_string(),
    })?;

    let password = if req.password.is_empty() {
        crypto::generate_secure_token(16).map_err(|e| AppError::InternalServerError {
            error_code: Some(format!("TOKEN_GEN_ERROR: {}", e)),
        })?
    } else {
        req.password
    };

    let mut user = UserService::register(
        &state.db,
        req.username,
        req.email,
        password,
        Some(req.display_name),
    )
    .await
    .map_err(|e| AppError::BusinessError {
        code: "USER_CREATE_FAILED".to_string(),
        message: e.to_string(),
    })?;

    if req.is_admin {
        let admin_group = ensure_admin_group(&state.db).await?;
        GroupRepo::add_user_to_group(&state.db, user.id, admin_group.id)
            .await
            .map_err(|e| AppError::InternalServerError {
                error_code: Some(format!("DB_ERROR: {}", e)),
            })?;

        user = UserRepo::find_by_id(&state.db, user.id)
            .await
            .map_err(|e| AppError::InternalServerError {
                error_code: Some(format!("DB_ERROR: {}", e)),
            })?
            .ok_or_else(|| AppError::BusinessError {
                code: "USER_NOT_FOUND".to_string(),
                message: "User not found after create".to_string(),
            })?;
    }

    Ok(Json(user_to_response(&state.db, user).await?))
}

/// 更新用户
pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>> {
    let _auth_user = require_admin(&state.db, &headers, &state.config.session_secret).await?;

    if let Some(ref email) = req.email {
        if !email.contains('@') {
            return Err(AppError::ValidationError {
                field: "email".to_string(),
                message: "邮箱格式不正确".to_string(),
            });
        }

        if let Some(existing) = UserRepo::find_by_email(&state.db, email)
            .await
            .map_err(|e| {
                tracing::error!("Database error while checking email: {}", e);
                AppError::InternalServerError {
                    error_code: Some("EMAIL_CHECK_ERROR".to_string()),
                }
            })?
        {
            if existing.id != id {
                return Err(AppError::BusinessError {
                    code: "EMAIL_ALREADY_EXISTS".to_string(),
                    message: "Email already exists".to_string(),
                });
            }
        }
    }

    let update = UpdateUser {
        display_name: req.display_name,
        given_name: None,
        family_name: None,
        picture: None,
        email_verified: None,
        enabled: req.is_active,
    };

    UserService::update_user(&state.db, id, update)
        .await
        .map_err(|e| AppError::BusinessError {
            code: "USER_UPDATE_FAILED".to_string(),
            message: e.to_string(),
        })?;

    if let Some(email) = req.email {
        UserRepo::update_email(&state.db, id, &email)
            .await
            .map_err(|e| AppError::BusinessError {
                code: "USER_UPDATE_FAILED".to_string(),
                message: format!("Failed to update email: {}", e),
            })?;
    }

    if let Some(is_admin) = req.is_admin {
        let admin_group = ensure_admin_group(&state.db).await?;
        if is_admin {
            GroupRepo::add_user_to_group(&state.db, id, admin_group.id)
                .await
                .map_err(|e| AppError::InternalServerError {
                    error_code: Some(format!("DB_ERROR: {}", e)),
                })?;
        } else {
            GroupRepo::remove_user_from_group(&state.db, id, admin_group.id)
                .await
                .map_err(|e| AppError::InternalServerError {
                    error_code: Some(format!("DB_ERROR: {}", e)),
                })?;
        }
    }

    let user = UserRepo::find_by_id(&state.db, id)
        .await
        .map_err(|e| AppError::InternalServerError {
            error_code: Some(format!("DB_ERROR: {}", e)),
        })?
        .ok_or_else(|| AppError::BusinessError {
            code: "USER_NOT_FOUND".to_string(),
            message: "User not found after update".to_string(),
        })?;

    Ok(Json(user_to_response(&state.db, user).await?))
}

#[derive(Debug, Serialize)]
pub struct ResetPasswordResponse {
    pub message: String,
}

/// 重置用户密码
pub async fn reset_user_password(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<ResetPasswordResponse>> {
    let auth_user = require_admin(&state.db, &headers, &state.config.session_secret).await?;

    let new_password =
        crypto::generate_secure_token(16).map_err(|e| AppError::InternalServerError {
            error_code: Some(format!("TOKEN_GEN_ERROR: {}", e)),
        })?;

    let password_hash =
        crypto::hash_password(&new_password).map_err(|e| AppError::InternalServerError {
            error_code: Some(format!("HASH_ERROR: {}", e)),
        })?;

    crate::repo::UserRepo::update_password(&state.db, id, &password_hash)
        .await
        .map_err(|e| AppError::BusinessError {
            code: "PASSWORD_RESET_FAILED".to_string(),
            message: e.to_string(),
        })?;

    // 管理员重置密码后，强制目标用户全局下线
    AuthService::logout_all_sessions(&state.db, id)
        .await
        .map_err(|_| AppError::InternalServerError {
            error_code: Some("SESSION_REVOKE_FAILED".to_string()),
        })?;

    RefreshTokenRepo::revoke_all_for_user(&state.db, id)
        .await
        .map_err(|_| AppError::InternalServerError {
            error_code: Some("TOKEN_REVOKE_FAILED".to_string()),
        })?;

    // 安全提取客户端 IP（带受信代理检查）
    let remote_ip = addr.ip().to_string();
    let is_trusted = state.config.trusted_proxy_ips.contains(&remote_ip);
    let ip_address = if is_trusted {
        headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(',').next().map(|ip| ip.trim().to_string()))
            .filter(|s| !s.is_empty())
            .or_else(|| {
                headers
                    .get("x-real-ip")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
            })
            .unwrap_or(remote_ip)
    } else {
        remote_ip
    };
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // 记录审计日志（包含操作者信息）
    let _ = AuditService::log_password_reset(
        &state.db,
        id,
        Some(auth_user.user.id.to_string()),
        Some(ip_address),
        user_agent,
    )
    .await;

    Ok(Json(ResetPasswordResponse {
        message: "Password has been reset successfully".to_string(),
    }))
}

// ============================================================================
// 用户组管理
// ============================================================================

#[derive(Debug, Serialize)]
pub struct GroupResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub member_count: i64,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateGroupRequest {
    #[validate(length(min = 1, max = 64, message = "组名长度必须在 1-64 之间"))]
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGroupRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// 获取所有用户组
pub async fn get_groups(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<GroupResponse>>> {
    let _auth_user = require_admin(&state.db, &headers, &state.config.session_secret).await?;

    let groups = GroupService::list_groups(&state.db).await.map_err(|e| {
        tracing::error!("Database error while listing groups: {}", e);
        AppError::InternalServerError {
            error_code: Some("GROUPS_FETCH_ERROR".to_string()),
        }
    })?;

    let mut responses = Vec::new();
    for group in groups {
        let member_count = crate::repo::GroupRepo::find_group_user_ids(&state.db, group.id)
            .await
            .map_err(|e| {
                tracing::error!("Database error while fetching group members: {}", e);
                AppError::InternalServerError {
                    error_code: Some("GROUP_MEMBERS_ERROR".to_string()),
                }
            })?
            .len() as i64;

        responses.push(GroupResponse {
            id: group.id.to_string(),
            name: group.name,
            description: group.description.unwrap_or_default(),
            member_count,
            created_at: group
                .created_at
                .format(&Rfc3339)
                .unwrap_or_else(|_| group.created_at.to_string()),
        });
    }

    Ok(Json(responses))
}

/// 创建用户组
pub async fn create_group(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreateGroupRequest>,
) -> Result<Json<GroupResponse>> {
    let _auth_user = require_admin(&state.db, &headers, &state.config.session_secret).await?;

    req.validate().map_err(|e| AppError::ValidationError {
        field: "request".to_string(),
        message: e.to_string(),
    })?;

    let input = CreateGroup {
        name: req.name,
        description: req.description,
    };

    let group = GroupService::create_group(&state.db, input)
        .await
        .map_err(|e| AppError::BusinessError {
            code: "GROUP_CREATE_FAILED".to_string(),
            message: e.to_string(),
        })?;

    Ok(Json(GroupResponse {
        id: group.id.to_string(),
        name: group.name,
        description: group.description.unwrap_or_default(),
        member_count: 0,
        created_at: group.created_at.to_string(),
    }))
}

/// 更新用户组
pub async fn update_group(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<UpdateGroupRequest>,
) -> Result<Json<GroupResponse>> {
    let _auth_user = require_admin(&state.db, &headers, &state.config.session_secret).await?;

    let update = UpdateGroup {
        name: req.name,
        description: req
            .description
            .map(|d| if d.is_empty() { None } else { Some(d) }),
    };

    let group = GroupService::update_group(&state.db, id, update)
        .await
        .map_err(|e| AppError::BusinessError {
            code: "GROUP_UPDATE_FAILED".to_string(),
            message: e.to_string(),
        })?;

    let member_count = crate::repo::GroupRepo::find_group_user_ids(&state.db, group.id)
        .await
        .map_err(|e| {
            tracing::error!("Database error while fetching group members: {}", e);
            AppError::InternalServerError {
                error_code: Some("GROUP_MEMBERS_ERROR".to_string()),
            }
        })?
        .len() as i64;

    Ok(Json(GroupResponse {
        id: group.id.to_string(),
        name: group.name,
        description: group.description.unwrap_or_default(),
        member_count,
        created_at: group.created_at.to_string(),
    }))
}

/// 删除用户组
pub async fn delete_group(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<impl IntoResponse> {
    let _auth_user = require_admin(&state.db, &headers, &state.config.session_secret).await?;

    // 保护 admin 组不被删除
    let group = GroupRepo::find_by_id(&state.db, id)
        .await
        .map_err(|e| AppError::InternalServerError {
            error_code: Some(format!("DB_ERROR: {}", e)),
        })?
        .ok_or_else(|| AppError::BusinessError {
            code: "GROUP_NOT_FOUND".to_string(),
            message: "Group not found".to_string(),
        })?;

    if group.name == "admin" {
        return Err(AppError::Forbidden {
            reason: Some("Cannot delete the admin group".to_string()),
        });
    }

    GroupService::delete_group(&state.db, id)
        .await
        .map_err(|e| AppError::BusinessError {
            code: "GROUP_DELETE_FAILED".to_string(),
            message: e.to_string(),
        })?;

    Ok(Json(serde_json::json!({ "success": true })))
}

// ============================================================================
// 应用/客户端管理
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ClientResponse {
    pub id: String,
    pub client_id: String,
    pub name: String,
    pub description: Option<String>,
    pub redirect_uris: Vec<String>,
    pub allowed_group_ids: Vec<String>,
    pub allowed_groups: Vec<String>,
    pub is_active: bool,
    pub enable_silent_authorize: bool,
    pub created_at: String,
    pub last_used: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateClientResponse {
    pub client: ClientResponse,
    pub client_secret: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateClientRequest {
    pub name: String,
    pub description: Option<String>,
    pub redirect_uris: Vec<String>,
    pub allowed_group_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateClientRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub redirect_uris: Option<Vec<String>>,
    pub is_active: Option<bool>,
    pub allowed_group_ids: Option<Vec<Uuid>>,
    pub enable_silent_authorize: Option<bool>,
}

impl From<crate::model::Client> for ClientResponse {
    fn from(client: crate::model::Client) -> Self {
        let redirect_uris = serde_json::from_value(client.redirect_uris).unwrap_or_default();
        Self {
            id: client.id.to_string(),
            client_id: client.client_id,
            name: client.name,
            description: client.description,
            redirect_uris,
            allowed_group_ids: Vec::new(),
            allowed_groups: Vec::new(),
            is_active: client.enabled,
            enable_silent_authorize: client.enable_silent_authorize,
            created_at: client.created_at.to_string(),
            last_used: None, // 由 client_to_response 填充
        }
    }
}

async fn client_to_response(
    pool: &sqlx::PgPool,
    client: crate::model::Client,
) -> Result<ClientResponse> {
    let mut response = ClientResponse::from(client.clone());
    let group_ids = ClientRepo::find_client_groups(pool, client.id)
        .await
        .map_err(|e| {
            tracing::error!("Database error while finding client groups: {}", e);
            AppError::InternalServerError {
                error_code: Some("CLIENT_GROUPS_FETCH_ERROR".to_string()),
            }
        })?;

    if !group_ids.is_empty() {
        let mut group_names = Vec::new();
        for group_id in &group_ids {
            if let Some(group) = GroupRepo::find_by_id(pool, *group_id).await.map_err(|e| {
                tracing::error!("Database error while finding client group details: {}", e);
                AppError::InternalServerError {
                    error_code: Some("CLIENT_GROUPS_FETCH_ERROR".to_string()),
                }
            })? {
                group_names.push(group.name);
            }
        }
        response.allowed_group_ids = group_ids
            .iter()
            .map(|group_id| group_id.to_string())
            .collect();
        response.allowed_groups = group_names;
    }

    // 填充客户端最近使用时间
    let last_used = crate::repo::RefreshTokenRepo::find_client_last_used(pool, client.id)
        .await
        .map_err(|e| {
            tracing::error!("Database error while fetching client last_used: {}", e);
            AppError::InternalServerError {
                error_code: Some("CLIENT_LAST_USED_FETCH_ERROR".to_string()),
            }
        })?;
    response.last_used = last_used.map(|dt| dt.format(&Rfc3339).unwrap_or_else(|_| dt.to_string()));

    Ok(response)
}

/// 获取所有客户端
pub async fn get_clients(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<ClientResponse>>> {
    let _auth_user = require_admin(&state.db, &headers, &state.config.session_secret).await?;

    let clients = ClientService::list_clients(&state.db).await.map_err(|e| {
        tracing::error!("Database error while listing clients: {}", e);
        AppError::InternalServerError {
            error_code: Some("CLIENTS_FETCH_ERROR".to_string()),
        }
    })?;

    let mut responses = Vec::with_capacity(clients.len());
    for client in clients {
        responses.push(client_to_response(&state.db, client).await?);
    }

    Ok(Json(responses))
}

/// 创建客户端
pub async fn create_client(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreateClientRequest>,
) -> Result<Json<CreateClientResponse>> {
    let _auth_user = require_admin(&state.db, &headers, &state.config.session_secret).await?;

    let input = CreateClient {
        client_id: String::new(), // 将由服务生成
        client_secret_hash: None,
        is_public: false, // 默认创建机密客户端
        name: req.name,
        description: req.description,
        app_url: None,
        redirect_uris: req.redirect_uris,
        post_logout_redirect_uris: None,
        grant_types: vec!["authorization_code".to_string()],
        response_types: vec!["code".to_string()],
        token_endpoint_auth_method: "client_secret_basic".to_string(),
    };

    let (client, secret) = ClientService::create_client(&state.db, input)
        .await
        .map_err(|e| AppError::BusinessError {
            code: "CLIENT_CREATE_FAILED".to_string(),
            message: e.to_string(),
        })?;

    if let Some(group_ids) = req.allowed_group_ids.as_ref() {
        ClientService::set_client_groups(&state.db, client.id, group_ids)
            .await
            .map_err(|e| AppError::BusinessError {
                code: "CLIENT_GROUPS_UPDATE_FAILED".to_string(),
                message: e.to_string(),
            })?;
    }

    Ok(Json(CreateClientResponse {
        client: client_to_response(&state.db, client).await?,
        client_secret: secret.unwrap_or_default(),
    }))
}

/// 更新客户端
pub async fn update_client(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<UpdateClientRequest>,
) -> Result<Json<ClientResponse>> {
    let _auth_user = require_admin(&state.db, &headers, &state.config.session_secret).await?;

    let update = UpdateClient {
        name: req.name,
        description: req.description,
        app_url: None,
        redirect_uris: req.redirect_uris,
        post_logout_redirect_uris: None,
        enabled: req.is_active,
        enable_silent_authorize: req.enable_silent_authorize,
    };

    let client = ClientService::update_client(&state.db, id, update)
        .await
        .map_err(|e| AppError::BusinessError {
            code: "CLIENT_UPDATE_FAILED".to_string(),
            message: e.to_string(),
        })?;

    if let Some(group_ids) = req.allowed_group_ids.as_ref() {
        ClientService::set_client_groups(&state.db, client.id, group_ids)
            .await
            .map_err(|e| AppError::BusinessError {
                code: "CLIENT_GROUPS_UPDATE_FAILED".to_string(),
                message: e.to_string(),
            })?;
    }

    Ok(Json(client_to_response(&state.db, client).await?))
}

/// 删除客户端
pub async fn delete_client(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<impl IntoResponse> {
    let _auth_user = require_admin(&state.db, &headers, &state.config.session_secret).await?;

    ClientService::delete_client(&state.db, id)
        .await
        .map_err(|e| AppError::BusinessError {
            code: "CLIENT_DELETE_FAILED".to_string(),
            message: e.to_string(),
        })?;

    Ok(Json(serde_json::json!({ "success": true })))
}

#[derive(Debug, Serialize)]
pub struct ResetSecretResponse {
    pub client: ClientResponse,
    pub client_secret: String,
}

/// 重置客户端密钥
pub async fn reset_client_secret(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<ResetSecretResponse>> {
    let _auth_user = require_admin(&state.db, &headers, &state.config.session_secret).await?;

    let secret = ClientService::regenerate_secret(&state.db, id)
        .await
        .map_err(|e| AppError::BusinessError {
            code: "SECRET_RESET_FAILED".to_string(),
            message: e.to_string(),
        })?;

    let client = ClientService::get_client(&state.db, id)
        .await
        .map_err(|e| AppError::BusinessError {
            code: "CLIENT_NOT_FOUND".to_string(),
            message: e.to_string(),
        })?;

    Ok(Json(ResetSecretResponse {
        client: client_to_response(&state.db, client).await?,
        client_secret: secret,
    }))
}

// ============================================================================
// 审计日志
// ============================================================================

#[derive(Debug, sqlx::FromRow)]
struct AuditLogRow {
    id: uuid::Uuid,
    actor_user_id: Option<uuid::Uuid>,
    username: Option<String>,
    attempted_username: Option<String>,
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
    pub attempted_username: Option<String>,
    pub client_id: Option<String>,
    pub client_name: Option<String>,
    pub ip_address: String,
    pub user_agent: String,
    pub outcome: String,
    pub reason: Option<String>,
    pub created_at: String,
}

/// 获取审计日志
pub async fn get_audit_logs(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<AuditLogResponse>>> {
    let _auth_user = require_admin(&state.db, &headers, &state.config.session_secret).await?;

    // 直接查询 JOIN 用户和客户端信息
    let logs = sqlx::query_as::<_, AuditLogRow>(
        r#"
        SELECT 
            al.id,
            al.actor_user_id,
            u.username,
            CASE
                WHEN al.action = 'login' AND al.outcome = 'failure'
                THEN COALESCE(al.metadata->>'username', NULLIF(al.target_id, ''))
                ELSE NULL
            END as attempted_username,
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
        ORDER BY al.created_at DESC
        LIMIT 500
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error while querying audit logs: {}", e);
        AppError::InternalServerError {
            error_code: Some("AUDIT_LOGS_FETCH_ERROR".to_string()),
        }
    })?;

    let responses: Vec<AuditLogResponse> = logs
        .into_iter()
        .map(|row| {
            // 将 action 转换为 event_type
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
                _ => row.action.as_str(),
            }
            .to_string();

            AuditLogResponse {
                id: row.id.to_string(),
                event_type,
                user_id: row.actor_user_id.map(|id| id.to_string()),
                username: row.username,
                attempted_username: row.attempted_username,
                client_id: row.client_id.map(|id| id.to_string()),
                client_name: row.client_name,
                ip_address: row.ip_address.unwrap_or_else(|| "unknown".to_string()),
                user_agent: row.user_agent.unwrap_or_else(|| "unknown".to_string()),
                outcome: row.outcome,
                reason: row.reason_code,
                created_at: row
                    .created_at
                    .format(&Rfc3339)
                    .unwrap_or_else(|_| row.created_at.to_string()),
            }
        })
        .collect();

    Ok(Json(responses))
}

// ============================================================================
// 系统设置
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct SettingsResponse {
    pub brand_name: String,
    pub logo_url: String,
    pub login_background_url: String,
    pub login_layout: String,
    pub session_timeout: i32,
    pub max_login_attempts: i32,
    pub enable_password_login: bool,
    pub enable_passkey_signup: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettingsRequest {
    pub brand_name: Option<String>,
    pub logo_url: Option<String>,
    pub login_background_url: Option<String>,
    pub login_layout: Option<String>,
    pub session_timeout: Option<i32>,
    pub max_login_attempts: Option<i32>,
    pub enable_password_login: Option<bool>,
    pub enable_passkey_signup: Option<bool>,
}

/// 获取系统设置
pub async fn get_settings(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<SettingsResponse>> {
    let _auth_user = require_admin(&state.db, &headers, &state.config.session_secret).await?;

    // 从数据库读取设置
    let settings =
        SettingsRepo::get_all(&state.db)
            .await
            .map_err(|e| AppError::InternalServerError {
                error_code: Some(format!("DB_ERROR: {}", e)),
            })?;

    // 转换为 map 方便查找
    let settings_map: std::collections::HashMap<String, String> = settings.into_iter().collect();

    let get_value = |key: &str, default: &str| -> String {
        settings_map
            .get(key)
            .cloned()
            .unwrap_or_else(|| default.to_string())
    };

    Ok(Json(SettingsResponse {
        brand_name: get_value("brand_name", "UNOIDC"),
        logo_url: get_value("logo_url", ""),
        login_background_url: get_value("login_background_url", ""),
        login_layout: get_value("login_layout", "split-left"),
        session_timeout: get_value("session_timeout", "24").parse().unwrap_or(24),
        max_login_attempts: get_value("max_login_attempts", "5").parse().unwrap_or(5),
        enable_password_login: get_value("enable_password_login", "true").parse().unwrap_or(true),
        enable_passkey_signup: get_value("enable_passkey_signup", "true").parse().unwrap_or(true),
    }))
}

/// 更新系统设置
pub async fn update_settings(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<UpdateSettingsRequest>,
) -> Result<Json<SettingsResponse>> {
    let auth_user = require_admin(&state.db, &headers, &state.config.session_secret).await?;

    // 构建要更新的设置列表
    let mut updates: Vec<(String, String)> = Vec::new();

    if let Some(brand_name) = req.brand_name {
        updates.push(("brand_name".to_string(), brand_name));
    }
    if let Some(logo_url) = req.logo_url {
        updates.push(("logo_url".to_string(), logo_url));
    }
    if let Some(login_background_url) = req.login_background_url {
        updates.push(("login_background_url".to_string(), login_background_url));
    }
    if let Some(login_layout) = req.login_layout {
        updates.push(("login_layout".to_string(), login_layout));
    }
    if let Some(session_timeout) = req.session_timeout {
        updates.push(("session_timeout".to_string(), session_timeout.to_string()));
    }
    if let Some(max_login_attempts) = req.max_login_attempts {
        updates.push((
            "max_login_attempts".to_string(),
            max_login_attempts.to_string(),
        ));
    }
    if let Some(enable_password_login) = req.enable_password_login {
        updates.push(("enable_password_login".to_string(), enable_password_login.to_string()));
    }
    if let Some(enable_passkey_signup) = req.enable_passkey_signup {
        updates.push(("enable_passkey_signup".to_string(), enable_passkey_signup.to_string()));
    }

    // 安全网：禁用密码登录前，当前管理员必须至少有一个 passkey
    if req.enable_password_login == Some(false) {
        let admin_passkeys = crate::repo::PasskeyRepo::list_by_user_id(&state.db, auth_user.user.id).await?;
        if admin_passkeys.is_empty() {
            return Err(AppError::BusinessError {
                code: "CANNOT_DISABLE_PASSWORD_LOGIN".to_string(),
                message: "您需要先绑定 passkey 才能禁用密码登录".to_string(),
            });
        }
    }

    // 批量更新到数据库
    if !updates.is_empty() {
        SettingsRepo::set_many(&state.db, &updates)
            .await
            .map_err(|e| AppError::InternalServerError {
                error_code: Some(format!("DB_ERROR: {}", e)),
            })?;
    }

    // 返回更新后的设置
    get_settings(State(state), headers).await
}

#[derive(Debug, Serialize)]
pub struct RotateKeyResponse {
    pub success: bool,
    pub message: String,
}

/// 轮换签名密钥
pub async fn rotate_key(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<RotateKeyResponse>> {
    let _auth_user = require_admin(&state.db, &headers, &state.config.session_secret).await?;

    let _new_key = KeyService::rotate_key(&state.db, &state.config.private_key_encryption_key)
        .await
        .map_err(|e| AppError::InternalServerError {
            error_code: Some(format!("KEY_ROTATION_ERROR: {}", e)),
        })?;

    Ok(Json(RotateKeyResponse {
        success: true,
        message: "密钥轮换成功".to_string(),
    }))
}

// 为 AuditService 添加密码重置日志方法
impl AuditService {
    pub async fn log_password_reset(
        pool: &sqlx::PgPool,
        user_id: Uuid,
        correlation_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> std::result::Result<crate::model::AuditLog, sqlx::Error> {
        let create_log = crate::model::CreateAuditLog::success(
            "password_reset",
            "user_account",
            user_id.to_string(),
        )
        .with_actor(user_id)
        .with_correlation_id(correlation_id.unwrap_or_default())
        .with_ip(ip_address.unwrap_or_else(|| "unknown".to_string()))
        .with_user_agent(user_agent.unwrap_or_else(|| "unknown".to_string()))
        .with_metadata(serde_json::json!({
            "event": "password_reset"
        }));

        AuditLogRepo::create(pool, create_log).await
    }
}
