use std::{collections::HashMap, sync::Mutex};

use casper_types::{bytesrepr::FromBytes, CLValue};
use odra_casper_types::{Address, Balance, CallArgs, OdraType};
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
    gas: Mutex<Option<Balance>>
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
    pub fn call_contract<T: OdraType>(&self, addr: Address, entrypoint: &str, args: &CallArgs) -> T {
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
pub fn deploy_new_contract(
    name: &str,
    args: CallArgs,
    entrypoints: HashMap<String, (EntrypointArgs, EntrypointCall)>
) -> Address {
    let gas = get_gas();
    let wasm_name = format!("{}.wasm", name);
    let address = CasperClient::new().deploy_wasm(&wasm_name, args, gas);

    let contract = ContractContainer::new(address, entrypoints);
    ClientEnv::instance_mut().register_contract(contract);

    address
}

/// Call contract's entrypoint.
pub fn call_contract<T: OdraType>(
    addr: Address,
    entrypoint: &str,
    args: &CallArgs,
    _amount: Option<Balance>
) -> T {
    match T::cl_type() {
        casper_types::CLType::Unit => {
            call_contract_deploy(addr, entrypoint, args, _amount);
            T::from_bytes(&[]).unwrap().0
        }
        _ => call_contract_getter_entrypoint(addr, entrypoint, args, _amount)
    }
}

/// Query current contract for a variable's value.
pub fn get_var_from_current_contract<T: OdraType>(key: &str) -> Option<T> {
    let address = ClientEnv::instance().current_contract();
    CasperClient::new().get_variable_value(address, key)
}

/// Query current contract for a dictionary's value.
pub fn get_dict_value_from_current_contract<K: OdraType, T: OdraType>(
    seed: &str,
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

fn call_contract_deploy(addr: Address, entrypoint: &str, args: &CallArgs, amount: Option<Balance>) {
    let gas = get_gas();
    CasperClient::new().deploy_entrypoint_call(addr, entrypoint, args, amount, gas);
}
