use std::collections::HashMap;

use odra_types::{bytesrepr::Bytes, RuntimeArgs};

#[derive(Default, Clone)]
pub struct ContractContainer {
    pub name: String,
    pub wasm_path: String,
    pub entrypoints: HashMap<String, fn(String, RuntimeArgs) -> Bytes>,
}

impl ContractContainer {
    pub fn add(&mut self, entrypoint: String, f: fn(String, RuntimeArgs) -> Bytes) {
        self.entrypoints.insert(entrypoint, f);
    }

    pub fn call(&self, entrypoint: String, args: RuntimeArgs) -> Bytes {
        let f = self.entrypoints.get(&entrypoint).unwrap();
        f(self.name.clone(), args)
    }
}
