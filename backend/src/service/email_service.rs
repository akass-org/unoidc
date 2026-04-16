// Email service
//
// 基于 lettre 的 SMTP 邮件发送服务

use anyhow::Result;

/// Email service (可选依赖，SMTP 未配置时为 None)
pub struct EmailService {
    from_address: String,
    host: String,
    port: u16,
    username: String,
    password: String,
}

impl EmailService {
    pub fn new(
        host: String,
        port: u16,
        username: String,
        password: String,
        from_address: String,
        _tls: bool,
    ) -> Self {
        Self {
            from_address,
            host,
            port,
            username,
            password,
        }
    }

    /// 检查 SMTP 是否已配置
    pub fn is_configured(&self) -> bool {
        !self.host.is_empty()
    }

    /// 发送邮箱变更验证邮件
    pub async fn send_email_change_verification(
        &self,
        to_email: &str,
        username: &str,
        verification_url: &str,
    ) -> Result<()> {
        if !self.is_configured() {
            tracing::warn!("Email service not configured, skipping email send");
            return Ok(());
        }

        let subject = "[OIDC] Verify your new email address";
        let text_body = format!(
            "Hi {},\n\nPlease verify your new email address by clicking:\n{}\n\nThis link expires in 24 hours.\n\nIf you did not request this change, please ignore this email.",
            username, verification_url
        );

        self.send(to_email, subject, &text_body).await
    }

    /// 发送密码重置邮件
    pub async fn send_password_reset(
        &self,
        to_email: &str,
        username: &str,
        reset_url: &str,
    ) -> Result<()> {
        if !self.is_configured() {
            tracing::warn!("Email service not configured, skipping email send");
            return Ok(());
        }

        let subject = "[OIDC] Reset your password";
        let text_body = format!(
            "Hi {},\n\nClick here to reset your password:\n{}\n\nThis link expires in 30 minutes.\n\nIf you did not request this reset, please ignore this email.",
            username, reset_url
        );

        self.send(to_email, subject, &text_body).await
    }

    async fn send(&self, to: &str, subject: &str, text_body: &str) -> Result<()> {
        use lettre::{Message, SmtpTransport, Transport};

        let from = self
            .from_address
            .parse::<lettre::message::Mailbox>()
            .map_err(|e| anyhow::anyhow!("Invalid from address: {}", e))?;
        let to_addr = to
            .parse::<lettre::message::Mailbox>()
            .map_err(|e| anyhow::anyhow!("Invalid to address: {}", e))?;

        let email = Message::builder()
            .from(from)
            .to(to_addr)
            .subject(subject)
            .body(text_body.to_string())
            .map_err(|e| anyhow::anyhow!("Failed to build email: {}", e))?;

        // 构建 SMTP transport（使用 builder_dangerous，KISS 原则）
        let creds = lettre::transport::smtp::authentication::Credentials::new(
            self.username.clone(),
            self.password.clone(),
        );
        let transport = SmtpTransport::builder_dangerous(&self.host)
            .port(self.port)
            .credentials(creds)
            .build();

        // lettre 的 Transport 是同步的，需要在 spawn_blocking 中调用
        tokio::task::spawn_blocking(move || transport.send(&email))
            .await
            .map_err(|e| anyhow::anyhow!("Email send task failed: {}", e))?
            .map_err(|e| anyhow::anyhow!("Failed to send email: {}", e))?;

        tracing::info!(to = %to, subject = %subject, "Email sent successfully");
        Ok(())
    }
}
