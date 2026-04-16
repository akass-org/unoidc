// 数据模型模块
//
// 定义所有与数据库表对应的数据模型

pub mod audit_log;
pub mod authorization_code;
pub mod client;
pub mod consent;
pub mod email_verification;
pub mod group;
pub mod jwk;
pub mod passkey_credential;
pub mod password_reset_token;
pub mod refresh_token;
pub mod session;
pub mod user;
pub mod webauthn_challenge;

// 重新导出常用类型
pub use audit_log::{AuditLog, AuditLogQuery, CreateAuditLog};
pub use authorization_code::{AuthorizationCode, CreateAuthorizationCode};
pub use client::{Client, CreateClient, UpdateClient};
pub use consent::{Consent, ConsentQuery, CreateConsent};
pub use email_verification::{CreateEmailVerificationToken, EmailVerificationToken};
pub use group::{CreateGroup, Group, UpdateGroup};
pub use jwk::{CreateJwk, Jwk};
pub use passkey_credential::{CreatePasskeyCredential, PasskeyCredential};
pub use password_reset_token::{CreatePasswordResetToken, PasswordResetToken};
pub use refresh_token::{CreateRefreshToken, RefreshToken, RefreshTokenRotation};
pub use session::{CreateSession, Session};
pub use user::{CreateUser, UpdateUser, User};
pub use webauthn_challenge::{CreateWebauthnChallenge, WebauthnChallenge};
