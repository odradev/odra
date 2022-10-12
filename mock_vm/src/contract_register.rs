use crate::account::Account;
use crate::contract_container::ContractContainer;
use odra_types::{bytesrepr::Bytes, RuntimeArgs};
use odra_types::{Address, OdraError, VmError};
use std::collections::{hash_map::IterMut, HashMap};

#[derive(Default)]
pub struct ContractRegister {
    contracts: HashMap<Address, ContractContainer>,
}

impl ContractRegister {
    pub fn add(&mut self, addr: Address, container: ContractContainer) {
        self.contracts.insert(addr, container);
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

#[derive(Default)]
pub struct ContractAccounts {
    accounts: HashMap<Address, Account>,
}

impl ContractAccounts {
    pub fn add(&mut self, addr: Address) {
        let contract_account = Account::zero_balance(addr);
        self.accounts.insert(addr, contract_account);
    }

    pub fn get_contract_accounts(&mut self) -> IterMut<'_, Address, Account> {
        self.accounts.iter_mut()
    }

    pub fn get_contract_account(&self, addr: Address) -> Option<&Account> {
        self.accounts.get(&addr)
    }
}
