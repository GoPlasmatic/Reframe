//! Configuration helper utilities
//!
//! This module provides reusable helper functions for configuration parsing
//! and environment variable handling.

use crate::logging::LogFormat;
use crate::package_manager::RuntimeConfig;

/// Calculate the number of Tokio worker threads based on configuration
///
/// Priority order:
/// 1. Environment variable TOKIO_WORKER_THREADS (highest)
/// 2. Configuration file value
/// 3. Auto-detect from CPU cores (fallback)
///
/// # Arguments
/// * `runtime_config` - Runtime configuration from reframe.config.json
///
/// # Returns
/// The number of worker threads to use
pub fn calculate_tokio_threads(runtime_config: &RuntimeConfig) -> usize {
    std::env::var("TOKIO_WORKER_THREADS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .or_else(|| {
            if runtime_config.tokio_worker_threads == "auto" {
                None
            } else {
                runtime_config.tokio_worker_threads.parse::<usize>().ok()
            }
        })
        .unwrap_or_else(num_cpus::get)
}

/// Parse log format string from configuration
///
/// Supports: json, compact, pretty
/// Falls back to compact if invalid format is provided
///
/// # Arguments
/// * `config_format` - Format string from configuration file
///
/// # Returns
/// Parsed LogFormat enum
pub fn parse_log_format(config_format: &str) -> LogFormat {
    // Check environment variable first
    let format_str = std::env::var("LOG_FORMAT")
        .ok()
        .unwrap_or_else(|| config_format.to_string());

    match format_str.as_str() {
        "json" => LogFormat::Json,
        "pretty" => LogFormat::Pretty,
        _ => LogFormat::Compact,
    }
}

/// Get thread name prefix from configuration or environment
///
/// # Arguments
/// * `runtime_config` - Runtime configuration from reframe.config.json
///
/// # Returns
/// Thread name prefix to use
pub fn get_thread_name_prefix(runtime_config: &RuntimeConfig) -> String {
    std::env::var("TOKIO_THREAD_NAME").unwrap_or_else(|_| runtime_config.thread_name_prefix.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_log_format() {
        assert!(matches!(parse_log_format("json"), LogFormat::Json));
        assert!(matches!(parse_log_format("pretty"), LogFormat::Pretty));
        assert!(matches!(parse_log_format("compact"), LogFormat::Compact));
        assert!(matches!(parse_log_format("invalid"), LogFormat::Compact));
    }
}
