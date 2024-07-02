//! The `world` module contains the `OdraWorld` struct, which holds the state of the world for the tests.

use core::panic;
use odra_core::{
    casper_types::{account, U256},
    contract_def::HasIdent,
    host::{HostEnv, HostRef},
    Address, Addressable
};
use std::{any::Any, collections::HashMap};

use super::{param::Account, steps::Cep18TokenHostRef};

/// OdraWorld is a struct that holds the state of the world for the tests.
pub struct OdraWorld {
    env: HostEnv,
    container: ContractsContainer,
    registered_users: HashMap<String, Address>
}

impl Default for OdraWorld {
    fn default() -> Self {
        Self {
            env: crate::env(),
            container: ContractsContainer::default(),
            registered_users: HashMap::new()
        }
    }
}

impl OdraWorld {
    /// Returns a reference to the HostEnv.
    pub fn env(&self) -> &HostEnv {
        &self.env
    }

    /// Registers a contract in the world.
    pub fn add_contract<T: HasIdent, F: FnOnce(&HostEnv) -> Box<dyn Addressable>>(
        &mut self,
        contract: F
    ) {
        let addressable = contract(&self.env);

        let address = addressable.address().clone();
        let contract = addressable.as_boxed_any();
        self.container.add::<T>(contract, address);
    }

    /// Returns a mutable reference to a contract.
    pub fn get_contract<T: HostRef + 'static, I: HasIdent>(&mut self) -> &mut T {
        self.container.get(&I::ident())
    }

    /// Returns a mutable reference to the Cep18TokenHostRef contract.
    pub fn cep18<I: HasIdent>(&mut self) -> &mut Cep18TokenHostRef {
        self.get_contract::<Cep18TokenHostRef, I>()
    }

    pub fn cep18_balance_of<I: HasIdent>(&mut self, account: Account) -> U256 {
        let contract = match account {
            Account::Contract(name) => self.get_contract::<Cep18TokenHostRef, I>(),
            _ => panic!("Only contract accounts can be used for this operation")
        };
        contract.balance_of(&self.get_address(account))
    }

    /// Returns the address of an account.
    pub fn get_address(&mut self, account: Account) -> Address {
        match account {
            Account::Alice => self.env.get_account(1),
            Account::Bob => self.env.get_account(2),
            Account::Charlie => self.env.get_account(3),
            Account::Dan => self.env.get_account(4),
            Account::Eve => self.env.get_account(5),
            Account::Fred => self.env.get_account(6),
            Account::George => self.env.get_account(7),
            Account::Harry => self.env.get_account(8),
            Account::Ian => self.env.get_account(9),
            Account::John => self.env.get_account(10),
            Account::Contract(name) => self.container.get_address(&name),
            Account::CustomRole(name) => {
                if let Some(address) = self.registered_users.get(&name) {
                    *address
                } else {
                    let idx = 19 - self.registered_users.len();
                    if idx < 11 {
                        panic!("Cannot register more custom roles");
                    }
                    let address = self.env.get_account(idx);
                    self.registered_users.insert(name, address);
                    address
                }
            }
        }
    }

    /// Sets the caller of the HostEnv.
    pub fn set_caller(&mut self, account: Account) {
        let address = self.get_address(account);
        self.env.set_caller(address);
    }

    /// Advances the block time of the HostEnv.
    pub fn advance_block_time(&mut self, seconds: u64) {
        self.env.advance_block_time(seconds);
    }
}

#[derive(Default)]
struct ContractsContainer {
    contracts: HashMap<String, Box<dyn Any>>,
    addresses: HashMap<String, Address>
}

impl ContractsContainer {
    fn add<T: HasIdent>(&mut self, contract: Box<dyn Any>, address: Address) {
        let name = format!("{}Contract", T::ident());
        self.contracts.insert(name.clone(), contract);
        self.addresses.insert(name, address);
    }

    fn get<T: HostRef + 'static>(&mut self, name: &str) -> &mut T {
        let name = format!("{}Contract", name);
        let t = &mut **self.contracts.get_mut(&name).unwrap();
        t.downcast_mut::<T>().unwrap()
    }

    fn get_address(&self, name: &String) -> Address {
        self.addresses.get(name).unwrap().clone()
    }
}
