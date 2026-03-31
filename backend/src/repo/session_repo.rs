// Session Repository
//
// 浏览器会话数据访问层

use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::metrics;
use crate::model::{CreateSession, Session};

pub struct SessionRepo;

impl SessionRepo {
    /// 根据ID查找会话
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Session>, sqlx::Error> {
        sqlx::query_as::<_, Session>(
            r#"
            SELECT * FROM user_sessions WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// 根据 session_id 查找会话
    pub async fn find_by_session_id(pool: &PgPool, session_id: &str) -> Result<Option<Session>, sqlx::Error> {
        sqlx::query_as::<_, Session>(
            r#"
            SELECT * FROM user_sessions WHERE session_id = $1
            "#,
        )
        .bind(session_id)
        .fetch_optional(pool)
        .await
    }

    /// 创建会话
    pub async fn create(pool: &PgPool, input: CreateSession) -> Result<Session, sqlx::Error> {
        let id = Uuid::new_v4();
        let session_id = crate::crypto::generate_session_id()
            .expect("Failed to generate session ID");
        let now = OffsetDateTime::now_utc();
        let expires_at = now + time::Duration::seconds(input.duration_seconds);

        let session = sqlx::query_as::<_, Session>(
            r#"
            INSERT INTO user_sessions (
                id, session_id, user_id, expires_at, created_at, last_seen_at, ip_address, user_agent
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&session_id)
        .bind(input.user_id)
        .bind(expires_at)
        .bind(now)
        .bind(now)
        .bind(&input.ip_address)
        .bind(&input.user_agent)
        .fetch_one(pool)
        .await?;

        metrics::SESSION_ACTIVE_TOTAL.inc();
        Ok(session)
    }

    /// 更新会话最后访问时间
    pub async fn touch(pool: &PgPool, session_id: &str) -> Result<(), sqlx::Error> {
        let now = OffsetDateTime::now_utc();

        sqlx::query(
            r#"
            UPDATE user_sessions
            SET last_seen_at = $2
            WHERE session_id = $1
            "#,
        )
        .bind(session_id)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 删除会话（登出）
    pub async fn delete(pool: &PgPool, session_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM user_sessions WHERE session_id = $1
            "#,
        )
        .bind(session_id)
        .execute(pool)
        .await?;

        metrics::SESSION_ACTIVE_TOTAL.dec();
        Ok(())
    }

    /// 删除用户的所有会话
    pub async fn delete_user_sessions(pool: &PgPool, user_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM user_sessions WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// 清理过期会话
    pub async fn delete_expired(pool: &PgPool) -> Result<u64, sqlx::Error> {
        let now = OffsetDateTime::now_utc();

        let result = sqlx::query(
            r#"
            DELETE FROM user_sessions WHERE expires_at < $1
            "#,
        )
        .bind(now)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// 获取用户的所有活跃会话
    pub async fn find_user_sessions(pool: &PgPool, user_id: Uuid) -> Result<Vec<Session>, sqlx::Error> {
        let now = OffsetDateTime::now_utc();

        sqlx::query_as::<_, Session>(
            r#"
            SELECT * FROM user_sessions
            WHERE user_id = $1 AND expires_at > $2
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .bind(now)
        .fetch_all(pool)
        .await
    }
}
