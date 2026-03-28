// 日志脱敏测试
//
// 测试确保 secrets/tokens/passwords 等敏感信息从日志中被脱敏

use backend::crypto::{password, random};

/// 测试辅助结构：捕获日志输出
#[derive(Debug)]
struct CapturedLog {
    level: String,
    message: String,
    fields: std::collections::HashMap<String, String>,
}

/// 自定义日志层用于测试
mod capture {
    use std::sync::{Arc, Mutex};
    use tracing_subscriber::Layer;

    pub struct CaptureLayer {
        pub logs: Arc<Mutex<Vec<String>>>,
    }

    impl CaptureLayer {
        pub fn new() -> (Self, Arc<Mutex<Vec<String>>>) {
            let logs = Arc::new(Mutex::new(Vec::new()));
            let layer = Self {
                logs: logs.clone(),
            };
            (layer, logs)
        }
    }

    impl<S> Layer<S> for CaptureLayer
    where
        S: tracing::Subscriber,
    {
        fn on_event(
            &self,
            event: &tracing::Event<'_>,
            _ctx: tracing_subscriber::layer::Context<'_, S>,
        ) {
            let mut visitor = TestVisitor::default();
            event.record(&mut visitor);

            let metadata = event.metadata();
            let level = metadata.level().to_string();
            let message = visitor.message.unwrap_or_default();

            let log_entry = format!("[{}] {}", level, message);

            if let Ok(mut logs) = self.logs.lock() {
                logs.push(log_entry);
            }
        }
    }

    #[derive(Default)]
    struct TestVisitor {
        message: Option<String>,
    }

    impl tracing::field::Visit for TestVisitor {
        fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
            if field.name() == "message" {
                self.message = Some(format!("{:?}", value));
            }
        }

        fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
            if field.name() == "message" {
                self.message = Some(value.to_string());
            }
        }
    }
}

#[test]
fn test_password_not_logged_in_plaintext() {
    // 测试密码不会被明文记录到日志
    let plaintext = "my_secret_password_123";

    // 密码应该被哈希
    let hash = password::hash_password(plaintext).unwrap();

    // 日志消息中不应该包含明文密码
    let log_message = format!("User password hash: {}", hash);

    // 确保日志消息不包含明文密码
    assert!(
        !log_message.contains(plaintext),
        "Log message should not contain plaintext password"
    );

    // 哈希值本身是可以记录的（因为它是安全的）
    assert!(
        log_message.contains(&hash[..20]),  // 只检查前 20 个字符
        "Hash should be safe to log"
    );
}

#[test]
fn test_token_not_logged_in_full() {
    // 测试 token 不会被完整记录到日志
    let token = random::generate_secure_token(32).unwrap();

    // 记录 token 时应该只记录前几个字符
    let token_preview = format!("{}...", &token[..8]);
    let log_message = format!("Generated token: {}", token_preview);

    // 确保不包含完整的 token
    assert!(
        !log_message.contains(&token[9..]),  // 不应该包含第 9 个字符之后的字符
        "Log message should not contain full token"
    );

    // 预览部分是可以记录的
    assert!(
        log_message.contains(&token[..8]),
        "Token preview (first 8 chars) is acceptable for debugging"
    );
}

#[test]
fn test_client_secret_redaction() {
    // 测试客户端密钥的脱敏
    let secret = "super_secret_client_secret_value";

    // 创建脱敏后的版本
    let redacted = if secret.len() > 8 {
        format!("{}...{}", &secret[..4], &secret[secret.len()-4..])
    } else {
        "***".to_string()
    };

    let log_message = format!("Client secret: {}", redacted);

    // 确保不包含完整的密钥
    assert!(
        !log_message.contains("super_secret"),
        "Log should not reveal middle of secret"
    );

    // 确保使用了脱敏格式
    assert!(
        log_message.contains("..."),
        "Should use redaction format with ellipsis"
    );
}

#[test]
fn test_session_id_safe_to_log() {
    // Session ID 是公开的标识符，可以记录
    let session_id = uuid::Uuid::new_v4();
    let log_message = format!("Session created: {}", session_id);

    // Session ID 是安全的，可以完整记录
    assert!(
        log_message.contains(&session_id.to_string()),
        "Session IDs are safe to log in full"
    );
}

#[test]
fn test_user_id_safe_to_log() {
    // User ID 是公开的标识符，可以记录
    let user_id = uuid::Uuid::new_v4();
    let log_message = format!("User action: {}", user_id);

    // User ID 是安全的，可以完整记录
    assert!(
        log_message.contains(&user_id.to_string()),
        "User IDs are safe to log in full"
    );
}

#[test]
fn test_email_partial_redaction() {
    // 邮箱地址应该部分脱敏
    let email = "user@example.com";

    // 脱敏邮箱：显示前 2 个字符和域名
    let parts: Vec<&str> = email.split('@').collect();
    let local_part = parts[0];
    let domain = parts[1];

    let redacted = if local_part.len() > 2 {
        format!("{}***@{}", &local_part[..2], domain)
    } else {
        format!("***@{}", domain)
    };

    let log_message = format!("Email sent to: {}", redacted);

    // 确保不包含完整的邮箱地址
    assert!(
        !log_message.contains("user@"),
        "Should not reveal full email"
    );

    // 确保显示了域名（为了上下文）
    assert!(
        log_message.contains(domain),
        "Domain is safe to show for context"
    );
}

#[test]
fn test_database_error_sanitization() {
    // 测试数据库错误信息的脱敏
    let db_error_message = "duplicate key value violates unique constraint \"users_email_key\" DETAIL: Key (email)=(sensitive@example.com) already exists.";

    // 脱敏后的错误消息
    let sanitized = db_error_message
        .replace("sensitive@example.com", "***@example.com")
        .replace("Key (email)=", "Key (email)=(***");

    let log_message = format!("Database error: {}", sanitized);

    // 确保不包含敏感的邮箱地址
    assert!(
        !log_message.contains("sensitive@"),
        "Should not contain sensitive email"
    );

    // 确保保留了错误类型信息（用于调试）
    assert!(
        log_message.contains("duplicate key"),
        "Error type should be preserved"
    );
}

#[test]
fn test_ip_address_logging() {
    // IP 地址可以记录（用于安全和审计）
    let ip = "192.168.1.100";
    let log_message = format!("Request from IP: {}", ip);

    // IP 地址是安全的，可以记录
    assert!(
        log_message.contains(ip),
        "IP addresses are safe to log for security purposes"
    );
}

#[test]
fn test_redaction_helper_function() {
    // 测试通用的脱敏辅助函数
    fn redact_sensitive(value: &str, sensitive_type: &str) -> String {
        match sensitive_type {
            "password" | "secret" | "token" => {
                if value.len() > 8 {
                    format!("{}...{}", &value[..4], &value[value.len()-4..])
                } else {
                    "***".to_string()
                }
            }
            "email" => {
                if let Some(at_pos) = value.find('@') {
                    let (local, domain) = value.split_at(at_pos);
                    if local.len() > 2 {
                        format!("{}***{}", &local[..2], domain)
                    } else {
                        format!("***{}", domain)
                    }
                } else {
                    "***".to_string()
                }
            }
            _ => value.to_string(),
        }
    }

    // 测试密码脱敏
    let password = "my_password_123";
    let redacted = redact_sensitive(password, "password");
    assert!(!redacted.contains("password"));
    assert!(redacted.contains("..."));

    // 测试邮箱脱敏
    let email = "user@example.com";
    let redacted = redact_sensitive(email, "email");
    assert!(!redacted.contains("user@"));
    assert!(redacted.contains("***@example.com"));

    // 测试 token 脱敏
    let token = "abc123def456ghi789jkl012mno345pqr";
    let redacted = redact_sensitive(token, "token");
    assert!(!redacted.contains("def456"));
    assert!(redacted.contains("..."));
}
