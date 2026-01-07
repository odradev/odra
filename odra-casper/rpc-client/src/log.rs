//! Logging utilities with configurable log levels.

use std::sync::OnceLock;

/// Log levels in order of verbosity (from least to most verbose).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum LogLevel {
    None = 0,
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4
}

impl LogLevel {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "none" => LogLevel::None,
            "error" => LogLevel::Error,
            "warn" => LogLevel::Warn,
            "info" => LogLevel::Info,
            "debug" => LogLevel::Debug,
            _ => LogLevel::Info // Default to info
        }
    }
}

static LOG_LEVEL: OnceLock<LogLevel> = OnceLock::new();

/// Initialize log level from environment variable.
/// Should be called once at startup.
pub fn init_log_level() {
    let level_str = std::env::var("ODRA_LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    LOG_LEVEL.set(LogLevel::from_str(&level_str)).ok();
}

fn get_log_level() -> LogLevel {
    *LOG_LEVEL.get_or_init(|| {
        let level_str = std::env::var("ODRA_LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
        LogLevel::from_str(&level_str)
    })
}

fn should_log(level: LogLevel) -> bool {
    get_log_level() >= level
}

/// Info message.
pub fn info<T: AsRef<str>>(message: T) {
    if should_log(LogLevel::Info) {
        prettycli::info(message.as_ref());
    }
}

/// Debug message.
pub fn debug<T: AsRef<str>>(message: T) {
    if should_log(LogLevel::Debug) {
        // reuse info style for now but strictly filtered by Debug level
        println!("[DEBUG] {}", message.as_ref());
    }
}

/// Error message.
pub fn error<T: AsRef<str>>(message: T) {
    if should_log(LogLevel::Error) {
        prettycli::error(message.as_ref());
    }
}

/// Wait message (treated as info level).
pub fn wait<T: AsRef<str>>(message: T) {
    if should_log(LogLevel::Info) {
        prettycli::wait(message.as_ref());
    }
}

/// Link message (treated as info level).
pub fn link<T: AsRef<str>>(message: T) {
    if should_log(LogLevel::Info) {
        prettycli::link(message.as_ref());
    }
}
