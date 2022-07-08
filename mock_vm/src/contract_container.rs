use odra_types::{bytesrepr::Bytes, RuntimeArgs, OdraError, VmError};
use std::collections::HashMap;

pub type EntrypointCall = fn(String, RuntimeArgs) -> Option<Bytes>;

#[derive(Default, Clone)]
pub struct ContractContainer {
    name: String,
    entrypoints: HashMap<String, EntrypointCall>,
}

impl ContractContainer {
    pub fn new(
        name: &str, 
        entrypoints: HashMap<String, EntrypointCall>,
    ) -> Self {
        Self {
            name: String::from(name),
            entrypoints,
        }
    }

    pub fn add(&mut self, entrypoint: String, f: EntrypointCall) {
        self.entrypoints.insert(entrypoint, f);
    }

    pub fn remove(&mut self, entrypoint: String) {
        self.entrypoints.remove(&entrypoint);
    }

    pub fn call(&self, entrypoint: String, args: RuntimeArgs) -> Result<Option<Bytes>, OdraError> {
        match self.entrypoints.get(&entrypoint) {
            Some(f) => Ok(f(self.name.clone(), args)),
            None => Err(OdraError::VmError(VmError::NoSuchMethod(entrypoint))),
        }
    }
}
