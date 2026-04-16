// 清理数据库工具
//
// 用于测试前清理数据库

use sqlx::PgPool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 加载 .env 文件
    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set (in .env or environment)");

    println!("Connecting to database...");
    let pool = PgPool::connect(&database_url).await?;

    println!("Dropping schema...");
    sqlx::query("DROP SCHEMA public CASCADE")
        .execute(&pool)
        .await?;

    println!("Creating schema...");
    sqlx::query("CREATE SCHEMA public").execute(&pool).await?;

    println!("Granting permissions...");
    sqlx::query("GRANT ALL ON SCHEMA public TO postgres")
        .execute(&pool)
        .await?;

    sqlx::query("GRANT ALL ON SCHEMA public TO public")
        .execute(&pool)
        .await?;

    println!("Database cleaned successfully!");

    Ok(())
}
