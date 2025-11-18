use casper_types::bytesrepr::FromBytes;
use casper_types::StoredValue::CLValue;
use casper_types::{CLTyped, StoredValue};
use odra_core::prelude::{ExecutionError, OdraError, OdraResult};
use std::path::{self, PathBuf};

/// Search for the wasm file in the current directory and in the parent directory.
pub fn find_wasm_file_path(wasm_file_name: &str) -> OdraResult<PathBuf> {
    let contract_path = PathBuf::from("wasm")
        .join(wasm_file_name)
        .with_extension("wasm");

    let project_root = project_root::get_project_root()
        .map_err(|_| OdraError::ExecutionError(ExecutionError::ContractDeploymentError))?;
    let mut current_dir = path::absolute(".")
        .map_err(|_| OdraError::ExecutionError(ExecutionError::ContractDeploymentError))?;

    let mut checked_paths = vec![];
    while current_dir != project_root {
        let path = current_dir.join(&contract_path);
        if path.exists() {
            crate::log::info(format!("Found wasm under {:?}.", path));
            return Ok(path);
        } else {
            checked_paths.push(path);
            current_dir = current_dir
                .parent()
                .ok_or(OdraError::ExecutionError(
                    ExecutionError::ContractDeploymentError
                ))?
                .to_path_buf();
        }
    }
    let path = current_dir.join(&contract_path);
    checked_paths.push(path.clone());
    if path.exists() {
        crate::log::info(format!("Found wasm under {:?}.", path));
        return Ok(path);
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

pub fn extract_stored_value<T: CLTyped + FromBytes>(value: StoredValue) -> T {
    match value {
        CLValue(value) => value
            .clone()
            .into_t()
            .unwrap_or_else(|_| panic!("Couldn't get bytes from CLValue: {:?}", value)),
        _ => panic!("Value stored in result key is not a CLValue")
    }
}
