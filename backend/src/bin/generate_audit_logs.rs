use sqlx::postgres::PgPool;
use uuid::Uuid;
use std::error::Error;
use time::OffsetDateTime;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/unoidc".to_string());
    
    let pool = PgPool::connect(&database_url).await?;
    
    println!("正在生成测试审计日志...");
    
    // 获取第一个用户
    let user = sqlx::query!("SELECT id FROM users LIMIT 1")
        .fetch_optional(&pool)
        .await?;
    
    // 获取第一个客户端
    let client = sqlx::query!("SELECT id FROM clients LIMIT 1")
        .fetch_optional(&pool)
        .await?;
    
    if user.is_none() {
        eprintln!("没有找到用户，请先创建一个用户");
        return Ok(());
    }
    
    let user_id = user.map(|u| u.id);
    let client_id = client.map(|c| c.id);
    
    // 插入登录成功事件
    for i in 0..3 {
        let _ = sqlx::query!(
            r#"
            INSERT INTO audit_logs (
                id, actor_user_id, client_id, correlation_id, action,
                target_type, target_id, outcome, reason_code, metadata,
                ip_address, user_agent, created_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13
            )
            "#,
            Uuid::new_v4(),
            user_id,
            client_id,
            format!("corr-{}", Uuid::new_v4()),
            "login",
            "user_session",
            format!("session-{}", i),
            "success",
            None::<String>,
            serde_json::json!({"event": "login_success"}),
            format!("192.168.1.{}", 100 + i),
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64)",
            OffsetDateTime::now_utc()
        )
        .execute(&pool)
        .await;
    }
    
    // 插入登出事件
    let _ = sqlx::query!(
        r#"
        INSERT INTO audit_logs (
            id, actor_user_id, client_id, correlation_id, action,
            target_type, target_id, outcome, reason_code, metadata,
            ip_address, user_agent, created_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13
        )
        "#,
        Uuid::new_v4(),
        user_id,
        None::<Uuid>,
        format!("corr-{}", Uuid::new_v4()),
        "logout",
        "user_session",
        "session-logout",
        "success",
        None::<String>,
        serde_json::json!({"event": "logout"}),
        "192.168.1.200",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)",
        OffsetDateTime::now_utc()
    )
    .execute(&pool)
    .await;
    
    // 插入令牌发放事件
    if client_id.is_some() {
        let _ = sqlx::query!(
            r#"
            INSERT INTO audit_logs (
                id, actor_user_id, client_id, correlation_id, action,
                target_type, target_id, outcome, reason_code, metadata,
                ip_address, user_agent, created_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13
            )
            "#,
            Uuid::new_v4(),
            user_id,
            client_id,
            format!("corr-{}", Uuid::new_v4()),
            "token_issued",
            "id_token",
            Uuid::new_v4().to_string(),
            "success",
            None::<String>,
            serde_json::json!({"event": "token_issued", "token_type": "id_token"}),
            "192.168.1.100",
            "Mozilla/5.0 (X11; Linux x86_64)",
            OffsetDateTime::now_utc()
        )
        .execute(&pool)
        .await;
    }
    
    // 插入登录失败事件
    let _ = sqlx::query!(
        r#"
        INSERT INTO audit_logs (
            id, actor_user_id, client_id, correlation_id, action,
            target_type, target_id, outcome, reason_code, metadata,
            ip_address, user_agent, created_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13
        )
        "#,
        Uuid::new_v4(),
        None::<Uuid>,
        None::<Uuid>,
        format!("corr-{}", Uuid::new_v4()),
        "login",
        "user_session",
        "invalid_user",
        "failure",
        Some("invalid_credentials"),
        serde_json::json!({"event": "login_failure", "username": "wronguser"}),
        "192.168.1.50",
        "Mozilla/5.0 (Windows NT 10.0)",
        OffsetDateTime::now_utc()
    )
    .execute(&pool)
    .await;
    
    let count = sqlx::query!("SELECT COUNT(*) as count FROM audit_logs")
        .fetch_one(&pool)
        .await?;
    
    println!("✓ 测试审计日志已生成!");
    println!("总审计日志数: {}", count.count.unwrap_or(0));
    
    Ok(())
}
