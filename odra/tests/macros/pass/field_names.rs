//! Fields may be named like the locals of the code Odra generates for a module, a type or an
//! event.
#![deny(warnings)]

use odra::casper_types::{bytesrepr::Bytes, U256};
use odra::prelude::*;

#[odra::odra_type]
pub struct NamedFields {
    pub result: u32,
    pub bytes: String,
    pub env: Option<u8>
}

#[odra::odra_type]
pub enum NamedVariants {
    Named { result: u32, bytes: Bytes },
    Tuple(u32, String),
    Unit
}

#[odra::event]
pub struct FieldsSet {
    pub bytes: u32,
    pub result: U256,
    pub env: Address
}

#[odra::module(events = [FieldsSet])]
pub struct FieldNames {
    env: Var<u32>,
    result: Var<NamedFields>,
    bytes: Mapping<u32, NamedVariants>
}

#[odra::module]
impl FieldNames {
    pub fn init(&mut self, env: u32, result: NamedFields) {
        self.env.set(env);
        self.result.set(result);
    }

    pub fn set_env(&mut self, value: u32) {
        let bytes = self.env.get_or_default();
        self.env.set(value);
        self.env().emit_event(FieldsSet {
            bytes,
            result: U256::from(value),
            env: self.env().caller()
        });
    }

    pub fn set_bytes(&mut self, key: u32, value: NamedVariants) {
        self.bytes.set(&key, value);
    }

    pub fn bytes(&self, key: u32) -> Option<NamedVariants> {
        self.bytes.get(&key)
    }
}

fn main() {}
