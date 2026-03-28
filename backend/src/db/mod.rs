// 数据库模块
//
// 提供数据库连接和迁移功能

pub mod migrations;

pub use migrations::{connect, run_migrations};
