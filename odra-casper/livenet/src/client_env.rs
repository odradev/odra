use std::{collections::BTreeMap, sync::Mutex};

use odra_casper_shared::consts;
use odra_core::{
    casper_types::{
        bytesrepr::{FromBytes, ToBytes},
        CLType, CLTyped, CLValue, RuntimeArgs, U512
    },
    Address
};
use ref_thread_local::RefThreadLocal;

use crate::{casper_client::CasperClient, EntrypointArgs, EntrypointCall};

use self::{
    callstack::Callstack, contract_container::ContractContainer,
    contract_register::ContractRegister
};

mod callstack;
mod contract_container;
mod contract_register;

/// Casper Lievent client environment.
#[derive(Default)]
struct ClientEnv {
    contracts: ContractRegister,
    callstack: Callstack,
    gas: Mutex<Option<U512>>
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
    pub fn call_contract<T: CLTyped + FromBytes>(
        &self,
        addr: Address,
        entrypoint: &str,
        args: &RuntimeArgs
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
    pub fn set_gas<T: Into<U512>>(&self, gas: T) {
        let new_gas: U512 = gas.into();
        let mut gas = self.gas.lock().unwrap();
        *gas = Some(new_gas);
    }

    /// Get gas and reset it, so it is not used twice.
    pub fn get_gas(&self) -> U512 {
        let mut gas = self.gas.lock().unwrap();
        let current_gas: U512 = gas.expect("Gas not set");
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
    entrypoints: BTreeMap<String, (EntrypointArgs, EntrypointCall)>
) {
    let contract = ContractContainer::new(address, entrypoints);
    ClientEnv::instance_mut().register_contract(contract);
}

/// Deploy WASM file with arguments.
pub fn deploy_new_contract(
    name: &str,
    mut args: RuntimeArgs,
    entrypoints: BTreeMap<String, (EntrypointArgs, EntrypointCall)>,
    constructor_name: Option<String>
) -> Address {
    let gas = get_gas();
    let wasm_name = format!("{}.wasm", name);
    let contract_package_hash_key = format!("{}_package_hash", name);
    args.insert(consts::ALLOW_KEY_OVERRIDE_ARG, true).unwrap();
    args.insert(consts::IS_UPGRADABLE_ARG, false).unwrap();
    args.insert(consts::PACKAGE_HASH_KEY_NAME_ARG, contract_package_hash_key)
        .unwrap();

    if let Some(constructor_name) = constructor_name {
        args.insert(consts::CONSTRUCTOR_NAME_ARG, constructor_name)
            .unwrap();
    };

    let address = CasperClient::new().deploy_wasm(&wasm_name, args, gas);
    let contract = ContractContainer::new(address, entrypoints);
    ClientEnv::instance_mut().register_contract(contract);

    address
}

/// Call contract's entrypoint.
pub fn call_contract<T: CLTyped + FromBytes>(
    addr: Address,
    entrypoint: &str,
    args: &RuntimeArgs,
    _amount: Option<U512>
) -> T {
    match T::cl_type() {
        CLType::Unit => {
            call_contract_deploy(addr, entrypoint, args, _amount);
            T::from_bytes(&[]).unwrap().0
        }
        _ => call_contract_getter_entrypoint(addr, entrypoint, args, _amount)
    }
}

/// Query current contract for a variable's value.
pub fn get_var_from_current_contract<T: FromBytes>(key: &[u8]) -> Option<T> {
    let address = ClientEnv::instance().current_contract();
    CasperClient::new().get_variable_value(address, key)
}

/// Query current contract for a dictionary's value.
pub fn get_dict_value_from_current_contract<K: ToBytes, T: FromBytes>(
    seed: &[u8],
    key: &K
) -> Option<T> {
    let address = ClientEnv::instance().current_contract();
    CasperClient::new().get_dict_value(address, seed, key)
}

/// Current caller.
pub fn caller() -> Address {
    CasperClient::new().caller()
}

/// Set gas for the next call.
pub fn set_gas<T: Into<U512>>(gas: T) {
    ClientEnv::instance_mut().set_gas(gas);
}

fn get_gas() -> U512 {
    ClientEnv::instance().get_gas()
}

fn call_contract_getter_entrypoint<T: CLTyped + FromBytes>(
    addr: Address,
    entrypoint: &str,
    args: &RuntimeArgs,
    _amount: Option<U512>
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

fn call_contract_deploy(addr: Address, entrypoint: &str, args: &RuntimeArgs, amount: Option<U512>) {
    let gas = get_gas();
    CasperClient::new().deploy_entrypoint_call(addr, entrypoint, args, amount, gas);
}
