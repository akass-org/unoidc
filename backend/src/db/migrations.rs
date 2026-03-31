// 数据库迁移模块
//
// 提供数据库连接和幂等迁移执行功能

use sqlx::PgPool;
use anyhow::{Result, Context};
use std::fs;
use tracing::{info, error};

/// 连接到数据库
pub async fn connect(database_url: &str) -> Result<PgPool> {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(30)
        .acquire_timeout(std::time::Duration::from_secs(120))
        .idle_timeout(std::time::Duration::from_secs(300))
        .connect(database_url)
        .await
        .context("Failed to connect to database")
}

/// 运行数据库迁移
///
/// 使用 _migrations 表跟踪已执行的迁移，确保幂等性
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    info!("Running database migrations...");

    // 创建迁移跟踪表
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS _migrations (
            name VARCHAR(255) PRIMARY KEY,
            executed_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
        )"
    )
    .execute(pool)
    .await
    .context("Failed to create migrations tracking table")?;

    // 获取 migrations 目录
    let current_dir = std::env::current_dir()
        .context("Failed to get current directory")?;
    let migrations_dir = current_dir.join("migrations");

    if !migrations_dir.exists() {
        anyhow::bail!("Migrations directory not found at {:?}", migrations_dir);
    }

    // 读取所有 .sql 文件并排序
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

    // 获取已执行的迁移
    let executed: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM _migrations"
    )
    .fetch_all(pool)
    .await
    .context("Failed to query executed migrations")?;

    // 如果 _migrations 表是空的，但数据库已有表，说明是旧版本手动执行的迁移
    // 自动检测并注册，确保向后兼容
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

    // 执行未执行的迁移
    for migration in migrations {
        if executed.contains(&migration) {
            info!("Skipping already executed migration: {}", migration);
            continue;
        }

        let file_path = migrations_dir.join(&migration);
        let sql = fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read migration file: {}", migration))?;

        info!("Running migration: {}", migration);

        execute_migration_sql(pool, &sql)
            .await
            .with_context(|| format!("Failed to execute migration: {}", migration))?;

        // 记录已执行的迁移
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

/// 执行迁移 SQL
///
/// 处理包含多条语句的 SQL 文件
async fn execute_migration_sql(pool: &PgPool, sql: &str) -> Result<()> {
    let mut current_statement = String::new();

    for line in sql.lines() {
        let trimmed = line.trim();

        // 跳过纯注释行
        if trimmed.starts_with("--") {
            continue;
        }

        // 添加到当前语句
        if !current_statement.is_empty() {
            current_statement.push('\n');
        }
        current_statement.push_str(trimmed);

        // 如果行以分号结尾，执行语句
        if trimmed.ends_with(';') {
            let statement = current_statement.trim();
            if !statement.is_empty() && statement != ";" {
                if let Err(e) = sqlx::query(statement)
                    .execute(pool)
                    .await
                {
                    error!("Failed to execute SQL statement: {}", e);
                    error!("Statement was: {}", statement);
                    return Err(e).context("Failed to execute SQL statement");
                }
            }
            current_statement.clear();
        }
    }

    // 执行最后一个语句（如果没有以分号结尾）
    let statement = current_statement.trim();
    if !statement.is_empty() {
        sqlx::query(statement)
            .execute(pool)
            .await
            .context("Failed to execute SQL statement")?;
    }

    Ok(())
}
