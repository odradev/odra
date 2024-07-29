use odra_core::{prelude::*, GasReport};
use std::cell::RefCell;
use std::env;
use std::path::PathBuf;

use casper_engine_test_support::{
    DeployItemBuilder, ExecuteRequestBuilder, ARG_AMOUNT, DEFAULT_ACCOUNT_INITIAL_BALANCE,
    DEFAULT_CHAINSPEC_REGISTRY, DEFAULT_GENESIS_CONFIG_HASH, DEFAULT_PAYMENT
};
use std::rc::Rc;

use crate::CasperVm;
use odra_core::casper_types::account::AccountHash;
use odra_core::casper_types::bytesrepr::{Bytes, ToBytes};
use odra_core::casper_types::{runtime_args, BlockTime, Key, Motes, SecretKey};
use odra_core::casper_types::{PublicKey, RuntimeArgs, U512};
use odra_core::consts;
use odra_core::consts::*;
use odra_core::entry_point_callback::EntryPointsCaller;
use odra_core::prelude::*;
use odra_core::EventError;
use odra_core::{
    host::{HostContext, HostEnv},
    CallDef, ContractEnv, VmError
};

/// HostContext utilizing the Casper test virtual machine.
pub struct CasperHost {
    /// The Casper VM used by the host.
    pub vm: Rc<RefCell<CasperVm>>
}

impl HostContext for CasperHost {
    fn set_caller(&self, caller: Address) {
        self.vm.borrow_mut().set_caller(caller)
    }

    fn set_gas(&self, gas: u64) {
        // Set gas does nothing in this context
    }

    fn caller(&self) -> Address {
        self.vm.borrow().get_caller()
    }

    fn get_account(&self, index: usize) -> Address {
        self.vm.borrow().get_account(index)
    }

    fn balance_of(&self, address: &Address) -> U512 {
        self.vm.borrow().balance_of(address)
    }

    fn advance_block_time(&self, time_diff: u64) {
        self.vm.borrow_mut().advance_block_time(time_diff)
    }

    fn block_time(&self) -> u64 {
        self.vm.borrow().block_time()
    }

    fn get_event(&self, contract_address: &Address, index: u32) -> Result<Bytes, EventError> {
        if !contract_address.is_contract() {
            return Err(EventError::TriedToQueryEventForNonContract);
        }
        self.vm.borrow().get_event(contract_address, index)
    }

    fn get_native_event(
        &self,
        contract_address: &Address,
        index: u32
    ) -> Result<Bytes, EventError> {
        if !contract_address.is_contract() {
            return Err(EventError::TriedToQueryEventForNonContract);
        }
        self.vm.borrow().get_native_event(contract_address, index)
    }

    fn get_events_count(&self, address: &Address) -> Result<u32, EventError> {
        self.vm.borrow().get_events_count(address)
    }

    fn get_native_events_count(&self, contract_address: &Address) -> Result<u32, EventError> {
        self.vm.borrow().get_native_events_count(contract_address)
    }

    fn call_contract(
        &self,
        address: &Address,
        call_def: CallDef,
        use_proxy: bool
    ) -> OdraResult<Bytes> {
        let mut opt_result: Option<Bytes> = None;
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            opt_result = Some(
                self.vm
                    .borrow_mut()
                    .call_contract(address, call_def, use_proxy)
            );
        }));

        match opt_result {
            Some(result) => Ok(result),
            None => {
                let error = self.vm.borrow().error();
                Err(error.unwrap_or(OdraError::VmError(VmError::Panic)))
            }
        }
    }

    fn new_contract(
        &self,
        name: &str,
        init_args: RuntimeArgs,
        entry_points_caller: EntryPointsCaller
    ) -> OdraResult<Address> {
        let mut opt_result: Option<Address> = None;
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            opt_result = Some(self.vm.borrow_mut().new_contract(
                name,
                init_args,
                entry_points_caller
            ));
        }));

        match opt_result {
            Some(result) => Ok(result),
            None => {
                let error = self.vm.borrow().error();
                Err(error.unwrap_or(OdraError::VmError(VmError::Panic)))
            }
        }
    }

    fn register_contract(
        &self,
        address: Address,
        contract_name: String,
        entry_points_caller: EntryPointsCaller
    ) {
        panic!("register_contract is not supported in CasperHost");
    }

    fn contract_env(&self) -> ContractEnv {
        unreachable!()
    }

    fn gas_report(&self) -> GasReport {
        self.vm.borrow().gas_report().clone()
    }

    fn last_call_gas_cost(&self) -> u64 {
        self.vm.borrow().last_call_gas_cost()
    }

    fn sign_message(&self, message: &Bytes, address: &Address) -> Bytes {
        self.vm.borrow().sign_message(message, address)
    }

    fn public_key(&self, address: &Address) -> PublicKey {
        self.vm.borrow().public_key(address)
    }

    fn transfer(&self, to: Address, amount: U512) -> OdraResult<()> {
        self.vm.borrow_mut().transfer(to, amount)
    }
}

impl CasperHost {
    /// Creates a new instance of the host.
    pub fn new(vm: Rc<RefCell<CasperVm>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self { vm }))
    }
}
