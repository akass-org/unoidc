// User Repository
//
// 用户数据访问层

use sqlx::PgPool;
use uuid::Uuid;

use crate::model::{CreateUser, UpdateUser, User};

pub struct UserRepo;

impl UserRepo {
    /// 根据ID查找用户
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// 根据用户名查找用户
    pub async fn find_by_username(pool: &PgPool, username: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(pool)
        .await
    }

    /// 根据邮箱查找用户
    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(pool)
        .await
    }

    /// 获取所有用户（分页）
    pub async fn find_all(pool: &PgPool, limit: i32, offset: i32) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    }

    /// 创建用户
    pub async fn create(pool: &PgPool, input: CreateUser) -> Result<User, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = time::OffsetDateTime::now_utc();

        sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (
                id, username, email, password_hash, display_name, given_name, family_name,
                email_verified, enabled, failed_login_attempts, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&input.username)
        .bind(&input.email)
        .bind(&input.password_hash)
        .bind(&input.display_name)
        .bind(&input.given_name)
        .bind(&input.family_name)
        .bind(false) // email_verified
        .bind(true) // enabled
        .bind(0) // failed_login_attempts
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await
    }

    /// 更新用户
    pub async fn update(pool: &PgPool, id: Uuid, input: UpdateUser) -> Result<User, sqlx::Error> {
        let now = time::OffsetDateTime::now_utc();

        sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET
                display_name = COALESCE($2, display_name),
                given_name = COALESCE($3, given_name),
                family_name = COALESCE($4, family_name),
                picture = COALESCE($5, picture),
                email_verified = COALESCE($6, email_verified),
                enabled = COALESCE($7, enabled),
                updated_at = $8
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(input.display_name)
        .bind(input.given_name)
        .bind(input.family_name)
        .bind(input.picture)
        .bind(input.email_verified)
        .bind(input.enabled)
        .bind(now)
        .fetch_one(pool)
        .await
    }

    /// 删除用户
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM users WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 更新最后登录时间
    pub async fn update_last_login(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        let now = time::OffsetDateTime::now_utc();

        sqlx::query(
            r#"
            UPDATE users
            SET last_login_at = $2, updated_at = $2
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 增加登录失败次数
    pub async fn increment_failed_login(pool: &PgPool, id: Uuid) -> Result<i32, sqlx::Error> {
        let now = time::OffsetDateTime::now_utc();

        let result: (i32,) = sqlx::query_as(
            r#"
            UPDATE users
            SET
                failed_login_attempts = failed_login_attempts + 1,
                updated_at = $2
            WHERE id = $1
            RETURNING failed_login_attempts
            "#,
        )
        .bind(id)
        .bind(now)
        .fetch_one(pool)
        .await?;

        Ok(result.0)
    }

    /// 重置登录失败次数
    pub async fn reset_failed_login(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        let now = time::OffsetDateTime::now_utc();

        sqlx::query(
            r#"
            UPDATE users
            SET
                failed_login_attempts = 0,
                locked_until = NULL,
                updated_at = $2
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 锁定账户
    pub async fn lock_account(pool: &PgPool, id: Uuid, lock_until: time::OffsetDateTime) -> Result<(), sqlx::Error> {
        let now = time::OffsetDateTime::now_utc();

        sqlx::query(
            r#"
            UPDATE users
            SET locked_until = $2, updated_at = $3
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(lock_until)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 统计用户总数
    pub async fn count(pool: &PgPool) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM users
            "#,
        )
        .fetch_one(pool)
        .await?;

        Ok(result.0)
    }
}
