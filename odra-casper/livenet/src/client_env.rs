use std::{collections::HashMap, sync::Mutex};

use casper_types::{bytesrepr::FromBytes, CLValue};
use odra_casper_shared::consts;
use odra_casper_types::{Address, Balance, CallArgs, OdraType};
use ref_thread_local::RefThreadLocal;

use crate::casper_types_port::timestamp::Timestamp;
use crate::{casper_client::CasperClient, EntrypointArgs, EntrypointCall};

use self::{
    callstack::Callstack, contract_container::ContractContainer,
    contract_register::ContractRegister
};

mod callstack;
mod contract_container;
mod contract_register;

/// Casper Livenet client environment.
#[derive(Default)]
pub struct ClientEnv {
    contracts: ContractRegister,
    callstack: Callstack,
    gas: Mutex<Option<Balance>>,
    casper_client: Option<CasperClient>
}

impl ClientEnv {
    /// Returns a singleton instance of a client environment.
    pub fn instance<'a>() -> ref_thread_local::Ref<'a, ClientEnv> {
        ENV.borrow()
    }

    /// Returns a mutable singleton instance of a client environment.
    pub fn instance_mut<'a>() -> ref_thread_local::RefMut<'a, ClientEnv> {
        ENV.borrow_mut()
    }

    /// Sets up the Casper client.
    pub fn setup_casper_client(&mut self, casper_client: CasperClient) {
        self.casper_client = Some(casper_client);
    }

    pub fn casper_client(&self) -> Option<&CasperClient> {
        // self.casper_client.as_ref().unwrap_or_else(|| panic!("Casper client not set"))
        self.casper_client.as_ref()
    }

    /// Register new contract.
    pub fn register_contract(&mut self, contract: ContractContainer) {
        self.contracts.add(contract);
    }

    /// Push address to callstack.
    pub fn push_on_stack(&mut self, addr: Address) {
        self.callstack.push(addr);
    }

    /// Pop address from callstack.
    pub fn pull_from_stack(&mut self) -> Option<Address> {
        self.callstack.pop()
    }

    /// Call contract.
    pub fn call_contract<T: OdraType>(
        &self,
        addr: Address,
        entrypoint: &str,
        args: &CallArgs
    ) -> T {
        let result = self.contracts.call(&addr, String::from(entrypoint), args);
        let bytes = result.unwrap();
        let (clvalue, _) = CLValue::from_bytes(&bytes).unwrap();
        clvalue.into_t().unwrap()
    }

    /// Get current contract address.
    pub fn current_contract(&self) -> Address {
        self.callstack.current()
    }

    /// Set gas.
    pub fn set_gas<T: Into<Balance>>(&self, gas: T) {
        let new_gas: Balance = gas.into();
        let mut gas = self.gas.lock().unwrap();
        *gas = Some(new_gas);
    }

    /// Get gas and reset it, so it is not used twice.
    pub fn get_gas(&self) -> Balance {
        let mut gas = self.gas.lock().unwrap();
        let current_gas: Balance = gas.expect("Gas not set");
        *gas = None;
        current_gas
    }
}

ref_thread_local::ref_thread_local!(
    static managed ENV: ClientEnv = ClientEnv::default();
);

/// Register existing contract.
pub fn register_existing_contract(
    address: Address,
    entrypoints: HashMap<String, (EntrypointArgs, EntrypointCall)>
) {
    let contract = ContractContainer::new(address, entrypoints);
    ClientEnv::instance_mut().register_contract(contract);
}

/// Deploy WASM file with arguments.
pub async fn deploy_new_contract(
    name: &str,
    mut args: CallArgs,
    entrypoints: HashMap<String, (EntrypointArgs, EntrypointCall)>,
    constructor_name: Option<String>,
    timestamp: Timestamp
) -> Address {
    let gas = get_gas();
    let wasm_name = format!("{}.wasm", name);

    build_args(&mut args, name, constructor_name);
    let address = ClientEnv::instance()
        .casper_client()
        .unwrap()
        .deploy_wasm(&wasm_name, args, gas, timestamp)
        .await;
    let contract = ContractContainer::new(address, entrypoints);
    ClientEnv::instance_mut().register_contract(contract);

    address
}

pub fn build_args(args: &mut CallArgs, name: &str, constructor_name: Option<String>) {
    let contract_package_hash_key = format!("{}_package_hash", name);
    args.insert(consts::ALLOW_KEY_OVERRIDE_ARG, true);
    args.insert(consts::IS_UPGRADABLE_ARG, false);
    args.insert(consts::PACKAGE_HASH_KEY_NAME_ARG, contract_package_hash_key);

    if let Some(constructor_name) = constructor_name {
        args.insert(consts::CONSTRUCTOR_NAME_ARG, constructor_name);
    };
}

/// Call contract's entrypoint.
pub async fn call_contract<T: OdraType>(
    addr: Address,
    entrypoint: &str,
    args: &CallArgs,
    _amount: Option<Balance>,
    timestamp: Timestamp
) -> T {
    match T::cl_type() {
        casper_types::CLType::Unit => {
            call_contract_deploy(addr, entrypoint, args, _amount, timestamp).await;
            T::from_bytes(&[]).unwrap().0
        }
        _ => call_contract_getter_entrypoint(addr, entrypoint, args, _amount)
    }
}

/// Query current contract for a variable's value.
pub async fn get_var_from_current_contract<T: OdraType>(key: &[u8]) -> Option<T> {
    let address = ClientEnv::instance().current_contract();
    ClientEnv::instance()
        .casper_client()
        .unwrap()
        .get_variable_value(address, key)
        .await
}

/// Query current contract for a dictionary's value.
pub async fn get_dict_value_from_current_contract<K: OdraType, T: OdraType>(
    seed: &[u8],
    key: &K
) -> Option<T> {
    let address = ClientEnv::instance().current_contract();
    ClientEnv::instance()
        .casper_client()
        .unwrap()
        .get_dict_value(address, seed, key)
        .await
}

/// Current caller.
pub fn caller() -> Address {
    ClientEnv::instance().casper_client().unwrap().caller()
}

/// Set gas for the next call.
pub fn set_gas<T: Into<Balance>>(gas: T) {
    ClientEnv::instance_mut().set_gas(gas);
}

fn get_gas() -> Balance {
    ClientEnv::instance().get_gas()
}

fn call_contract_getter_entrypoint<T: OdraType>(
    addr: Address,
    entrypoint: &str,
    args: &CallArgs,
    _amount: Option<Balance>
) -> T {
    {
        ClientEnv::instance_mut().push_on_stack(addr);
    }
    let result = { ClientEnv::instance().call_contract(addr, entrypoint, args) };
    {
        ClientEnv::instance_mut().pull_from_stack();
    };
    result
}

async fn call_contract_deploy(
    addr: Address,
    entrypoint: &str,
    args: &CallArgs,
    amount: Option<Balance>,
    timestamp: Timestamp
) {
    let gas = get_gas();
    ClientEnv::instance()
        .casper_client()
        .unwrap()
        .deploy_entrypoint_call(addr, entrypoint, args, amount, gas, timestamp)
        .await;
}
