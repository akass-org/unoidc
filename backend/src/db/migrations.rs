use anyhow::{Context, Result};
use sqlx::PgPool;
use std::fs;
use tracing::info;

/// 自然排序比较函数
///
/// 比较字符串时，将数字部分按数值大小比较而非字典序
/// 例如: "0002_foo.sql" < "0010_bar.sql" （字典序会认为 "10" < "2"）
fn natural_sort_compare(a: &str, b: &str) -> std::cmp::Ordering {
    let mut a_chars = a.chars().peekable();
    let mut b_chars = b.chars().peekable();

    loop {
        match (a_chars.peek(), b_chars.peek()) {
            (None, None) => return std::cmp::Ordering::Equal,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (Some(a_ch), Some(b_ch)) => {
                let a_is_digit = a_ch.is_ascii_digit();
                let b_is_digit = b_ch.is_ascii_digit();

                match (a_is_digit, b_is_digit) {
                    // 都是数字，按数值比较
                    (true, true) => {
                        let a_num = take_number(&mut a_chars);
                        let b_num = take_number(&mut b_chars);
                        match a_num.cmp(&b_num) {
                            std::cmp::Ordering::Equal => continue,
                            other => return other,
                        }
                    }
                    // 都不是数字，按字符比较
                    (false, false) => {
                        let a_ch = a_chars.next().unwrap();
                        let b_ch = b_chars.next().unwrap();
                        match a_ch.cmp(&b_ch) {
                            std::cmp::Ordering::Equal => continue,
                            other => return other,
                        }
                    }
                    // 一个是数字一个不是，数字排在前面
                    (true, false) => return std::cmp::Ordering::Less,
                    (false, true) => return std::cmp::Ordering::Greater,
                }
            }
        }
    }
}

/// 从迭代器中连续读取数字字符并解析为 u64
fn take_number<I>(chars: &mut std::iter::Peekable<I>) -> u64
where
    I: Iterator<Item = char>,
{
    let mut num_str = String::new();
    while let Some(&ch) = chars.peek() {
        if ch.is_ascii_digit() {
            num_str.push(ch);
            chars.next();
        } else {
            break;
        }
    }
    num_str.parse().unwrap_or(0)
}

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
        )",
    )
    .execute(pool)
    .await
    .context("Failed to create migrations tracking table")?;

    let current_dir = std::env::current_dir().context("Failed to get current directory")?;
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

    // 自然排序：正确处理数字前缀，如 0001, 0002, 0010 而非字典序 0001, 0010, 0002
    migrations.sort_by(|a, b| natural_sort_compare(a, b));

    info!("Found {} migration files", migrations.len());

    let executed: Vec<String> = sqlx::query_scalar("SELECT name FROM _migrations")
        .fetch_all(pool)
        .await
        .context("Failed to query executed migrations")?;

    if executed.is_empty() {
        let table_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.tables
                WHERE table_name = 'users'
            )",
        )
        .fetch_one(pool)
        .await
        .unwrap_or(false);

        if table_exists {
            info!("Detected existing database schema, registering all migrations as executed");
            for migration in &migrations {
                sqlx::query(
                    "INSERT INTO _migrations (name) VALUES ($1) ON CONFLICT (name) DO NOTHING",
                )
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

        sqlx::query("INSERT INTO _migrations (name) VALUES ($1)")
            .bind(&migration)
            .execute(pool)
            .await
            .with_context(|| format!("Failed to record migration: {}", migration))?;

        info!("Completed migration: {}", migration);
    }

    info!("All migrations completed successfully");
    Ok(())
}
