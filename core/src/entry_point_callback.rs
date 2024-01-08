use crate::call_def::CallDef;
use crate::{Bytes, OdraError};
use crate::{ContractEnv, HostEnv};

#[derive(Clone)]
pub struct EntryPointsCaller {
    pub f: fn(contract_env: ContractEnv, call_def: CallDef) -> Result<Bytes, OdraError>,
    host_env: HostEnv
}

impl EntryPointsCaller {
    pub fn new(
        host_env: HostEnv,
        f: fn(contract_env: ContractEnv, call_def: CallDef) -> Result<Bytes, OdraError>
    ) -> Self {
        EntryPointsCaller { f, host_env }
    }

    pub fn call(&self, call_def: CallDef) -> Result<Bytes, OdraError> {
        (self.f)(self.host_env.contract_env(), call_def)
    }
}
