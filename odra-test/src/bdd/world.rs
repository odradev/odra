//! The `world` module contains the `OdraWorld` struct, which holds the state of the world for the tests.

use core::panic;
use odra_core::{
    contract_def::HasIdent,
    host::{HostEnv, HostRef},
    Address, Addressable
};
use std::{any::Any, collections::HashMap, fmt::Debug};

use super::{
    param::Account,
    refs::Cep18TokenHostRef
};

#[derive(cucumber::World)]
/// OdraWorld is a struct that holds the state of the world for the tests.
pub struct OdraWorld {
    env: HostEnv,
    container: ContractsContainer,
    registered_users: HashMap<String, Address>,
    state: HashMap<String, Box<dyn Any>>
}

impl Debug for OdraWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OdraWorld")
            .field("container", &self.container)
            .field("registered_users", &self.registered_users)
            .field("state", &self.state)
            .finish()
    }
}

impl Default for OdraWorld {
    fn default() -> Self {
        Self {
            env: crate::env(),
            container: ContractsContainer::default(),
            registered_users: HashMap::new(),
            state: HashMap::new()
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
    pub fn get_contract<T: HostRef + HasIdent + 'static>(&mut self) -> &mut T {
        self.container.get(&T::ident())
    }

    /// Returns a mutable reference to the Cep18TokenHostRef contract.
    pub fn cep18<I: HasIdent>(&mut self) -> Cep18TokenHostRef {
        let address = self.get_contract_address::<I>();
        Cep18TokenHostRef::new(address, self)
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

    /// Returns the address of a contract.
    pub fn get_contract_address<I: HasIdent>(&self) -> Address {
        let name = format!("{}Contract", I::ident());
        self.container.get_address(&name)
    }

    /// Sets the caller of the HostEnv.
    pub fn set_caller(&mut self, account: Account) {
        let address = self.get_address(account);
        self.env.set_caller(address);
    }

    /// Sets the caller of the HostEnv and returns a mutable reference to the world.
    pub fn with_caller(&mut self, account: Account) -> &mut Self {
        let address = self.get_address(account);
        self.env.set_caller(address);
        self
    }

    /// Advances the block time of the HostEnv.
    pub fn advance_block_time(&mut self, seconds: u64) {
        self.env.advance_block_time(seconds);
    }

    /// Sets the state of the world.
    pub fn set_state<T: 'static>(&mut self, key: String, state: T) {
        self.state.insert(key, Box::new(state));
    }

    /// Returns the state of the world.
    pub fn get_state<T: 'static>(&self, key: &str) -> &T {
        self.state.get(key).unwrap().downcast_ref::<T>().unwrap()
    }
}

#[derive(Default, Debug)]
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
