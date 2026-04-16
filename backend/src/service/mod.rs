// Service 模块
//
// 业务逻辑层

pub mod audit_service;
pub mod auth_service;
pub mod cleanup_service;
pub mod client_service;
pub mod consent_service;
pub mod email_service;
pub mod email_verification_service;
pub mod group_service;
pub mod key_service;
pub mod logout_service;
pub mod oidc_service;
pub mod passkey_service;
pub mod rate_limit_service;
pub mod token_service;
pub mod user_service;

// 重新导出常用类型
pub use audit_service::AuditService;
pub use auth_service::AuthService;
pub use cleanup_service::CleanupService;
pub use client_service::ClientService;
pub use consent_service::ConsentService;
pub use email_service::EmailService;
pub use email_verification_service::EmailVerificationService;
pub use group_service::GroupService;
pub use key_service::KeyService;
pub use logout_service::LogoutService;
pub use oidc_service::OidcService;
pub use passkey_service::PasskeyService;
pub use token_service::TokenService;
pub use user_service::UserService;
