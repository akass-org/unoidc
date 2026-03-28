// 数据库迁移模块
//
// 提供数据库连接和迁移执行功能

use sqlx::PgPool;
use std::fs;
use tracing::{info, error};
use anyhow::{Result, Context};

/// 连接到数据库
pub async fn connect(database_url: &str) -> Result<PgPool> {
    PgPool::connect(database_url)
        .await
        .context("Failed to connect to database")
}

/// 运行数据库迁移
///
/// 从 migrations/ 目录读取 SQL 文件并按顺序执行
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    info!("Running database migrations...");

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

    // 执行每个迁移文件
    for migration in migrations {
        let file_path = migrations_dir.join(&migration);
        let sql = fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read migration file: {}", migration))?;

        info!("Running migration: {}", migration);

        // 执行 SQL（可能包含多条语句）
        // 注意：SQLx 的 query() 函数每次只能执行一条 SQL 语句
        // 对于包含多条语句的迁移文件，我们需要分割并逐条执行
        execute_migration_sql(pool, &sql)
            .await
            .with_context(|| format!("Failed to execute migration: {}", migration))?;

        info!("Completed migration: {}", migration);
    }

    info!("All migrations completed successfully");
    Ok(())
}

/// 执行迁移 SQL
///
/// 处理包含多条语句的 SQL 文件
async fn execute_migration_sql(pool: &PgPool, sql: &str) -> Result<()> {
    // 改进的 SQL 语句分割
    // 1. 按分号分割
    // 2. 移除注释
    // 3. 跳过空语句

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
                // 输出即将执行的语句（截断到前200字符）
                let preview = if statement.len() > 200 {
                    format!("{}...", &statement[..200])
                } else {
                    statement.to_string()
                };
                info!("Executing SQL:\n{}", preview);

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
        tracing::debug!("Executing final SQL: {}", &statement[..statement.len().min(100)]);
        sqlx::query(statement)
            .execute(pool)
            .await
            .context("Failed to execute SQL statement")?;
    }

    Ok(())
}
