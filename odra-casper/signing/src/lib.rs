#![doc = include_str!("../README.md")]

pub mod build;
mod decode;
mod error;
mod file;
mod inspect;
mod signature;
mod transport;

pub use decode::decode_account_change;
pub use error::Error;
pub use file::{read_transaction, write_transaction};
pub use inspect::{inspect, Inspection, Signer, Status};
pub use signature::{merge, sign, verify_approval, SignatureString};
pub use transport::{FileTransport, SigningTransport};

/// The most approvals a transaction can carry (`max_associated_keys` in the Casper 2.x chainspec).
pub const MAX_APPROVALS: usize = odra_casper_account_manager::MAX_ASSOCIATED_KEYS;
