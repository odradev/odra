use odra_core::prelude::rc::Rc;
use odra::{ContractEnv, prelude::Module, ModuleWrapper};

pub struct Ownable {
    env: Rc<ContractEnv>,
    owner: Variable<1, u32>,
}

impl Module for Ownable {
    fn new(env: Rc<ContractEnv>) -> Self {
        let owner = Variable::new(Rc::clone(&env));
        Ownable { env, owner }
    }

    fn env(&self) -> &ContractEnv {
        &self.env
    }
}

pub struct OwnableWrapper {
    env: Rc<ContractEnv>,
    ownable: ModuleWrapper<1, Ownable>,
}

impl Module for OwnableWrapper {
    fn new(env: Rc<ContractEnv>) -> Self {
        let ownable = ModuleWrapper::new(Rc::clone(&env));
        OwnableWrapper { env, ownable }
    }

    fn env(&self) -> &ContractEnv {
        &self.env
    }
}
