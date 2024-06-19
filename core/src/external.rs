use core::{
    cell::OnceCell,
    ops::{Deref, DerefMut}
};

use alloc::rc::Rc;

use crate::{
    module::{ModuleComponent, ModulePrimitive},
    Address, ContractEnv, ContractRef, ExecutionError, Var
};

/// A module component that is a reference to an external contract.
pub struct External<T: ContractRef> {
    env: Rc<ContractEnv>,
    value: Var<Address>,
    contract_ref: OnceCell<T>
}

impl<T: ContractRef> ModuleComponent for External<T> {
    /// Creates a new instance of `External` with the given environment and index.
    fn instance(env: Rc<ContractEnv>, index: u8) -> Self {
        Self {
            env: env.clone(),
            value: Var::instance(env, index),
            contract_ref: OnceCell::new()
        }
    }
}

impl<T: ContractRef> ModulePrimitive for External<T> {}

impl<T: ContractRef> External<T> {
    /// Sets the address of the external contract.
    pub fn set(&mut self, address: Address) {
        if self.value.get().is_some() {
            self.env.revert(ExecutionError::AddressAlreadySet);
        }
        self.value.set(address);
    }

    fn contract_ref_mut(&mut self) -> &mut T {
        if self.contract_ref.get().is_none() {
            let _ = self.contract_ref();
        }
        self.contract_ref.get_mut().unwrap()
    }

    fn contract_ref(&self) -> &T {
        self.contract_ref.get_or_init(|| {
            let address = self
                .value
                .get_or_revert_with(ExecutionError::MissingAddress);
            T::new(self.env.clone(), address)
        })
    }
}

impl<T: ContractRef> Deref for External<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.contract_ref()
    }
}

impl<T: ContractRef> DerefMut for External<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.contract_ref_mut()
    }
}
