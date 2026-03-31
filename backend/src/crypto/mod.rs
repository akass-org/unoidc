// 加密工具模块
//
// 提供密码哈希、随机令牌生成、JWT 签名等安全相关功能

pub mod common;
pub mod password;
pub mod random;
pub mod jwt;
pub mod key_encryption;

// 重新导出常用函数，方便使用
pub use common::{hash_token, verify_pkce_s256, sign_session, verify_session_signature};
pub use password::{hash_password, verify_password, hash_client_secret, verify_client_secret};
pub use random::{
    generate_secure_token, generate_authorization_code, generate_refresh_token,
    generate_session_id, generate_pkce_code_verifier, generate_csrf_token,
    generate_client_id, generate_client_secret, generate_user_id, generate_group_id,
};
