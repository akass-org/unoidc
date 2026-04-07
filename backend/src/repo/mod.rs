// Repository 模块
//
// 数据访问层

pub mod audit_log_repo;
pub mod auth_code_repo;
pub mod client_repo;
pub mod consent_repo;
pub mod email_verification_repo;
pub mod group_repo;
pub mod jwk_repo;
pub mod password_reset_token_repo;
pub mod refresh_token_repo;
pub mod session_repo;
pub mod settings_repo;
pub mod user_repo;

// 重新导出常用类型
pub use audit_log_repo::AuditLogRepo;
pub use auth_code_repo::AuthCodeRepo;
pub use client_repo::ClientRepo;
pub use consent_repo::ConsentRepo;
pub use email_verification_repo::EmailVerificationTokenRepo;
pub use group_repo::GroupRepo;
pub use jwk_repo::JwkRepo;
pub use password_reset_token_repo::PasswordResetTokenRepo;
pub use refresh_token_repo::RefreshTokenRepo;
pub use session_repo::SessionRepo;
pub use settings_repo::SettingsRepo;
pub use user_repo::UserRepo;
