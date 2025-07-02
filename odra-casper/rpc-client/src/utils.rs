use casper_types::bytesrepr::FromBytes;
use casper_types::StoredValue::CLValue;
use casper_types::{CLTyped, StoredValue};
use odra_core::prelude::{ExecutionError, OdraError, OdraResult};
use serde_json::Value;
use std::path::PathBuf;

/// Search for the wasm file in the current directory and in the parent directory.
pub fn find_wasm_file_path(wasm_file_name: &str) -> OdraResult<PathBuf> {
    let contract_path = PathBuf::from("wasm")
        .join(wasm_file_name)
        .with_extension("wasm");

    let mut base_path = project_root::get_project_root()
        .map_err(|_| OdraError::ExecutionError(ExecutionError::ContractDeploymentError))?;
    let mut checked_paths = vec![];
    for _ in 0..2 {
        let path = base_path.join(&contract_path);
        if path.exists() {
            crate::log::info(format!("Found wasm under {:?}.", path));
            return Ok(path);
        } else {
            checked_paths.push(path);
            base_path = base_path
                .parent()
                .ok_or(OdraError::ExecutionError(
                    ExecutionError::ContractDeploymentError
                ))?
                .to_path_buf();
        }
    }
    crate::log::error(format!("Could not find wasm under {:?}.", checked_paths));
    Err(OdraError::ExecutionError(
        ExecutionError::ContractDeploymentError
    ))
}

/// Gets an env variable
pub fn get_env_variable(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|err| {
        crate::log::error(format!(
            "{} must be set. Have you setup your .env file?",
            name
        ));
        panic!("{}", err)
    })
}

/// Gets an optional env variable
pub fn get_optional_env_variable(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

/// Converts RuntimeArgs into Vec<String> compatible with rustSDK
pub fn runtime_args_to_simple_args(runtime_args: &casper_types::RuntimeArgs) -> Vec<String> {
    runtime_args
        .named_args()
        .map(|named_arg| {
            let value = serde_json::to_string(&named_arg.cl_value()).unwrap();
            let json: Value = serde_json::from_str(&value).unwrap();
            let value = json.get("parsed").unwrap().to_string();
            format!(
                "{}:{}='{}'",
                named_arg.name(),
                named_arg.cl_value().cl_type(),
                value,
            )
        })
        .collect()
}

pub fn extract_stored_value<T: CLTyped + FromBytes>(value: StoredValue) -> T {
    match value {
        CLValue(value) => value
            .clone()
            .into_t()
            .unwrap_or_else(|_| panic!("Couldn't get bytes from CLValue: {:?}", value)),
        _ => panic!("Value stored in result key is not a CLValue")
    }
}
