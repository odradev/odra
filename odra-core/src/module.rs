use core::cell::OnceCell;
use core::ops::{Deref, DerefMut};

use crate::call_def::CallDef;
use crate::contract_env::ContractEnv;
use crate::odra_result::OdraResult;
use crate::prelude::*;

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
    fn env(&self) -> &ContractEnv;
}

pub struct ModuleWrapper<const N: u8, T> {
    env: Rc<ContractEnv>,
    module: OnceCell<T>
}

impl<const N: u8, T: Module> ModuleWrapper<N, T> {
    pub fn new(env: Rc<ContractEnv>) -> Self {
        Self {
            env,
            module: OnceCell::new()
        }
    }

    pub fn new_module(env: &ContractEnv) -> T {
        T::new(Rc::new(env.child(N)))
    }

    pub fn module(&self) -> &T {
        self.module.get_or_init(|| Self::new_module(&self.env))
    }

    pub fn module_mut(&mut self) -> &mut T {
        if self.module.get().is_none() {
            let _ = self.module();
        }
        self.module.get_mut().unwrap()
    }
}

impl<const N: u8, T: Module> Deref for ModuleWrapper<N, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.module()
    }
}

impl<const N: u8, T: Module> DerefMut for ModuleWrapper<N, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.module_mut()
    }
}
