use backend::{build_app_with_state, config::Config, db, metrics, AppState};
use std::{io::IsTerminal, net::SocketAddr};
use tracing::{Event, Subscriber};
use tracing_subscriber::{
    fmt::{
        format::{FormatEvent, FormatFields, Writer},
        time::FormatTime,
        FmtContext,
    },
    layer::SubscriberExt,
    registry::LookupSpan,
    util::SubscriberInitExt,
};

/// 自定义日志格式化器：时间/target 灰色，级别彩色，错误/警告消息也带色
struct ColoredFormatter {
    ansi: bool,
}

impl ColoredFormatter {
    fn new(ansi: bool) -> Self {
        Self { ansi }
    }
}

impl<S, N> FormatEvent<S, N> for ColoredFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        let meta = event.metadata();
        let timer = tracing_subscriber::fmt::time::time();

        // 时间 - 灰色
        if self.ansi {
            write!(writer, "\x1b[90m")?;
        }
        timer.format_time(&mut writer)?;
        if self.ansi {
            write!(writer, "\x1b[0m")?;
        }
        write!(writer, " ")?;

        // 级别 - 彩色
        if self.ansi {
            let color = match *meta.level() {
                tracing::Level::ERROR => "\x1b[1;31m", // bold red
                tracing::Level::WARN => "\x1b[1;33m",  // bold yellow
                tracing::Level::INFO => "\x1b[1;32m",  // bold green
                tracing::Level::DEBUG => "\x1b[1;34m", // bold blue
                tracing::Level::TRACE => "\x1b[1;35m", // bold magenta
            };
            write!(writer, "{}{:>5}\x1b[0m", color, meta.level())?;
        } else {
            write!(writer, "{:>5}", meta.level())?;
        }

        // target - 灰色
        if self.ansi {
            write!(writer, " \x1b[90m{}\x1b[0m", meta.target())?;
        } else {
            write!(writer, " {}", meta.target())?;
        }

        write!(writer, ": ")?;

        // 消息体：ERROR 淡红，WARN 黄，其他默认
        if self.ansi {
            match *meta.level() {
                tracing::Level::ERROR => write!(writer, "\x1b[91m")?,
                tracing::Level::WARN => write!(writer, "\x1b[93m")?,
                _ => {}
            }
        }

        ctx.field_format().format_fields(writer.by_ref(), event)?;

        if self.ansi && matches!(*meta.level(), tracing::Level::ERROR | tracing::Level::WARN) {
            write!(writer, "\x1b[0m")?;
        }

        writeln!(writer)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ansi = if let Ok(value) = std::env::var("UNOIDC_LOG_COLOR") {
        let v = value.trim().to_ascii_lowercase();
        matches!(v.as_str(), "1" | "true" | "yes" | "on")
    } else {
        std::env::var_os("NO_COLOR").is_none() && std::io::stdout().is_terminal()
    };

    // 初始化日志（带颜色自定义）
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=debug,sqlx=info,tower_http=info".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(ansi)
                .event_format(ColoredFormatter::new(ansi)),
        )
        .init();

    // 加载配置
    let config = Config::from_env()?;
    tracing::info!("Configuration loaded");

    // 连接数据库
    let db = db::connect(&config.database_url).await?;
    tracing::info!("Database connected");

    // 运行迁移
    db::run_migrations(&db).await?;
    tracing::info!("Migrations completed");

    // 初始化 metrics
    metrics::init();
    tracing::info!("Metrics initialized");

    // 初始化活跃会话指标（M-05: 启动时统计实际会话数）
    match backend::repo::SessionRepo::count_active(&db).await {
        Ok(count) => {
            metrics::SESSION_ACTIVE_TOTAL.set(count as f64);
            tracing::info!("Initialized session_active_total to {}", count);
        }
        Err(e) => {
            tracing::warn!("Failed to count active sessions on startup: {}", e);
            // 继续使用默认值 0，不阻止启动
        }
    }

    // 构建应用（注入配置和数据库连接）
    let email_service = if !config.smtp.host.is_empty() {
        Some(backend::service::EmailService::new(
            config.smtp.host.clone(),
            config.smtp.port,
            config.smtp.username.clone(),
            config.smtp.password.clone(),
            config.smtp.from_address.clone(),
            config.smtp.tls,
        ))
    } else {
        tracing::info!("SMTP not configured, email features will be disabled");
        None
    };

    // 初始化 WebAuthn
    let origin_url = webauthn_rs::prelude::Url::parse(&config.webauthn_origin)
        .expect("WEBAUTHN_ORIGIN must be a valid URL");
    let webauthn = webauthn_rs::WebauthnBuilder::new(&config.webauthn_rp_id, &origin_url)
        .expect("Invalid WebAuthn configuration")
        .rp_name("unoidc")
        .build()
        .expect("Failed to build Webauthn instance");

    let state = AppState::new(config, db, email_service, webauthn);
    let app = build_app_with_state(state);

    // 启动服务器
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("Server listening on {}", listener.local_addr()?);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
