//! Module definition and implementation.
//!
//! An Odra module is a composition of [ModuleComponent]s (eg. other modules) and [ModulePrimitive]s
//! ([Var](crate::var::Var), [Mapping](crate::mapping::Mapping), [List](crate::list::List)).
//!
//! In order to create a module, you need to create a struct that implements the [Module] trait.
//! However, most of the time you will want to use `#[odra::module]` macro to generate the module.

use crate::{contract_def::HasEvents, prelude::*};
use core::cell::OnceCell;

use crate::contract_env::ContractEnv;
use core::ops::{Deref, DerefMut};

/// Represents a module in the Odra system.
pub trait Module {
    /// Creates a new instance of the module with the given contract environment.
    fn new(env: Rc<ContractEnv>) -> Self;

    /// Returns the [contract environment](ContractEnv) associated with the module.
    fn env(&self) -> Rc<ContractEnv>;
}

/// Represents a component that can be a part of a module.
pub trait ModuleComponent {
    /// Creates a new instance of the module component.
    fn instance(env: Rc<ContractEnv>, index: u8) -> Self;
}

/// A marker trait for a module component that does not emit events.
///
/// This trait allows to implement `HasEvents` for components like Var, List, Mapping,
/// or any other custom component that does not emit events.
pub trait ModulePrimitive: ModuleComponent {}

/// A wrapper struct for a module implementing the [Module] trait.
///
/// This struct is used to implement an Odra module that is a composition of other modules.
pub struct SubModule<T> {
    env: Rc<ContractEnv>,
    module: OnceCell<T>,
    index: u8
}

impl<T: Module> ModuleComponent for SubModule<T> {
    fn instance(env: Rc<ContractEnv>, index: u8) -> Self {
        Self {
            env,
            module: OnceCell::new(),
            index
        }
    }
}

impl<M: ModulePrimitive> HasEvents for M {
    fn events() -> Vec<crate::contract_def::Event> {
        Vec::new()
    }
}

impl<M: HasEvents> HasEvents for SubModule<M> {
    fn events() -> Vec<crate::contract_def::Event> {
        M::events()
    }
}

/// Wrapper for a module implementing the `Module` trait.
impl<T: Module> SubModule<T> {
    /// Returns a reference to the module.
    ///
    /// If the module is not yet initialized, it will be lazily initialized.
    pub fn module(&self) -> &T {
        self.module
            .get_or_init(|| T::new(Rc::new(self.env.child(self.index))))
    }

    /// Returns a mutable reference to the module.
    ///
    /// If the module is not yet initialized, it will be lazily initialized.
    pub fn module_mut(&mut self) -> &mut T {
        if self.module.get().is_none() {
            let _ = self.module();
        }
        self.module.get_mut().unwrap()
    }
}

impl<T: Module> Deref for SubModule<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.module()
    }
}

impl<T: Module> DerefMut for SubModule<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.module_mut()
    }
}
