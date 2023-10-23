use crate::casper_vm::CasperVm;
use odra_core::prelude::{collections::*, *};
use std::cell::RefCell;
use std::env;
use std::path::PathBuf;

use casper_engine_test_support::{
    DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, ARG_AMOUNT,
    DEFAULT_ACCOUNT_INITIAL_BALANCE, DEFAULT_CHAINSPEC_REGISTRY, DEFAULT_GENESIS_CONFIG,
    DEFAULT_GENESIS_CONFIG_HASH, DEFAULT_PAYMENT
};
use std::rc::Rc;

use casper_execution_engine::core::engine_state::{GenesisAccount, RunGenesisRequest};
use odra_casper_shared::consts;
use odra_casper_shared::consts::*;
use odra_core::entry_point_callback::EntryPointsCaller;
use odra_core::{CallDef, ContractEnv, HostContext, HostEnv};
use odra_types::casper_types::account::AccountHash;
use odra_types::casper_types::bytesrepr::{Bytes, ToBytes};
use odra_types::casper_types::{
    runtime_args, BlockTime, ContractPackageHash, Key, Motes, SecretKey
};
use odra_types::{Address, PublicKey, U512};
use odra_types::{EventData, RuntimeArgs};

pub struct CasperHost {
    pub vm: Rc<RefCell<CasperVm>>
}

impl HostContext for CasperHost {
    fn set_caller(&self, caller: Address) {
        self.vm.borrow_mut().set_caller(caller)
    }

    fn get_account(&self, index: usize) -> Address {
        self.vm.borrow().get_account(index)
    }

    fn advance_block_time(&self, time_diff: BlockTime) {
        self.vm.borrow_mut().advance_block_time(time_diff)
    }

    fn get_event(&self, contract_address: Address, index: i32) -> Option<EventData> {
        self.vm.borrow().get_event(contract_address, index)
    }

    fn attach_value(&self, amount: U512) {
        self.vm.borrow_mut().attach_value(amount)
    }

    fn call_contract(&self, address: &Address, call_def: CallDef) -> Bytes {
        self.vm.borrow_mut().call_contract(address, call_def)
    }

    fn new_contract(
        &self,
        name: &str,
        init_args: Option<RuntimeArgs>,
        entry_points_caller: Option<EntryPointsCaller>
    ) -> Address {
        self.vm
            .borrow_mut()
            .new_contract(name, init_args, entry_points_caller)
    }

    fn contract_env(&self) -> ContractEnv {
        unreachable!()
    }

    fn print_gas_report(&self) {
        self.vm.borrow().print_gas_report()
    }
}

impl CasperHost {
    pub fn new(vm: Rc<RefCell<CasperVm>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self { vm }))
    }
}
