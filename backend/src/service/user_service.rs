// User Service
//
// 用户业务逻辑层

use sqlx::PgPool;
use uuid::Uuid;

use crate::crypto;
use crate::model::{CreateUser, LoginResult, UpdateUser, User};
use crate::repo::{GroupRepo, UserRepo};

pub struct UserService;

impl UserService {
    /// 注册新用户
    pub async fn register(
        pool: &PgPool,
        username: String,
        email: String,
        password: String,
    ) -> Result<User, anyhow::Error> {
        // 验证用户名和邮箱格式
        if username.is_empty() || username.len() > 64 {
            return Err(anyhow::anyhow!("Username must be 1-64 characters"));
        }
        if email.is_empty() || !email.contains('@') {
            return Err(anyhow::anyhow!("Invalid email address"));
        }
        if password.len() < 8 {
            return Err(anyhow::anyhow!("Password must be at least 8 characters"));
        }

        // 检查用户名是否已存在
        if UserRepo::find_by_username(pool, &username).await?.is_some() {
            return Err(anyhow::anyhow!("Username already exists"));
        }

        // 检查邮箱是否已存在
        if UserRepo::find_by_email(pool, &email).await?.is_some() {
            return Err(anyhow::anyhow!("Email already exists"));
        }

        // 哈希密码
        let password_hash = crypto::hash_password(&password)?;

        // 创建用户
        let user = UserRepo::create(
            pool,
            CreateUser {
                username,
                email,
                password_hash,
                display_name: None,
                given_name: None,
                family_name: None,
            },
        )
        .await?;

        Ok(user)
    }

    /// 用户登录
    pub async fn login(
        pool: &PgPool,
        username: &str,
        password: &str,
    ) -> Result<LoginResult, anyhow::Error> {
        // 查找用户
        let user = UserRepo::find_by_username(pool, username)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Invalid username or password"))?;

        // 检查账户是否可以登录
        if !user.can_login() {
            return Err(anyhow::anyhow!("Account is disabled or locked"));
        }

        // 验证密码
        let password_valid = crypto::verify_password(password, &user.password_hash)?;

        if !password_valid {
            // 增加失败次数
            let failed_attempts = UserRepo::increment_failed_login(pool, user.id).await?;

            // 如果失败次数超过 5 次，锁定账户 30 分钟
            if failed_attempts >= 5 {
                let lock_until = time::OffsetDateTime::now_utc() + time::Duration::minutes(30);
                UserRepo::lock_account(pool, user.id, lock_until).await?;
                return Err(anyhow::anyhow!(
                    "Account locked due to too many failed attempts"
                ));
            }

            return Err(anyhow::anyhow!("Invalid username or password"));
        }

        // 登录成功，重置失败次数并更新登录时间
        UserRepo::reset_failed_login(pool, user.id).await?;
        UserRepo::update_last_login(pool, user.id).await?;

        // 获取用户所属的组
        let groups = GroupRepo::find_user_groups(pool, user.id).await?;
        let group_names: Vec<String> = groups.into_iter().map(|g| g.name).collect();

        Ok(LoginResult {
            user,
            groups: group_names,
        })
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
        // 获取用户
        let user = Self::get_user(pool, user_id).await?;

        // 验证旧密码
        if !crypto::verify_password(old_password, &user.password_hash)? {
            return Err(anyhow::anyhow!("Invalid old password"));
        }

        // 验证新密码
        if new_password.len() < 8 {
            return Err(anyhow::anyhow!("Password must be at least 8 characters"));
        }

        // 哈希新密码
        let new_hash = crypto::hash_password(new_password)?;

        // 更新密码（通过更新用户的 password_hash 字段）
        // 注意：这里需要在 UserRepo 中添加 update_password 方法
        // 暂时使用直接 SQL
        sqlx::query(
            r#"
            UPDATE users
            SET password_hash = $2, updated_at = $3
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .bind(&new_hash)
        .bind(time::OffsetDateTime::now_utc())
        .execute(pool)
        .await?;

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
