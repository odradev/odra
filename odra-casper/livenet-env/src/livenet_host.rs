//! Livenet implementation of HostContext for HostEnv.
use std::sync::{Arc, RwLock};
use std::thread::sleep;

use odra_casper_client::casper_client::CasperClient;
use odra_casper_client::log::info;
use odra_core::callstack::{Callstack, CallstackElement, Entrypoint};
use odra_core::contract_container::ContractContainer;
use odra_core::contract_register::ContractRegister;
use odra_core::event::EventError;
use odra_core::event::EventError::CouldntExtractEventData;
use odra_core::prelude::*;
use odra_core::{
    Address, Bytes, CallDef, ContractEnv, EntryPointsCaller, HostContext, OdraError, PublicKey,
    RuntimeArgs, ToBytes, U512
};

use crate::livenet_contract_env::LivenetContractEnv;

/// LivenetHost struct.
pub struct LivenetHost {
    casper_client: Rc<RefCell<CasperClient>>,
    contract_register: Arc<RwLock<ContractRegister>>,
    contract_env: Rc<ContractEnv>,
    callstack: Rc<RefCell<Callstack>>
}

impl LivenetHost {
    /// Creates a new instance of LivenetHost.
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self::new_instance()))
    }

    fn new_instance() -> Self {
        let casper_client: Rc<RefCell<CasperClient>> = Default::default();
        let callstack: Rc<RefCell<Callstack>> = Default::default();
        let contract_register = Arc::new(RwLock::new(Default::default()));
        let livenet_contract_env = LivenetContractEnv::new(
            casper_client.clone(),
            callstack.clone(),
            contract_register.clone()
        );
        let contract_env = Rc::new(ContractEnv::new(0, livenet_contract_env));
        Self {
            casper_client,
            contract_register,
            contract_env,
            callstack
        }
    }
}

impl HostContext for LivenetHost {
    fn set_caller(&self, caller: Address) {
        self.casper_client.borrow_mut().set_caller(caller);
    }

    fn set_gas(&self, gas: u64) {
        self.casper_client.borrow_mut().set_gas(gas);
    }

    fn caller(&self) -> Address {
        self.casper_client.borrow().caller()
    }

    fn get_account(&self, index: usize) -> Address {
        self.casper_client.borrow().get_account(index)
    }

    fn balance_of(&self, address: &Address) -> U512 {
        self.casper_client.borrow().get_balance(address)
    }

    fn advance_block_time(&self, time_diff: u64) {
        info(format!(
            "advance_block_time called - Waiting for {} ms",
            time_diff
        ));
        sleep(std::time::Duration::from_millis(time_diff));
    }

    fn get_event(&self, contract_address: &Address, index: u32) -> Result<Bytes, EventError> {
        self.casper_client
            .borrow()
            .get_event(contract_address, index)
            .map_err(|_| CouldntExtractEventData)
    }

    fn get_events_count(&self, contract_address: &Address) -> u32 {
        self.casper_client.borrow().events_count(contract_address)
    }

    fn call_contract(
        &self,
        address: &Address,
        call_def: CallDef,
        use_proxy: bool
    ) -> Result<Bytes, OdraError> {
        if !call_def.is_mut() {
            self.callstack
                .borrow_mut()
                .push(CallstackElement::Entrypoint(Entrypoint::new(
                    *address,
                    call_def.clone()
                )));
            let result = self
                .contract_register
                .read()
                .expect("Couldn't read contract register.")
                .call(address, call_def);
            self.callstack.borrow_mut().pop();
            return result;
        }
        match use_proxy {
            true => self
                .casper_client
                .borrow_mut()
                .deploy_entrypoint_call_with_proxy(*address, call_def),
            false => {
                self.casper_client
                    .borrow_mut()
                    .deploy_entrypoint_call(*address, call_def);
                Ok(
                    ().to_bytes()
                        .expect("Couldn't serialize (). This shouldn't happen.")
                        .into()
                )
            }
        }
    }

    fn register_contract(&self, address: Address, entry_points_caller: EntryPointsCaller) {
        self.contract_register
            .write()
            .expect("Couldn't write contract register.")
            .add(address, ContractContainer::new(entry_points_caller));
    }

    fn new_contract(
        &self,
        name: &str,
        init_args: RuntimeArgs,
        entry_points_caller: EntryPointsCaller
    ) -> Address {
        let address = self.casper_client.borrow_mut().deploy_wasm(name, init_args);
        self.register_contract(address, entry_points_caller);
        address
    }

    fn contract_env(&self) -> ContractEnv {
        (*self.contract_env).clone()
    }

    fn print_gas_report(&self) {
        // Todo: implement
        println!("Gas report is unavailable for livenet");
    }

    fn last_call_gas_cost(&self) -> u64 {
        // Todo: implement
        0
    }

    fn sign_message(&self, message: &Bytes, address: &Address) -> Bytes {
        self.casper_client
            .borrow()
            .sign_message(message, address)
            .unwrap()
    }

    fn public_key(&self, address: &Address) -> PublicKey {
        self.casper_client.borrow().address_public_key(address)
    }
}
