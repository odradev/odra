use casper_types::CLType;

use crate::call_def::CallDef;
use crate::prelude::*;
use crate::{Bytes, OdraError};
use crate::{ContractEnv, HostEnv};

#[derive(Clone)]
pub struct EntryPointsCaller {
    f: fn(contract_env: ContractEnv, call_def: CallDef) -> Result<Bytes, OdraError>,
    host_env: HostEnv,
    entry_points: Vec<EntryPoint>
}

impl EntryPointsCaller {
    pub fn new(
        host_env: HostEnv,
        entry_points: Vec<EntryPoint>,
        f: fn(contract_env: ContractEnv, call_def: CallDef) -> Result<Bytes, OdraError>
    ) -> Self {
        EntryPointsCaller {
            f,
            host_env,
            entry_points
        }
    }

    pub fn call(&self, call_def: CallDef) -> Result<Bytes, OdraError> {
        (self.f)(self.host_env.contract_env(), call_def)
    }

    pub fn entry_points(&self) -> &[EntryPoint] {
        self.entry_points.as_ref()
    }
}

#[derive(Clone)]
pub struct EntryPoint {
    pub name: String,
    pub args: Vec<EntryPointArgument>
}

impl EntryPoint {
    pub fn new(name: String, args: Vec<EntryPointArgument>) -> Self {
        Self { name, args }
    }
}

#[derive(Clone)]
pub struct EntryPointArgument {
    pub name: String,
    pub ty: CLType
}

impl EntryPointArgument {
    pub fn new(name: String, ty: CLType) -> Self {
        Self { name, ty }
    }
}
