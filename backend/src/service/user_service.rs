// User Service
//
// 用户业务逻辑层

use sqlx::PgPool;
use uuid::Uuid;

use crate::crypto;
use crate::model::{CreateUser, UpdateUser, User};
use crate::repo::{GroupRepo, UserRepo};

pub struct UserService;

impl UserService {
    /// 注册新用户
    pub async fn register(
        pool: &PgPool,
        username: String,
        email: String,
        password: String,
        display_name: Option<String>,
    ) -> Result<User, anyhow::Error> {
        if username.is_empty() || username.len() > 64 {
            return Err(anyhow::anyhow!("Username must be 1-64 characters"));
        }
        if email.is_empty() || !email.contains('@') {
            return Err(anyhow::anyhow!("Invalid email address"));
        }
        if password.len() < 8 {
            return Err(anyhow::anyhow!("Password must be at least 8 characters"));
        }

        if UserRepo::find_by_username(pool, &username).await?.is_some() {
            return Err(anyhow::anyhow!("Username already exists"));
        }

        if UserRepo::find_by_email(pool, &email).await?.is_some() {
            return Err(anyhow::anyhow!("Email already exists"));
        }

        let password_hash = crypto::hash_password(&password)?;

        let user = UserRepo::create(
            pool,
            CreateUser {
                username,
                email,
                password_hash,
                display_name,
                given_name: None,
                family_name: None,
            },
        )
        .await?;

        // 检查是否是第一个用户，如果是则设为管理员
        let user_count = UserRepo::count(pool).await?;
        if user_count == 1 {
            // 确保 admin 组存在
            let admin_group = match GroupRepo::find_by_name(pool, "admin").await? {
                Some(g) => g,
                None => {
                    GroupRepo::create(
                        pool,
                        crate::model::CreateGroup {
                            name: "admin".to_string(),
                            description: Some("System administrators".to_string()),
                        },
                    )
                    .await?
                }
            };
            // 将用户添加到 admin 组
            GroupRepo::add_user_to_group(pool, user.id, admin_group.id).await?;
        }

        Ok(user)
    }

    /// 根据 ID 获取用户
    pub async fn get_user(pool: &PgPool, id: Uuid) -> Result<User, anyhow::Error> {
        UserRepo::find_by_id(pool, id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("User not found"))
    }

    /// 根据用户名获取用户
    pub async fn get_user_by_username(pool: &PgPool, username: &str) -> Result<User, anyhow::Error> {
        UserRepo::find_by_username(pool, username)
            .await?
            .ok_or_else(|| anyhow::anyhow!("User not found"))
    }

    /// 更新用户信息
    pub async fn update_user(
        pool: &PgPool,
        id: Uuid,
        input: UpdateUser,
    ) -> Result<User, anyhow::Error> {
        UserRepo::update(pool, id, input)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to update user: {}", e))
    }

    /// 修改密码
    pub async fn change_password(
        pool: &PgPool,
        user_id: Uuid,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), anyhow::Error> {
        let user = Self::get_user(pool, user_id).await?;

        if !crypto::verify_password(old_password, &user.password_hash)? {
            return Err(anyhow::anyhow!("Invalid old password"));
        }

        if new_password.len() < 8 {
            return Err(anyhow::anyhow!("Password must be at least 8 characters"));
        }

        let new_hash = crypto::hash_password(new_password)?;

        UserRepo::update_password(pool, user_id, &new_hash)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to update password: {}", e))?;

        Ok(())
    }

    /// 直接设置新密码（用于密码重置，不需要旧密码）
    pub async fn change_password_raw(
        pool: &PgPool,
        user_id: Uuid,
        new_password: &str,
    ) -> Result<(), anyhow::Error> {
        if new_password.len() < 8 {
            return Err(anyhow::anyhow!("Password must be at least 8 characters"));
        }

        let new_hash = crypto::hash_password(new_password)?;

        UserRepo::update_password(pool, user_id, &new_hash)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to update password: {}", e))?;

        Ok(())
    }

    /// 获取所有用户
    pub async fn list_users(pool: &PgPool, limit: i32, offset: i32) -> Result<Vec<User>, anyhow::Error> {
        UserRepo::find_all(pool, limit, offset)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list users: {}", e))
    }

    /// 删除用户
    pub async fn delete_user(pool: &PgPool, id: Uuid) -> Result<(), anyhow::Error> {
        UserRepo::delete(pool, id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete user: {}", e))
    }
}
