use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    EnvFilter,
    fmt::{self, format::FmtSpan, time::FormatTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

/// Custom time formatter for consistent timestamp format
struct CustomTimeFormat;

impl FormatTime for CustomTimeFormat {
    fn format_time(&self, w: &mut fmt::format::Writer<'_>) -> std::fmt::Result {
        let now = chrono::Local::now();
        write!(w, "{}", now.format("%Y-%m-%d %H:%M:%S%.3f"))
    }
}

/// Logging configuration options
#[derive(Debug, Clone)]
pub struct LogConfig {
    pub format: LogFormat,
    pub level: String,
    pub file_output: Option<FileOutputConfig>,
    pub show_target: bool,
    pub show_thread: bool,
    pub show_file_location: bool,
    pub show_span_events: bool,
}

#[derive(Debug, Clone)]
pub enum LogFormat {
    Json,
    Compact,
    Pretty,
}

#[derive(Debug, Clone)]
pub struct FileOutputConfig {
    pub directory: String,
    pub file_prefix: String,
    pub rotation: Rotation,
}

impl Default for LogConfig {
    fn default() -> Self {
        // Determine format based on environment
        let format = if cfg!(debug_assertions) {
            LogFormat::Pretty
        } else {
            LogFormat::Compact
        };

        Self {
            format,
            level: std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            file_output: None,
            show_target: false,
            show_thread: false,
            show_file_location: cfg!(debug_assertions),
            show_span_events: false,
        }
    }
}

/// Initialize the logging system with the given configuration
pub fn init_logging(config: LogConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Create environment filter
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.level));

    // Create the base subscriber
    let subscriber = tracing_subscriber::registry().with(env_filter);

    // Configure console layer based on format
    match config.format {
        LogFormat::Json => {
            let console_layer = fmt::layer()
                .json()
                .with_current_span(true)
                .with_span_list(true)
                .with_target(config.show_target)
                .with_thread_ids(config.show_thread)
                .with_thread_names(config.show_thread)
                .with_file(config.show_file_location)
                .with_line_number(config.show_file_location);

            if let Some(file_config) = config.file_output {
                let file_appender = RollingFileAppender::new(
                    file_config.rotation,
                    file_config.directory,
                    file_config.file_prefix,
                );
                let file_layer = fmt::layer()
                    .json()
                    .with_writer(file_appender)
                    .with_ansi(false);

                subscriber.with(console_layer).with(file_layer).try_init()?;
            } else {
                subscriber.with(console_layer).try_init()?;
            }
        }
        LogFormat::Compact => {
            let console_layer = fmt::layer()
                .compact()
                .with_timer(CustomTimeFormat)
                .with_target(config.show_target)
                .with_thread_ids(config.show_thread)
                .with_file(config.show_file_location)
                .with_line_number(config.show_file_location)
                .with_span_events(if config.show_span_events {
                    FmtSpan::ENTER | FmtSpan::EXIT
                } else {
                    FmtSpan::NONE
                });

            if let Some(file_config) = config.file_output {
                let file_appender = RollingFileAppender::new(
                    file_config.rotation,
                    file_config.directory,
                    file_config.file_prefix,
                );
                let file_layer = fmt::layer()
                    .compact()
                    .with_writer(file_appender)
                    .with_ansi(false)
                    .with_timer(CustomTimeFormat);

                subscriber.with(console_layer).with(file_layer).try_init()?;
            } else {
                subscriber.with(console_layer).try_init()?;
            }
        }
        LogFormat::Pretty => {
            let console_layer = fmt::layer()
                .pretty()
                .with_timer(CustomTimeFormat)
                .with_target(config.show_target)
                .with_thread_ids(config.show_thread)
                .with_file(config.show_file_location)
                .with_line_number(config.show_file_location)
                .with_span_events(if config.show_span_events {
                    FmtSpan::ENTER | FmtSpan::EXIT
                } else {
                    FmtSpan::NONE
                });

            if let Some(file_config) = config.file_output {
                let file_appender = RollingFileAppender::new(
                    file_config.rotation,
                    file_config.directory,
                    file_config.file_prefix,
                );
                let file_layer = fmt::layer()
                    .pretty()
                    .with_writer(file_appender)
                    .with_ansi(false)
                    .with_timer(CustomTimeFormat);

                subscriber.with(console_layer).with(file_layer).try_init()?;
            } else {
                subscriber.with(console_layer).try_init()?;
            }
        }
    }

    Ok(())
}

/// Log system information at startup
pub fn log_system_info(version: &str) {
    tracing::info!("===========================================");
    tracing::info!("Reframe Transformation Service");
    tracing::info!("Version: {}", version);
    tracing::info!(
        "Build: {}",
        if cfg!(debug_assertions) {
            "Debug"
        } else {
            "Release"
        }
    );
    tracing::info!("===========================================");

    // Log runtime environment
    tracing::debug!("Runtime Information:");
    tracing::debug!("  CPU Cores: {}", num_cpus::get());
    tracing::debug!(
        "  Physical Memory: {} MB",
        sys_info::mem_info().map(|m| m.total / 1024).unwrap_or(0)
    );
    tracing::debug!(
        "  OS: {} {}",
        sys_info::os_type().unwrap_or_else(|_| "Unknown".to_string()),
        sys_info::os_release().unwrap_or_else(|_| "".to_string())
    );
}

/// Create a span for a request with correlation ID
#[macro_export]
macro_rules! request_span {
    ($request_id:expr, $method:expr, $path:expr) => {
        tracing::info_span!(
            "request",
            request_id = %$request_id,
            method = %$method,
            path = %$path,
            latency_ms = tracing::field::Empty,
            status_code = tracing::field::Empty,
        )
    };
}

/// Log a transformation operation with context
#[macro_export]
macro_rules! log_transformation {
    ($level:expr, $direction:expr, $message_type:expr, $status:expr, $details:expr) => {
        match $level {
            tracing::Level::INFO => tracing::info!(
                direction = $direction,
                message_type = $message_type,
                status = $status,
                "{}",
                $details
            ),
            tracing::Level::WARN => tracing::warn!(
                direction = $direction,
                message_type = $message_type,
                status = $status,
                "{}",
                $details
            ),
            tracing::Level::ERROR => tracing::error!(
                direction = $direction,
                message_type = $message_type,
                status = $status,
                "{}",
                $details
            ),
            _ => tracing::debug!(
                direction = $direction,
                message_type = $message_type,
                status = $status,
                "{}",
                $details
            ),
        }
    };
}
