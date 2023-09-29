//! Blockchain-agnostic types used by Odra Framework.

#![no_std]

extern crate alloc;

mod address;
pub mod arithmetic;
#[cfg(not(target_arch = "wasm32"))]
pub mod contract_def;
mod error;
pub mod event;
pub mod uints;

use alloc::vec::Vec;

pub type BlockTime = u64;
pub type EventData = Vec<u8>;

pub use address::{Address, OdraAddress};
pub use casper_types;
pub use casper_types::bytesrepr::{Bytes, FromBytes, ToBytes};
use casper_types::CLValue;
pub use casper_types::{runtime_args, RuntimeArgs};
pub use casper_types::{CLType, CLTyped, PublicKey, SecretKey, U128, U256, U512};
pub use error::{AddressError, CollectionError, ExecutionError, OdraError, VmError};

pub trait UncheckedGetter {
    fn get<T: FromBytes + CLTyped>(&self, key: &str) -> T;
}

impl UncheckedGetter for RuntimeArgs {
    fn get<T: FromBytes + CLTyped>(&self, key: &str) -> T {
        self.get(key)
            .map(Clone::clone)
            .map(CLValue::into_t)
            .unwrap()
            .unwrap()
    }
}
