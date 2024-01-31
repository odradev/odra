//! Logging utilities.
/// Info message.
pub fn info<T: AsRef<str>>(message: T) {
    prettycli::info(message.as_ref());
}

/// Error message.
pub fn error<T: AsRef<str>>(message: T) {
    prettycli::error(message.as_ref());
}

/// Wait message.
pub fn wait<T: AsRef<str>>(message: T) {
    prettycli::wait(message.as_ref());
}
