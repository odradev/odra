//! Livenet implementation of HostContext for HostEnv.

use crate::error;
use crate::livenet_contract_env::LivenetContractEnv;
use odra_casper_rpc_client::casper_client::CasperClient;
use odra_casper_rpc_client::log::info;
use odra_core::callstack::{Callstack, CallstackElement};
use odra_core::casper_types::Timestamp;
use odra_core::entry_point_callback::EntryPointsCaller;
use odra_core::ExecutionError::{UnexpectedError, User};
use odra_core::{
    casper_types::{bytesrepr::Bytes, PublicKey, RuntimeArgs, U512},
    host::HostContext,
    Address, CallDef, ContractEnv, GasReport, OdraError
};
use odra_core::{prelude::*, EventError, OdraResult};
use odra_core::{ContractContainer, ContractRegister};
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;
use std::thread::sleep;
use tokio::runtime::Runtime;

/// Enum representing a contract identifier used by Livenet Host.
#[derive(Debug)]
pub enum ContractId {
    /// Contract name.
    Name(String),
    /// Contract address.
    Address(Address)
}

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

    fn get_events_count(&self, contract_address: &Address) -> u32 {
        let rt = Runtime::new().unwrap();
        let client = self.casper_client.borrow();
        rt.block_on(async { client.events_count(contract_address).await })
            .unwrap_or_default()
    }

    fn call_contract(
        &self,
        address: &Address,
        call_def: CallDef,
        use_proxy: bool
    ) -> OdraResult<Bytes> {
        if !call_def.is_mut() {
            self.callstack
                .borrow_mut()
                .push(CallstackElement::new_contract_call(
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
                    .map_err(|e| {
                        self.map_error_code_to_odra_error(
                            ContractId::Address(*address),
                            &e.error_message()
                        )
                    })
            }),
            false => rt.block_on(async {
                let r = client
                    .deploy_entrypoint_call(*address, call_def, timestamp)
                    .await;
                r.map_err(|e| {
                    self.map_error_code_to_odra_error(
                        ContractId::Address(*address),
                        &e.error_message()
                    )
                })
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
        let wasm_path = find_wasm_file_path(name);
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
                    todo!("Handle error: {:?}", e);
                }
            }
        };
        self.register_contract(address, name.to_string(), entry_points_caller);
        Ok(address)
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
        Ok(rt
            .block_on(async { client.transfer(to, amount, timestamp).await })
            .map_err(|e| {
                self.map_error_code_to_odra_error(
                    ContractId::Address(client.caller()),
                    &e.error_message()
                )
            })?)
    }
}

impl LivenetHost {
    fn map_error_code_to_odra_error(&self, contract_id: ContractId, error_msg: &str) -> OdraError {
        let found = match contract_id {
            ContractId::Name(contract_name) => error::find(&contract_name, error_msg).ok(),
            ContractId::Address(addr) => match self.contract_register.read().unwrap().get(&addr) {
                Some(contract_name) => error::find(contract_name, error_msg).ok(),
                None => None
            }
        };

        match found {
            None => OdraError::ExecutionError(UnexpectedError),
            Some((message, error)) => {}
        }
    }
}

fn find_wasm_file_path(wasm_file_name: &str) -> PathBuf {
    let mut path = PathBuf::from("wasm")
        .join(wasm_file_name)
        .with_extension("wasm");
    let mut checked_paths = vec![];
    for _ in 0..2 {
        if path.exists() && path.is_file() {
            info(format!("Found wasm under {:?}.", path));
            return path;
        } else {
            checked_paths.push(path.clone());
            path = path.parent().unwrap().to_path_buf();
        }
    }
    odra_casper_rpc_client::log::error(format!("Could not find wasm under {:?}.", checked_paths));
    panic!("Wasm not found");
}
