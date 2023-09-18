use crate::call_def::CallDef;
use crate::contract_env::{ContractContext, ContractEnv};
use crate::odra_result::OdraResult;

pub trait Callable {
    fn call(&self, env: ContractEnv, call_def: CallDef) -> OdraResult<Vec<u8>>;
}

pub struct ModuleCaller(pub fn(env: ContractEnv, call_def: CallDef) -> OdraResult<Vec<u8>>);

impl ModuleCaller {
    pub fn new(f: fn(env: ContractEnv, call_def: CallDef) -> OdraResult<Vec<u8>>) -> Self {
        Self(f)
    }
    pub fn call_module(&self, env: ContractEnv, call_def: CallDef) -> OdraResult<Vec<u8>> {
        (self.0)(env, call_def)
    }
}
