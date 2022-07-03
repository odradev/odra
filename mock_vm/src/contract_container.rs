use odra_types::{bytesrepr::Bytes, RuntimeArgs};
use std::collections::HashMap;

pub type EntrypointCall = fn(String, RuntimeArgs) -> Option<Bytes>;

#[derive(Default, Clone)]
pub struct ContractContainer {
    name: String,
    entrypoints: HashMap<String, EntrypointCall>,
}

impl ContractContainer {
    pub fn new(name: &str, entrypoints: HashMap<String, EntrypointCall>) -> Self {
        Self {
            name: String::from(name),
            entrypoints,
        }
    }

    pub fn add(&mut self, entrypoint: String, f: EntrypointCall) {
        self.entrypoints.insert(entrypoint, f);
    }

    pub fn call(&self, entrypoint: String, args: RuntimeArgs) -> Option<Bytes> {
        let f = self.entrypoints.get(&entrypoint).unwrap();
        f(self.name.clone(), args)
    }
}
