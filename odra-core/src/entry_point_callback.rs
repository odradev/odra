use crate::{ContractEnv, HostEnv};
use odra_types::call_def::CallDef;
use odra_types::Bytes;

#[derive(Clone)]
pub struct EntryPointsCaller {
    pub f: fn(contract_env: ContractEnv, call_def: CallDef) -> Bytes,
    host_env: HostEnv
}

impl EntryPointsCaller {
    pub fn new(
        host_env: HostEnv,
        f: fn(contract_env: ContractEnv, call_def: CallDef) -> Bytes
    ) -> Self {
        EntryPointsCaller { f, host_env }
    }

    pub fn call(&self, call_def: CallDef) -> Bytes {
        (self.f)(self.host_env.contract_env(), call_def)
    }
}
