use std::collections::HashMap;

use casper_types::{bytesrepr::FromBytes, CLValue};
use odra_casper_types::{Address, Balance, CallArgs, OdraType};
use ref_thread_local::RefThreadLocal;

use crate::{EntrypointArgs, EntrypointCall};

use self::{
    callstack::Callstack, contract_container::ContractContainer,
    contract_register::ContractRegister
};

mod callstack;
mod casper_client;
mod contract_container;
mod contract_register;

#[derive(Default)]
struct ClientEnv {
    contracts: ContractRegister,
    callstack: Callstack
}

impl ClientEnv {
    pub fn instance<'a>() -> ref_thread_local::Ref<'a, ClientEnv> {
        ENV.borrow()
    }

    pub fn instance_mut<'a>() -> ref_thread_local::RefMut<'a, ClientEnv> {
        ENV.borrow_mut()
    }

    pub fn register_contract(&mut self, contract: ContractContainer) {
        self.contracts.add(contract);
    }

    pub fn push_on_stack(&mut self, addr: Address) {
        self.callstack.push(addr);
    }

    pub fn pull_from_stack(&mut self) -> Option<Address> {
        self.callstack.pop()
    }

    pub fn call_contract<T: OdraType>(&self, addr: Address, entrypoint: &str, args: CallArgs) -> T {
        let result = self.contracts.call(&addr, String::from(entrypoint), args);
        let bytes = result.unwrap();
        let (clvalue, _) = CLValue::from_bytes(&bytes).unwrap();
        clvalue.into_t().unwrap()
    }

    pub fn current_contract(&self) -> Address {
        self.callstack.current()
    }
}

ref_thread_local::ref_thread_local!(
    static managed ENV: ClientEnv = ClientEnv::default();
);

/// Deploy WASM file with arguments.
pub fn register_contract(
    address: Address,
    entrypoints: HashMap<String, (EntrypointArgs, EntrypointCall)>
) {
    let contract = ContractContainer::new(address, entrypoints);
    ClientEnv::instance_mut().register_contract(contract);
}

pub fn call_contract<T: OdraType>(
    addr: Address,
    entrypoint: &str,
    args: CallArgs,
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

pub fn get_var_from_current_contract<T: OdraType>(key: &str) -> Option<T> {
    let address = ClientEnv::instance().current_contract();
    casper_client::CasperClient::intergration_testnet().get_var(address, key)
}
