// Audit Log Repository
//
// 审计日志数据访问层

use sqlx::PgPool;
use uuid::Uuid;

use crate::model::{AuditLog, AuditLogQuery, CreateAuditLog};

pub struct AuditLogRepo;

impl AuditLogRepo {
    /// 根据ID查找审计日志
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<AuditLog>, sqlx::Error> {
        sqlx::query_as::<_, AuditLog>(
            r#"
            SELECT * FROM audit_logs WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// 创建审计日志
    pub async fn create(pool: &PgPool, input: CreateAuditLog) -> Result<AuditLog, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = time::OffsetDateTime::now_utc();

        sqlx::query_as::<_, AuditLog>(
            r#"
            INSERT INTO audit_logs (
                id, actor_user_id, client_id, correlation_id, action,
                target_type, target_id, outcome, reason_code, metadata,
                ip_address, user_agent, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(input.actor_user_id)
        .bind(input.client_id)
        .bind(&input.correlation_id)
        .bind(&input.action)
        .bind(&input.target_type)
        .bind(&input.target_id)
        .bind(&input.outcome)
        .bind(&input.reason_code)
        .bind(&input.metadata)
        .bind(&input.ip_address)
        .bind(&input.user_agent)
        .bind(now)
        .fetch_one(pool)
        .await
    }

    /// 查询审计日志
    pub async fn query(pool: &PgPool, query: AuditLogQuery) -> Result<Vec<AuditLog>, sqlx::Error> {
        let mut conditions = Vec::new();
        let mut param_count = 1;

        // 构建查询条件
        if let Some(_actor_user_id) = query.actor_user_id {
            conditions.push(format!("actor_user_id = ${}", param_count));
            param_count += 1;
        }
        if let Some(_client_id) = query.client_id {
            conditions.push(format!("client_id = ${}", param_count));
            param_count += 1;
        }
        if let Some(_action) = &query.action {
            conditions.push(format!("action = ${}", param_count));
            param_count += 1;
        }
        if let Some(_outcome) = &query.outcome {
            conditions.push(format!("outcome = ${}", param_count));
            param_count += 1;
        }
        if let Some(_from_time) = query.from_time {
            conditions.push(format!("created_at >= ${}", param_count));
            param_count += 1;
        }
        if let Some(_to_time) = query.to_time {
            conditions.push(format!("created_at <= ${}", param_count));
            param_count += 1;
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        const MAX_LIMIT: i32 = 1000;
        let limit = query.limit.unwrap_or(100).min(MAX_LIMIT);
        let offset = query.offset.unwrap_or(0);

        let sql = format!(
            "SELECT * FROM audit_logs {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            where_clause,
            param_count,
            param_count + 1
        );

        let mut query_builder = sqlx::query_as::<_, AuditLog>(&sql);

        // 绑定参数
        if let Some(actor_user_id) = query.actor_user_id {
            query_builder = query_builder.bind(actor_user_id);
        }
        if let Some(client_id) = query.client_id {
            query_builder = query_builder.bind(client_id);
        }
        if let Some(ref action) = query.action {
            query_builder = query_builder.bind(action);
        }
        if let Some(ref outcome) = query.outcome {
            query_builder = query_builder.bind(outcome);
        }
        if let Some(from_time) = query.from_time {
            query_builder = query_builder.bind(from_time);
        }
        if let Some(to_time) = query.to_time {
            query_builder = query_builder.bind(to_time);
        }

        query_builder = query_builder.bind(limit).bind(offset);

        query_builder.fetch_all(pool).await
    }

    /// 获取用户的审计日志
    pub async fn find_user_logs(
        pool: &PgPool,
        user_id: Uuid,
        limit: i32,
    ) -> Result<Vec<AuditLog>, sqlx::Error> {
        sqlx::query_as::<_, AuditLog>(
            r#"
            SELECT * FROM audit_logs
            WHERE actor_user_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(pool)
        .await
    }

    /// 获取客户端的审计日志
    pub async fn find_client_logs(
        pool: &PgPool,
        client_id: Uuid,
        limit: i32,
    ) -> Result<Vec<AuditLog>, sqlx::Error> {
        sqlx::query_as::<_, AuditLog>(
            r#"
            SELECT * FROM audit_logs
            WHERE client_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(client_id)
        .bind(limit)
        .fetch_all(pool)
        .await
    }

    /// 清理过期的审计日志（可选）
    pub async fn delete_before(
        pool: &PgPool,
        before: time::OffsetDateTime,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM audit_logs WHERE created_at < $1
            "#,
        )
        .bind(before)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}
