//! Module definition and implementation.
//!
//! An Odra module is a composition of [ModuleComponent]s (eg. other modules) and [ModulePrimitive]s
//! ([Var], [Mapping], [List]).
//!
//! In order to create a module, you need to create a struct that implements the [Module] trait.
//! However, most of the time you will want to use `#[odra::module]` macro to generate the module.

use crate::{contract_def::HasEvents, prelude::*};
use core::cell::OnceCell;

use crate::contract_env::ContractEnv;
use core::ops::{Deref, DerefMut};

/// Represents a module in the Odra system.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not an Odra module",
    label = "not a module",
    note = "a module is a struct with `#[odra::module]` on it and on its impl block"
)]
pub trait Module {
    /// Creates a new instance of the module with the given contract environment.
    fn new(env: Rc<ContractEnv>) -> Self;

    /// Returns the [contract environment](ContractEnv) associated with the module.
    fn env(&self) -> Rc<ContractEnv>;
}

/// Implemented for every module by `#[odra::module]` on its impl block, so an impl block whose
/// struct has no `#[odra::module]` is reported first as "not an Odra module", before the errors
/// the rest of the generated code adds.
#[doc(hidden)]
pub trait IsModule: Module {}

/// Represents a component that can be a part of a module.
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be a field of a module",
    label = "not a storage component",
    note = "a module's field is a `Var<T>`, a `Mapping<K, V>`, a `List<T>` or a `SubModule<M>`: the module stores values, it does not hold them"
)]
pub trait ModuleComponent {
    /// Creates a new instance of the module component.
    fn instance(env: Rc<ContractEnv>, index: u8) -> Self;
}

/// Represents a component that can revert the contract execution.
pub trait Revertible {
    /// Reverts the contract execution with the given error.
    fn revert<E: Into<OdraError>>(&self, error: E) -> !;
}

impl Revertible for Rc<ContractEnv> {
    fn revert<E: Into<OdraError>>(&self, error: E) -> ! {
        self.as_ref().revert(error)
    }
}

impl<T: Module> Revertible for T {
    fn revert<E: Into<OdraError>>(&self, error: E) -> ! {
        self.env().revert(error)
    }
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

    fn event_schemas() -> crate::prelude::BTreeMap<String, casper_event_standard::Schema> {
        M::event_schemas()
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
        self.module
            .get_mut()
            .expect("initialized by module() above")
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
