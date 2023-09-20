//! Odra Casper Livenet capabilites.

#![feature(once_cell)]

pub mod casper_client;
mod casper_node_port;
pub mod client_env;
pub mod contract_env;

use odra_types::casper_types::RuntimeArgs;

pub type EntrypointCall = fn(String, &RuntimeArgs) -> Vec<u8>;
pub type EntrypointArgs = Vec<String>;

mod log {
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
}
