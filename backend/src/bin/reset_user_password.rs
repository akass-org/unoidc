use std::env;

use backend::crypto::password;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <username> <new_password>", args[0]);
        std::process::exit(1);
    }

    let username = &args[1];
    let new_password = &args[2];

    if new_password.len() < 8 || new_password.len() > 128 {
        anyhow::bail!("password length must be between 8 and 128");
    }

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await?;

    let new_hash = password::hash_password(new_password)?;

    let updated = sqlx::query(
        r#"
        UPDATE users
        SET
            password_hash = $2,
            failed_login_attempts = 0,
            locked_until = NULL,
            enabled = TRUE,
            updated_at = CURRENT_TIMESTAMP
        WHERE username = $1
        "#,
    )
    .bind(username)
    .bind(new_hash)
    .execute(&pool)
    .await?;

    if updated.rows_affected() == 0 {
        anyhow::bail!("user '{}' not found", username);
    }

    println!("Password reset successfully for user '{}'", username);
    Ok(())
}
