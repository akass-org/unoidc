use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <key> <value>", args[0]);
        eprintln!("Example: {} login_layout centered", args[0]);
        std::process::exit(1);
    }

    let key = &args[1];
    let value = &args[2];

    // Load database URL from .env file
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    // Connect to database
    let pool = sqlx::PgPool::connect(&database_url).await?;

    // Update setting
    sqlx::query(
        r#"
        INSERT INTO system_settings (key, value, updated_at)
        VALUES ($1, $2, CURRENT_TIMESTAMP)
        ON CONFLICT (key) DO UPDATE SET
            value = EXCLUDED.value,
            updated_at = CURRENT_TIMESTAMP
        "#
    )
    .bind(key)
    .bind(value)
    .execute(&pool)
    .await?;

    println!("Updated setting: {} = {}", key, value);
    Ok(())
}
