use odra_core::prelude::*;
use schemars::JsonSchema;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[wasm_bindgen]
pub struct Contracts {
    schemas: Vec<Value>,
    wasm_bytes: BTreeMap<String, Vec<u8>>
}

impl Contracts {
    pub fn new(schemas: Vec<Value>, wasm_bytes: BTreeMap<String, Vec<u8>>) -> Self {
        Self {
            schemas,
            wasm_bytes
        }
    }

    pub fn load(schemas_str: &str, wasm_bytes: BTreeMap<String, Vec<u8>>) -> Self {
        let schemas = Self::load_schemas(schemas_str).unwrap();
        Self {
            schemas,
            wasm_bytes
        }
    }

    pub fn load_schemas(schemas_str: &str) -> Result<Vec<Value>, String> {
        let schemas: Vec<Value> = match serde_json::from_str(schemas_str) {
            Ok(schemas) => schemas,
            Err(_) => return Err("Error parsing contract schemas".to_string())
        };
        Ok(schemas)
    }

    pub fn assert_contract_exists_in_schema(
        contract_name: &str,
        schemas: Vec<Value>
    ) -> Result<(), String> {
        match schemas.iter().find(|s| s["contract_name"] == contract_name) {
            None => Err(format!(
                "Could not find a {contract_name} contract in schemas."
            )),
            Some(_) => Ok(())
        }
    }
}
