// 系统设置 Repository
//
// 管理全局系统配置

use sqlx::PgPool;

pub struct SettingsRepo;

impl SettingsRepo {
    /// 获取单个设置值
    pub async fn get(pool: &PgPool, key: &str) -> Result<Option<String>, sqlx::Error> {
        let row =
            sqlx::query_scalar::<_, String>("SELECT value FROM system_settings WHERE key = $1")
                .bind(key)
                .fetch_optional(pool)
                .await?;

        Ok(row)
    }

    /// 获取所有设置
    pub async fn get_all(pool: &PgPool) -> Result<Vec<(String, String)>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (String, String)>("SELECT key, value FROM system_settings")
            .fetch_all(pool)
            .await?;

        Ok(rows)
    }

    /// 更新设置值
    pub async fn set(pool: &PgPool, key: &str, value: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO system_settings (key, value, updated_at)
            VALUES ($1, $2, CURRENT_TIMESTAMP)
            ON CONFLICT (key) DO UPDATE SET
                value = EXCLUDED.value,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(key)
        .bind(value)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 批量更新设置
    pub async fn set_many(pool: &PgPool, settings: &[(String, String)]) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;

        for (key, value) in settings {
            sqlx::query(
                r#"
                INSERT INTO system_settings (key, value, updated_at)
                VALUES ($1, $2, CURRENT_TIMESTAMP)
                ON CONFLICT (key) DO UPDATE SET
                    value = EXCLUDED.value,
                    updated_at = CURRENT_TIMESTAMP
                "#,
            )
            .bind(key)
            .bind(value)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}
