use odra_types::{bytesrepr::Bytes, RuntimeArgs};
use odra_types::{Address, OdraError, VmError};
use std::collections::HashMap;

use crate::account::Account;
use crate::contract_container::ContractContainer;

#[derive(Default)]
pub struct ContractRegister {
    contracts: HashMap<Address, ContractContainer>,
    accounts: HashMap<Address, Account>,
}

impl ContractRegister {
    pub fn add(&mut self, addr: Address, container: ContractContainer) {
        let contract_account = Account::zero_balance(addr);
        self.contracts.insert(addr, container);
        self.accounts.insert(addr, contract_account);
    }

    pub fn call(
        &self,
        addr: &Address,
        entrypoint: String,
        args: RuntimeArgs,
    ) -> Result<Option<Bytes>, OdraError> {
        self.internal_call(addr, |container| {
            std::panic::catch_unwind(|| container.call(entrypoint, args))?
        })
    }

    pub fn call_constructor(
        &self,
        addr: &Address,
        entrypoint: String,
        args: RuntimeArgs,
    ) -> Result<Option<Bytes>, OdraError> {
        self.internal_call(addr, |container| {
            std::panic::catch_unwind(|| container.call_constructor(entrypoint, args))?
        })
    }

    pub fn get_contract_account_mut(&mut self, addr: Address) -> Option<&mut Account> {
        self.accounts.get_mut(&addr)
    }

    pub fn get_contract_accounts(&mut self) -> std::collections::hash_map::IterMut<'_, Address, Account> {
        self.accounts.iter_mut()
    }

    pub fn get_contract_account(&self, addr: Address) -> Option<&Account> {
        self.accounts.get(&addr)
    } 

    fn internal_call<F: FnOnce(&ContractContainer) -> Result<Option<Bytes>, OdraError>>(
        &self,
        addr: &Address,
        call_fn: F,
    ) -> Result<Option<Bytes>, OdraError> {
        let contract = self.contracts.get(addr);
        match contract {
            Some(container) => call_fn(container),
            None => Err(OdraError::VmError(VmError::InvalidContractAddress)),
        }
    }
}
