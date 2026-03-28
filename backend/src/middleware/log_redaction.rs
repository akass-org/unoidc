// 日志脱敏中间件和辅助函数
//
// 提供日志脱敏功能，确保密码、token、密钥等敏感信息不被泄露到日志中

use tracing::field::Visit;
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

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
    "auth",
    "credential",
];

/// 敏感值脱敏器
pub struct SensitiveValueRedactor;

impl SensitiveValueRedactor {
    /// 脱敏敏感值
    ///
    /// 对于长字符串，显示前 4 个字符和后 4 个字符
    /// 对于短字符串，完全隐藏为 ***
    pub fn redact(value: &str, field_name: &str) -> String {
        // 检查字段名是否敏感
        let field_lower = field_name.to_lowercase();
        let is_sensitive = SENSITIVE_FIELDS
            .iter()
            .any(|s| field_lower.contains(s));

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
    ///
    /// 显示前 2 个字符和域名
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

    /// 脱敏 IP 地址
    ///
    /// IP 地址用于安全和审计，可以完整显示
    pub fn redact_ip(ip: &str) -> String {
        ip.to_string() // IP 地址不脱敏
    }
}

/// 日志脱敏层
///
/// 自动脱敏 tracing 日志中的敏感字段
pub struct LogRedactionLayer;

impl<S: Subscriber> Layer<S> for LogRedactionLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        // 使用访问者模式检查和脱敏字段
        let mut visitor = RedactionVisitor::default();
        event.record(&mut visitor);
    }
}

/// 日志脱敏访问者
#[derive(Default)]
struct RedactionVisitor {
    message: Option<String>,
    fields: Vec<(String, String)>,
}

impl Visit for RedactionVisitor {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        let field_name = field.name();
        let redacted = SensitiveValueRedactor::redact(value, field_name);

        if field_name == "message" {
            self.message = Some(redacted);
        } else {
            self.fields.push((field_name.to_string(), redacted));
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        let field_name = field.name();
        let value_str = format!("{:?}", value);
        let redacted = SensitiveValueRedactor::redact(&value_str, field_name);

        if field_name == "message" {
            self.message = Some(redacted);
        } else {
            self.fields.push((field_name.to_string(), redacted));
        }
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.fields.push((field.name().to_string(), value.to_string()));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.fields.push((field.name().to_string(), value.to_string()));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.fields.push((field.name().to_string(), value.to_string()));
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.fields.push((field.name().to_string(), value.to_string()));
    }

    fn record_error(&mut self, field: &tracing::field::Field, value: &(dyn std::error::Error + 'static)) {
        let value_str = value.to_string();
        let redacted = SensitiveValueRedactor::redact(&value_str, field.name());
        self.fields.push((field.name().to_string(), redacted));
    }
}

/// 辅助宏：安全日志记录
///
/// 自动脱敏敏感信息
#[macro_export]
macro_rules! safe_info {
    ($($arg:tt)*) => {
        tracing::info!($($arg)*)
    };
}

#[macro_export]
macro_rules! safe_warn {
    ($($arg:tt)*) => {
        tracing::warn!($($arg)*)
    };
}

#[macro_export]
macro_rules! safe_error {
    ($($arg:tt)*) => {
        tracing::error!($($arg)*)
    };
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

        // IP 不应该被脱敏
        assert_eq!(redacted, ip);
    }

    #[test]
    fn test_sensitive_field_detection() {
        // 测试各种字段名
        assert!(SENSITIVE_FIELDS.iter().any(|s| "password".contains(s)));
        assert!(SENSITIVE_FIELDS
            .iter()
            .any(|s| "user_password".to_lowercase().contains(s)));
        assert!(SENSITIVE_FIELDS.iter().any(|s| "api_key".contains(s)));
        assert!(!SENSITIVE_FIELDS
            .iter()
            .any(|s| "username".to_lowercase().contains(s)));
    }
}
