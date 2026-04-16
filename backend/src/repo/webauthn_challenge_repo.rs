// WebAuthn Challenge Repository
//
// WebAuthn 挑战临时数据访问层

use sqlx::PgPool;

use crate::model::{CreateWebauthnChallenge, WebauthnChallenge};

pub struct WebauthnChallengeRepo;

impl WebauthnChallengeRepo {
    /// 插入挑战
    pub async fn create(
        pool: &PgPool,
        input: CreateWebauthnChallenge,
    ) -> Result<WebauthnChallenge, sqlx::Error> {
        sqlx::query_as::<_, WebauthnChallenge>(
            r#"
            INSERT INTO webauthn_challenges (
                challenge_hash, user_id, purpose, state_data, expires_at
            )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(&input.challenge_hash)
        .bind(input.user_id)
        .bind(&input.purpose)
        .bind(&input.state_data)
        .bind(input.expires_at)
        .fetch_one(pool)
        .await
    }

    /// 根据哈希查找挑战
    pub async fn find_by_hash(
        pool: &PgPool,
        hash: &[u8],
    ) -> Result<Option<WebauthnChallenge>, sqlx::Error> {
        sqlx::query_as::<_, WebauthnChallenge>(
            r#"
            SELECT * FROM webauthn_challenges WHERE challenge_hash = $1
            "#,
        )
        .bind(hash)
        .fetch_optional(pool)
        .await
    }

    /// 根据哈希删除挑战
    pub async fn delete_by_hash(pool: &PgPool, hash: &[u8]) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM webauthn_challenges WHERE challenge_hash = $1
            "#,
        )
        .bind(hash)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn delete_by_hash_in_tx(
        conn: &mut sqlx::PgConnection,
        hash: &[u8],
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM webauthn_challenges WHERE challenge_hash = $1
            "#,
        )
        .bind(hash)
        .execute(conn)
        .await?;

        Ok(result.rows_affected())
    }

    /// 删除所有已过期的挑战
    pub async fn delete_expired(pool: &PgPool) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM webauthn_challenges WHERE expires_at < NOW()
            "#,
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}
