pub mod migrations;

use sqlx::{AnyPool, Pool, Any};

pub async fn connect(database_url: &str) -> Result<Pool<Any>, sqlx::Error> {
    AnyPool::connect(database_url).await
}

pub async fn run_migrations(pool: &Pool<Any>) -> Result<(), sqlx::Error> {
    // TODO: 实现数据库迁移逻辑
    Ok(())
}
