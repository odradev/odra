#![feature(box_patterns)]

use odra_schema::casper_contract_schema::ContractSchema;
use std::path::Path;

pub mod codegen;
mod schema;
mod types;

pub fn read_schema<P: AsRef<Path>>(schema_path: P) -> Result<ContractSchema, String> {
    let schema = std::fs::read_to_string(schema_path)
        .map_err(|e| format!("Failed to read schema file: {}", e))?;
    let result = serde_json::from_str::<serde_json::Value>(&schema)
        .map_err(|e| format!("Failed to convert schema to value: {}", e))?;

    schema::parse(&result)
}
