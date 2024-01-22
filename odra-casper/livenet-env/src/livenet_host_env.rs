use crate::livenet_contract_env::LivenetContractEnv;
use odra_casper_client::casper_client::CasperClient;
use odra_core::callstack::{Callstack, CallstackElement, Entrypoint};
use odra_core::casper_types::bytesrepr::deserialize;
use odra_core::contract_container::ContractContainer;
use odra_core::contract_register::ContractRegister;
use odra_core::event::EventError;
use odra_core::prelude::*;
use odra_core::{
    Address, Bytes, CLType, CallDef, ContractEnv, EntryPointsCaller, HostContext, OdraError,
    PublicKey, RuntimeArgs, U512
};
use casper_node_port::*;
use std::sync::{Arc, RwLock};
use std::thread::sleep;
use odra_casper_client::casper_node_port;

pub struct LivenetEnv {
    casper_client: Rc<RefCell<CasperClient>>,
    contract_register: Arc<RwLock<ContractRegister>>,
    contract_env: Rc<ContractEnv>,
    callstack: Rc<RefCell<Callstack>>
}

impl LivenetEnv {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self::new_instance()))
    }

    pub fn new_instance() -> Self {
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

impl HostContext for LivenetEnv {
    fn set_caller(&self, _caller: Address) {
        todo!()
    }

    fn set_gas(&self, gas: u64) {
        self.casper_client.borrow_mut().set_gas(gas);
    }

    fn caller(&self) -> Address {
        self.casper_client.borrow().caller()
    }

    fn get_account(&self, _index: usize) -> Address {
        todo!()
    }

    fn balance_of(&self, _address: &Address) -> U512 {
        todo!()
    }

    fn advance_block_time(&self, time_diff: u64) {
        // Todo: implement logging, especially for such cases:
        // info!("advance_block_time called - Waiting for {} ms", time_diff);
        sleep(std::time::Duration::from_millis(time_diff));
    }

    fn get_event(&self, _contract_address: &Address, _index: i32) -> Result<Bytes, EventError> {
        todo!()
    }

    fn get_events_count(&self, contract_address: &Address) -> u32 {
        let d = self.casper_client.borrow().query_global_state_path(*contract_address, "__events_count".to_string());
        let c = d.unwrap().stored_value;
        let ec_uref = match d.unwrap().stored_value {
            casper_node_port::rpcs::StoredValue::Contract(contract) => {
                contract.named_keys().find_map(|named_key |{
                    if named_key.name == "__events_count" {
                        Some(named_key.key.clone())
                    } else {
                        None
                    }
                })
            },
            _ => panic!("Not a contract")
        }.unwrap();

        // next - extract value from uref

        // let bytes = self
        //     .casper_client
        //     .borrow()
        //     .query_global_state_for_contract_hash(contract_address);
        //     .unwrap()
        //     .to_vec();
        // // dbg!(bytes.clone());
        // deserialize(bytes).unwrap();
        2
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
                .unwrap()
                .call(address, call_def);
            self.callstack.borrow_mut().pop();
            return result;
        }
        match use_proxy {
            true => Ok(self
                .casper_client
                .borrow_mut()
                .deploy_entrypoint_call_with_proxy(*address, call_def)),
            false => {
                self.casper_client
                    .borrow_mut()
                    .deploy_entrypoint_call(*address, call_def);
                Ok(Default::default())
            }
        }
    }

    fn register_contract(&self, address: Address, entry_points_caller: EntryPointsCaller) {
        self.contract_register.write().unwrap().add(
            address,
            ContractContainer::new(&address.to_string(), entry_points_caller)
        );
    }

    fn new_contract(
        &self,
        name: &str,
        init_args: Option<RuntimeArgs>,
        entry_points_caller: EntryPointsCaller
    ) -> Address {
        let mut args = match init_args {
            None => RuntimeArgs::new(),
            Some(args) => args
        };
        // todo: move this up the stack
        args.insert("odra_cfg_is_upgradable", false).unwrap();
        args.insert("odra_cfg_allow_key_override", true).unwrap();
        args.insert(
            "odra_cfg_package_hash_key_name",
            format!("{}_package_hash", name)
        )
        .unwrap();
        let address = self.casper_client.borrow_mut().deploy_wasm(name, args);
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

    fn sign_message(&self, _message: &Bytes, _address: &Address) -> Bytes {
        todo!()
    }

    fn public_key(&self, _address: &Address) -> PublicKey {
        todo!()
    }
}
