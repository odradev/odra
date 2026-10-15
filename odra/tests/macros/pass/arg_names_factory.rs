//! Arguments named like generated locals in a factory module.
#![deny(warnings)]

use odra::prelude::*;

#[odra::module(factory = on)]
pub struct FactoryArgNames {
    value: Var<u32>
}

#[odra::module(factory = on)]
impl FactoryArgNames {
    pub fn init(&mut self, env: u32, contract: u32) {
        self.value.set(env + contract);
    }

    pub fn add(&mut self, exec_env: u32, env_rc: u32, named_args: u32) -> u32 {
        let value = self.value.get_or_default() + exec_env + env_rc + named_args;
        self.value.set(value);
        value
    }

    pub fn get(&self, result: u32, call_def: u32, contract_env: u32) -> u32 {
        self.value.get_or_default() + result + call_def + contract_env
    }

    #[odra(payable)]
    pub fn deposit(&mut self, exec_env: u32, contract: u32) {
        self.value.set(exec_env + contract);
    }

    #[odra(non_reentrant)]
    pub fn guarded(&mut self, exec_env: u32, env: u32) {
        self.value.set(exec_env + env);
    }

    pub fn upgrade(&mut self, env: u32, contract: u32) {
        self.value.set(env + contract);
    }
}

fn main() {}
