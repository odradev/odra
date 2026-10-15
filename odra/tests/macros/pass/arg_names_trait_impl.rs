//! Arguments named like generated locals in a module implemented as a trait impl.
#![deny(warnings)]

use odra::prelude::*;

pub trait Counter {
    fn init(&mut self, env: u32);
    fn add(&mut self, exec_env: u32, env_rc: u32) -> u32;
    fn get(&self, contract: u32, named_args: u32, result: u32) -> u32;
}

#[odra::module]
pub struct TraitCounter {
    value: Var<u32>
}

#[odra::module]
impl Counter for TraitCounter {
    fn init(&mut self, env: u32) {
        self.value.set(env);
    }

    fn add(&mut self, exec_env: u32, env_rc: u32) -> u32 {
        let value = self.value.get_or_default() + exec_env + env_rc;
        self.value.set(value);
        value
    }

    fn get(&self, contract: u32, named_args: u32, result: u32) -> u32 {
        self.value.get_or_default() + contract + named_args + result
    }
}

fn main() {}
