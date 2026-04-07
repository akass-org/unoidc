// 日志脱敏辅助函数
//
// 提供日志脱敏功能，确保密码、token、密钥等敏感信息不被泄露到日志中。

/// 敏感字段名称列表
const SENSITIVE_FIELDS: &[&str] = &[
    "password",
    "passwd",
    "pwd",
    "secret",
    "token",
    "api_key",
    "apikey",
    "access_token",
    "refresh_token",
    "client_secret",
    "private_key",
    "authorization",
    "credential",
    "session_id",
];

/// 敏感值脱敏器
pub struct SensitiveValueRedactor;

impl SensitiveValueRedactor {
    /// 脱敏敏感值
    ///
    /// 对于长字符串，显示前 4 个字符和后 4 个字符。
    /// 对于短字符串，完全隐藏为 ***。
    pub fn redact(value: &str, field_name: &str) -> String {
        let field_lower = field_name.to_lowercase();
        let is_sensitive = SENSITIVE_FIELDS.iter().any(|s| field_lower.contains(s));

        if is_sensitive {
            if value.len() > 8 {
                format!("{}...{}", &value[..4], &value[value.len() - 4..])
            } else {
                "***".to_string()
            }
        } else {
            value.to_string()
        }
    }

    /// 脱敏邮箱地址
    pub fn redact_email(email: &str) -> String {
        if let Some(at_pos) = email.find('@') {
            let (local, domain) = email.split_at(at_pos);
            if local.len() > 2 {
                format!("{}***{}", &local[..2], domain)
            } else {
                format!("***{}", domain)
            }
        } else {
            "***".to_string()
        }
    }

    /// 脱敏 IP 地址（不脱敏，用于安全审计）
    pub fn redact_ip(ip: &str) -> String {
        ip.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_password() {
        let password = "my_secret_password_123";
        let redacted = SensitiveValueRedactor::redact(password, "password");

        assert!(!redacted.contains("secret"));
        assert!(!redacted.contains("password"));
        assert!(redacted.contains("..."));
    }

    #[test]
    fn test_redact_token() {
        let token = "abc123def456ghi789jkl";
        let redacted = SensitiveValueRedactor::redact(token, "access_token");

        assert!(!redacted.contains("def456"));
        assert!(redacted.contains("..."));
    }

    #[test]
    fn test_redact_short_secret() {
        let secret = "short";
        let redacted = SensitiveValueRedactor::redact(secret, "secret");

        assert_eq!(redacted, "***");
    }

    #[test]
    fn test_redact_non_sensitive() {
        let value = "user123";
        let redacted = SensitiveValueRedactor::redact(value, "username");

        assert_eq!(redacted, "user123");
    }

    #[test]
    fn test_redact_email() {
        let email = "user@example.com";
        let redacted = SensitiveValueRedactor::redact_email(email);

        assert!(!redacted.contains("user@"));
        assert!(redacted.contains("***@example.com"));
    }

    #[test]
    fn test_redact_short_email() {
        let email = "a@b.com";
        let redacted = SensitiveValueRedactor::redact_email(email);

        assert!(redacted.contains("***@b.com"));
    }

    #[test]
    fn test_redact_ip() {
        let ip = "192.168.1.100";
        let redacted = SensitiveValueRedactor::redact_ip(ip);

        assert_eq!(redacted, ip);
    }

    #[test]
    fn test_sensitive_field_detection() {
        assert!(SENSITIVE_FIELDS.iter().any(|s| "password".contains(s)));
        assert!(SENSITIVE_FIELDS
            .iter()
            .any(|s| "user_password".to_lowercase().contains(s)));
        assert!(SENSITIVE_FIELDS.iter().any(|s| "api_key".contains(s)));
        assert!(!SENSITIVE_FIELDS
            .iter()
            .any(|s| "username".to_lowercase().contains(s)));
    }

    #[test]
    fn test_auth_field_is_sensitive() {
        // "auth" 不再包含在敏感列表中，避免误脱敏 auth_time 等字段
        let value = "2024-01-01T00:00:00Z";
        let redacted = SensitiveValueRedactor::redact(value, "auth_time");
        assert_eq!(redacted, value);
    }
}
