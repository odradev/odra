//! Blockchain-agnostic types used by Odra Framework.

#![no_std]

extern crate alloc;

mod address;
pub mod arithmetic;
#[cfg(not(target_arch = "wasm32"))]
pub mod contract_def;
mod error;
pub mod event;

use alloc::vec::Vec;
pub use error::{AddressError, CollectionError, ExecutionError, OdraError, VmError};

pub type EncodedKeyHash = [u8; 16];

pub type BlockTime = u64;
pub type EventData = Vec<u8>;

pub use casper_types;
pub use casper_types::PublicKey;

pub use address::{Address, OdraAddress};
