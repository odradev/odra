use crate::{contract_def::HasEvents, prelude::*};
use core::cell::OnceCell;

use crate::call_def::CallDef;
use crate::contract_env::ContractEnv;
use crate::odra_result::OdraResult;
use core::ops::{Deref, DerefMut};

pub trait Callable {
    fn call(&self, env: ContractEnv, call_def: CallDef) -> OdraResult<Vec<u8>>;
}

#[derive(Clone)]
pub struct ModuleCaller(pub fn(env: ContractEnv, call_def: CallDef) -> OdraResult<Vec<u8>>);

impl ModuleCaller {
    pub fn new(f: fn(env: ContractEnv, call_def: CallDef) -> OdraResult<Vec<u8>>) -> Self {
        Self(f)
    }
    pub fn call_module(&self, env: ContractEnv, call_def: CallDef) -> OdraResult<Vec<u8>> {
        (self.0)(env, call_def)
    }
}

pub trait Module {
    fn new(env: Rc<ContractEnv>) -> Self;
    fn env(&self) -> Rc<ContractEnv>;
}

pub trait ModuleComponent {
    fn instance(env: Rc<ContractEnv>, index: u8) -> Self;
}

/// A marker trait for a module component that does not emit events.
///
/// This trait allows to implement `HasEvents` for components like Variable, List, Mapping,
/// or any other custom component that does not emit events.
pub trait ModulePrimitive: ModuleComponent {}

pub struct ModuleWrapper<T> {
    env: Rc<ContractEnv>,
    module: OnceCell<T>,
    index: u8
}

impl<T: Module> ModuleComponent for ModuleWrapper<T> {
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

impl<M: HasEvents> HasEvents for ModuleWrapper<M> {
    fn events() -> Vec<crate::contract_def::Event> {
        M::events()
    }
}

impl<T: Module> ModuleWrapper<T> {
    pub fn module(&self) -> &T {
        self.module
            .get_or_init(|| T::new(Rc::new(self.env.child(self.index))))
    }

    pub fn module_mut(&mut self) -> &mut T {
        if self.module.get().is_none() {
            let _ = self.module();
        }
        self.module.get_mut().unwrap()
    }
}

impl<T: Module> Deref for ModuleWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.module()
    }
}

impl<T: Module> DerefMut for ModuleWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.module_mut()
    }
}
