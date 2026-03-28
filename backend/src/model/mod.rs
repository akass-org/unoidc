// 数据模型模块
//
// 定义所有与数据库表对应的数据模型

pub mod user;
pub mod group;
pub mod client;
pub mod consent;
pub mod session;
pub mod authorization_code;
pub mod refresh_token;
pub mod jwk;
pub mod audit_log;

// 重新导出常用类型
pub use user::{User, CreateUser, UpdateUser, LoginResult};
pub use group::{Group, CreateGroup, UpdateGroup};
pub use client::{Client, CreateClient, UpdateClient};
pub use consent::{Consent, CreateConsent, ConsentQuery};
pub use session::{Session, CreateSession};
pub use authorization_code::{AuthorizationCode, CreateAuthorizationCode};
pub use refresh_token::{RefreshToken, CreateRefreshToken, RefreshTokenRotation};
pub use jwk::{Jwk, CreateJwk};
pub use audit_log::{AuditLog, CreateAuditLog, AuditLogQuery};
