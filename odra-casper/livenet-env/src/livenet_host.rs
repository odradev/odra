//! Livenet implementation of HostContext for HostEnv.

use crate::error;
use crate::livenet_contract_env::LivenetContractEnv;
use odra_casper_rpc_client::casper_client::CasperClient;
use odra_casper_rpc_client::log::info;
use odra_casper_rpc_client::utils::find_wasm_file_path;
use odra_core::callstack::{Callstack, CallstackElement};
use odra_core::casper_types::Timestamp;
use odra_core::entry_point_callback::EntryPointsCaller;
use odra_core::{
    casper_types::{bytesrepr::Bytes, PublicKey, RuntimeArgs, U512},
    host::HostContext,
    CallDef, ContractEnv, GasReport
};
use odra_core::{prelude::*, EventError, VmError};
use odra_core::{ContractContainer, ContractRegister};
use std::fs;
use std::sync::RwLock;
use std::thread::sleep;
use tokio::runtime::Runtime;

/// LivenetHost struct.
pub struct LivenetHost {
    casper_client: Rc<RefCell<CasperClient>>,
    contract_register: Rc<RwLock<ContractRegister>>,
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
        let contract_register = Rc::new(RwLock::new(Default::default()));
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

    fn get_validator(&self, index: usize) -> PublicKey {
        let rt = Runtime::new().unwrap();
        let client = self.casper_client.borrow_mut();
        rt.block_on(async { client.get_validator(index).await })
    }

    fn remove_validator(&self, _index: usize) {
        panic!("remove_validator is not supported on livenet");
    }

    fn balance_of(&self, address: &Address) -> U512 {
        let rt = Runtime::new().unwrap();
        let client = self.casper_client.borrow();
        rt.block_on(async { client.get_balance(address).await })
    }

    fn advance_block_time(&self, time_diff: u64) {
        info(format!(
            "advance_block_time called - Waiting for {} ms",
            time_diff
        ));
        sleep(std::time::Duration::from_millis(time_diff));
    }

    fn advance_with_auctions(&self, diff: u64) {
        println!("advance_with_auctions called - Waiting for {diff} ms");
        sleep(std::time::Duration::from_millis(diff));
    }

    fn auction_delay(&self) -> u64 {
        let rt = Runtime::new().unwrap();
        let client = self.casper_client.borrow_mut();
        rt.block_on(async { client.auction_delay().await })
    }

    fn unbonding_delay(&self) -> u64 {
        let rt = Runtime::new().unwrap();
        let client = self.casper_client.borrow_mut();
        rt.block_on(async { client.unbonding_delay().await })
    }

    fn delegated_amount(&self, delegator: Address, validator: PublicKey) -> U512 {
        let rt = Runtime::new().unwrap();
        let client = self.casper_client.borrow_mut();
        rt.block_on(async { client.delegated_amount(delegator, validator).await })
    }

    fn block_time(&self) -> u64 {
        let rt = Runtime::new().unwrap();
        let client = self.casper_client.borrow();
        rt.block_on(async { client.get_block_time().await.unwrap() })
    }

    fn get_event(&self, contract_address: &Address, index: u32) -> Result<Bytes, EventError> {
        let rt = Runtime::new().unwrap();
        let client = self.casper_client.borrow();
        rt.block_on(async { client.get_event(contract_address, index).await })
            .map_err(|_| EventError::CouldntExtractEventData)
    }

    fn get_native_event(
        &self,
        _contract_address: &Address,
        _index: u32
    ) -> Result<Bytes, EventError> {
        // TODO: Implement
        Err(EventError::CouldntExtractEventData)
    }

    fn get_events_count(&self, contract_address: &Address) -> Result<u32, EventError> {
        let rt = Runtime::new().unwrap();
        let client = self.casper_client.borrow();
        rt.block_on(async { client.events_count(contract_address).await })
            .ok_or(EventError::CouldntExtractEventData)
    }

    fn get_native_events_count(&self, _contract_address: &Address) -> Result<u32, EventError> {
        // TODO: Implement
        Err(EventError::CouldntExtractEventData)
    }

    fn call_contract(
        &self,
        address: &Address,
        call_def: CallDef,
        use_proxy: bool
    ) -> OdraResult<Bytes> {
        if !call_def.is_mut() {
            let contract_name = self
                .contract_register
                .read()
                .expect("Couldn't read contract register.")
                .get(address)
                .unwrap_or_default()
                .to_string();

            self.callstack
                .borrow_mut()
                .push(CallstackElement::new_contract_call(
                    contract_name,
                    *address,
                    call_def.clone()
                ));
            let result = self
                .contract_register
                .read()
                .expect("Couldn't read contract register.")
                .call(address, call_def);
            self.callstack.borrow_mut().pop();
            return result;
        }
        let timestamp = Timestamp::now();
        let rt = Runtime::new().unwrap();
        let client = self.casper_client.borrow_mut();
        match use_proxy {
            true => rt.block_on(async {
                client
                    .deploy_entrypoint_call_with_proxy(*address, call_def, timestamp)
                    .await
                    .map_err(|e| e.error_message())
                    .map_err(Self::error_msg_to_odra_error)
            }),
            false => rt.block_on(async {
                client
                    .deploy_entrypoint_call(*address, call_def, timestamp)
                    .await
                    .map_err(|e| e.error_message())
                    .map_err(Self::error_msg_to_odra_error)
            })
        }
    }

    fn new_contract(
        &self,
        name: &str,
        init_args: RuntimeArgs,
        entry_points_caller: EntryPointsCaller
    ) -> OdraResult<Address> {
        let timestamp = Timestamp::now();
        let wasm_path = find_wasm_file_path(name)?;
        let wasm_bytes = fs::read(wasm_path).unwrap();
        let address = {
            let mut client = self.casper_client.borrow_mut();
            let rt = Runtime::new().unwrap();
            match rt.block_on(async {
                client
                    .deploy_wasm(name, init_args, timestamp, wasm_bytes)
                    .await
            }) {
                Ok(addr) => addr,
                Err(e) => {
                    log::error!("Error deploying contract: {}", e);
                    return Err(ExecutionError::ContractDeploymentError.into());
                }
            }
        };
        self.register_contract(address, name.to_string(), entry_points_caller);
        Ok(address)
    }

    fn upgrade_contract(
        &self,
        name: &str,
        contract_to_upgrade: Address,
        upgrade_args: RuntimeArgs,
        entry_points_caller: EntryPointsCaller
    ) -> OdraResult<Address> {
        let timestamp = Timestamp::now();
        let wasm_path = find_wasm_file_path(name)?;
        let wasm_bytes = fs::read(wasm_path).unwrap();
        let mut client = self.casper_client.borrow_mut();
        let rt = Runtime::new().unwrap();
        match rt.block_on(async {
            client
                .deploy_wasm(name, upgrade_args, timestamp, wasm_bytes)
                .await
        }) {
            Ok(_) => {}
            Err(e) => {
                log::error!("Error deploying contract: {}", e);
                return Err(ExecutionError::ContractDeploymentError.into());
            }
        }
        self.register_contract(contract_to_upgrade, name.to_string(), entry_points_caller);
        Ok(contract_to_upgrade)
    }

    fn register_contract(
        &self,
        address: Address,
        contract_name: String,
        entry_points_caller: EntryPointsCaller
    ) {
        self.contract_register
            .write()
            .expect("Couldn't write contract register.")
            .add(
                address,
                ContractContainer::new(&contract_name, entry_points_caller)
            );
    }

    fn contract_env(&self) -> ContractEnv {
        (*self.contract_env).clone()
    }

    fn gas_report(&self) -> GasReport {
        println!("Gas report is unavailable for livenet");
        todo!()
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

    fn transfer(&self, to: Address, amount: U512) -> OdraResult<()> {
        let rt = Runtime::new().unwrap();
        let timestamp = Timestamp::now();
        let client = self.casper_client.borrow_mut();
        rt.block_on(async { client.transfer(to, amount, timestamp).await })
            .map_err(|e| e.error_message())
            .map_err(Self::error_msg_to_odra_error)
    }
}

impl LivenetHost {
    fn error_msg_to_odra_error(error_msg: String) -> OdraError {
        match error::find(&error_msg) {
            Ok(err) => err,
            _ => OdraError::VmError(VmError::Other(error_msg))
        }
    }
}
