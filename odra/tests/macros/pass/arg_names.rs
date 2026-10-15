//! Entry point arguments may be named like the locals of the code Odra generates around an entry
//! point. The crate has no build script (no `odra_build::build()`), so `deny(warnings)` also
//! checks that the `odra_module` cfg does not trigger `unexpected_cfgs`.
#![deny(warnings)]

use odra::casper_types::U512;
use odra::prelude::*;
use odra::ContractRef;

#[odra::module]
pub struct ArgNames {
    value: Var<u32>,
    owner: Var<Address>
}

#[odra::module]
impl ArgNames {
    pub fn init(&mut self, env: u32, contract: Address) {
        self.value.set(env);
        self.owner.set(contract);
    }

    pub fn set_owner(&mut self, contract: Address) {
        self.owner.set(contract);
    }

    pub fn add(&mut self, exec_env: u32, env_rc: u32) -> u32 {
        let value = self.value.get_or_default() + exec_env + env_rc;
        self.value.set(value);
        value
    }

    pub fn echo(
        &self,
        named_args: u32,
        result: u32,
        call_def: u32,
        contract_env: u32,
        env: u32
    ) -> u32 {
        named_args + result + call_def + contract_env + env
    }

    pub fn getter(&self, contract: u32, env_rc: u32) -> u32 {
        contract + env_rc
    }

    #[odra(payable)]
    pub fn deposit(&mut self, exec_env: u32, contract: u32) -> U512 {
        self.env().attached_value() + U512::from(exec_env + contract)
    }

    #[odra(non_reentrant)]
    pub fn guarded(&mut self, exec_env: u32, named_args: u32, result: u32) -> u32 {
        exec_env + named_args + result
    }

    #[odra(offchain)]
    pub fn offchain_sum(&self, contract: u32, exec_env: u32, named_args: u32, env: u32) -> u32 {
        contract + exec_env + named_args + env
    }

    pub fn echo_of(&self, contract: Address, named_args: u32, result: u32) -> u32 {
        ArgNamesExtContractRef::new(self.env(), contract).echo(named_args, result, 0, 1, 2)
    }
}

#[odra::external_contract]
pub trait ArgNamesExt {
    fn echo(&self, named_args: u32, result: u32, call_def: u32, contract_env: u32, env: u32)
        -> u32;
    fn add(&mut self, exec_env: u32, env_rc: u32) -> u32;
    fn set_owner(&mut self, contract: Address);
}

fn main() {}
