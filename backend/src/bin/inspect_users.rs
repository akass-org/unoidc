use sqlx::Row;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = sqlx::PgPool::connect(&database_url).await?;

    println!("== users ==");
    let users = sqlx::query(
        r#"
        SELECT id, username, email, enabled, failed_login_attempts, locked_until
        FROM users
        ORDER BY created_at ASC
        "#,
    )
    .fetch_all(&pool)
    .await?;

    for row in users {
        let id: uuid::Uuid = row.try_get("id")?;
        let username: String = row.try_get("username")?;
        let email: String = row.try_get("email")?;
        let enabled: bool = row.try_get("enabled")?;
        let failed: i32 = row.try_get("failed_login_attempts")?;
        let locked_until: Option<time::OffsetDateTime> = row.try_get("locked_until")?;
        println!(
            "{} | {} | {} | enabled={} failed={} locked_until={:?}",
            id, username, email, enabled, failed, locked_until
        );
    }

    println!("\n== user_groups ==");
    let rows = sqlx::query(
        r#"
        SELECT g.name as group_name, u.username
        FROM groups g
        JOIN user_groups ug ON ug.group_id = g.id
        JOIN users u ON u.id = ug.user_id
        ORDER BY g.name, u.username
        "#,
    )
    .fetch_all(&pool)
    .await?;

    for row in rows {
        let group_name: String = row.try_get("group_name")?;
        let username: String = row.try_get("username")?;
        println!("{} -> {}", group_name, username);
    }

    Ok(())
}
