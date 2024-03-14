//! Casper Client implementation for Odra.
//! It uses parts of the Casper Client implementation to communicate with Casper Node's RPC API.
pub mod casper_client;
pub mod casper_node_port;
pub mod casper_types_port;
pub mod log;

pub use casper_client::CasperClientConfiguration;
pub use odra_core::casper_types::SecretKey;
