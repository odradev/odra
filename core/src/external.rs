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
///
/// # Example
///
/// use core::ops::DerefMut;
/// use odra::{Address, External, prelude::*};
///
/// #[odra::module]
/// pub struct Contract {
///     ext: odra::External<SetterContractRef>
/// }
///
/// #[odra::module]
/// impl Contract {
///     pub fn init(&mut self, address: odra::Address) {
///         self.ext.set(address);
///     }
///
///     /// If a contract implements the set() method, you can't use
///     /// the deref coercion mechanism, so you call the `External::set()` method.
///     /// In this case you need to dereference the contract explicitly:
///     pub fn set(&mut self, value: bool) {
///         self.ext.deref_mut().set(value);
///         // or
///         // DerefMut::deref_mut(&mut self.ext).set(value);
///     }
///
///     /// For any other method, you can use the deref coercion mechanism, and call
///     /// the method directly on the external instance:
///     pub fn get(&self) -> bool {
///         self.ext.get()
///     }
/// }
///
/// #[odra::external_contract]
/// pub trait SetterGetter {
///     fn set(&mut self, value: bool);
///     fn get(&self) -> bool;
/// }
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
