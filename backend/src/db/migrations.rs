use sqlx::PgPool;
use anyhow::{Result, Context};
use std::fs;
use tracing::info;

pub async fn connect(database_url: &str) -> Result<PgPool> {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(30)
        .acquire_timeout(std::time::Duration::from_secs(120))
        .idle_timeout(std::time::Duration::from_secs(300))
        .connect(database_url)
        .await
        .context("Failed to connect to database")
}

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    info!("Running database migrations...");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS _migrations (
            name VARCHAR(255) PRIMARY KEY,
            executed_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
        )"
    )
    .execute(pool)
    .await
    .context("Failed to create migrations tracking table")?;

    let current_dir = std::env::current_dir()
        .context("Failed to get current directory")?;
    let migrations_dir = current_dir.join("migrations");

    if !migrations_dir.exists() {
        anyhow::bail!("Migrations directory not found at {:?}", migrations_dir);
    }

    let mut migrations: Vec<String> = fs::read_dir(&migrations_dir)
        .context("Failed to read migrations directory")?
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.extension()?.to_str()? == "sql" {
                    path.file_name()?.to_str().map(String::from)
                } else {
                    None
                }
            })
        })
        .collect();

    migrations.sort();

    info!("Found {} migration files", migrations.len());

    let executed: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM _migrations"
    )
    .fetch_all(pool)
    .await
    .context("Failed to query executed migrations")?;

    if executed.is_empty() {
        let table_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.tables
                WHERE table_name = 'users'
            )"
        )
        .fetch_one(pool)
        .await
        .unwrap_or(false);

        if table_exists {
            info!("Detected existing database schema, registering all migrations as executed");
            for migration in &migrations {
                sqlx::query("INSERT INTO _migrations (name) VALUES ($1) ON CONFLICT (name) DO NOTHING")
                    .bind(migration)
                    .execute(pool)
                    .await
                    .ok();
            }
            info!("All existing migrations registered");
            return Ok(());
        }
    }

    for migration in migrations {
        if executed.contains(&migration) {
            info!("Skipping already executed migration: {}", migration);
            continue;
        }

        let file_path = migrations_dir.join(&migration);
        let sql = fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read migration file: {}", migration))?;

        info!("Running migration: {}", migration);

        sqlx::raw_sql(&sql)
            .execute(pool)
            .await
            .with_context(|| format!("Failed to execute migration: {}", migration))?;

        sqlx::query(
            "INSERT INTO _migrations (name) VALUES ($1)"
        )
        .bind(&migration)
        .execute(pool)
        .await
        .with_context(|| format!("Failed to record migration: {}", migration))?;

        info!("Completed migration: {}", migration);
    }

    info!("All migrations completed successfully");
    Ok(())
}
