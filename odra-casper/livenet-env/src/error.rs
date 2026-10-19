//! Module for handling Odra errors coming out of the Livenet execution.
use std::{fs, path::PathBuf};

use anyhow::{anyhow, Result};
use odra_core::prelude::*;
use serde_json::Value;

/// Finds the error message in the contract schema.
///
/// A user error is looked up in the schema of `contract_name` first, the contract that was called.
/// Contracts reuse small codes, so the first schema with the code could name another contract's
/// error. The code alone does not tell which contract reverted, so if the called contract has no
/// such error (a contract it called reverted), the schemas of all the contracts are searched.
pub fn find(error_msg: &str, contract_name: Option<&str>) -> Result<OdraError> {
    if error_msg == "Out of gas error" {
        return Ok(ExecutionError::OutOfGas.into());
    }

    let error_num: u16 = error_msg
        .strip_prefix("User error: ")
        .ok_or_else(|| anyhow!("Couldn't parse error message: {:?}", error_msg))?
        .parse()?;

    if is_internal_error(error_num) {
        return ExecutionError::from_code(error_num)
            .map(Into::into)
            .ok_or_else(|| anyhow!("Unknown Odra error code: {}", error_num));
    }

    let schemas = schema_files()?;
    let named = contract_name.and_then(|name| {
        schemas
            .iter()
            .filter(|schema| schema["contract_name"].as_str() == Some(name))
            .find_map(|schema| find_error_in_schema(schema, error_num))
    });
    named
        .or_else(|| {
            schemas
                .iter()
                .find_map(|schema| find_error_in_schema(schema, error_num))
        })
        .ok_or_else(|| anyhow!("Couldn't find error in the contract schema: {}", error_msg))
}

/// The contract schemas in the schema directory of the current directory and of its parents, up
/// to the project root, nearest first.
fn schema_files() -> Result<Vec<Value>> {
    let root =
        project_root::get_project_root().map_err(|_| anyhow!("Couldn't get project root"))?;
    let mut current_dir =
        std::env::current_dir().map_err(|_| anyhow!("Couldn't get current directory"))?;
    #[cfg(test)]
    let schema_path = std::path::PathBuf::from("resources/test");
    #[cfg(not(test))]
    let schema_path = std::path::PathBuf::from("resources/casper_contract_schemas");

    let mut schemas = read_schemas(current_dir.join(&schema_path));
    while current_dir != root {
        current_dir = current_dir
            .parent()
            .ok_or_else(|| anyhow!("Couldn't get parent directory"))?
            .to_path_buf();
        schemas.extend(read_schemas(current_dir.join(&schema_path)));
    }
    Ok(schemas)
}

fn read_schemas(path: PathBuf) -> Vec<Value> {
    let mut paths = odra_schema::find_schemas_file_paths(path).unwrap_or_default();
    // The directory order depends on the file system.
    paths.sort();
    paths
        .into_iter()
        .filter_map(|path| fs::read_to_string(path).ok())
        .filter_map(|schema| serde_json::from_str(&schema).ok())
        .collect()
}

fn find_error_in_schema(schema: &Value, error_num: u16) -> Option<OdraError> {
    schema["errors"]
        .as_array()?
        .iter()
        .find_map(|err| match_error(err, error_num))
}

fn match_error(val: &Value, error_num: u16) -> Option<OdraError> {
    if val["discriminant"].as_u64() == Some(error_num as u64) {
        val["name"]
            .as_str()
            .map(|msg| OdraError::user(error_num, msg))
    } else {
        None
    }
}

#[inline]
fn is_internal_error(error_num: u16) -> bool {
    error_num >= ExecutionError::UserErrorTooHigh.code()
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_reading_errors() {
        // Contract errors
        assert_eq!(
            call("User error: 60017").ok(),
            Some(OdraError::user(60017, "CannotTargetSelfUser"))
        );
        assert_eq!(
            call("User error: 60010").ok(),
            Some(OdraError::user(60010, "InsufficientRights"))
        );
        // Odra error
        assert_eq!(
            call("User error: 64537").ok(),
            Some(ExecutionError::UnwrapError.into())
        );
        assert_eq!(
            call("User error: 64659").ok(),
            Some(ExecutionError::MissingAddress.into())
        );
        // Unknown user error
        assert!(call("User error: 60300").is_err());
        // Other errors
        assert!(call("Casper Engine error").is_err());
        assert_eq!(
            call("Out of gas error").ok(),
            Some(ExecutionError::OutOfGas.into())
        );
    }

    #[test]
    fn user_error_is_named_by_the_called_contract() {
        assert_eq!(
            find("User error: 1", Some("Second")).ok(),
            Some(OdraError::user(1, "SecondError"))
        );
        assert_eq!(
            find("User error: 1", Some("First")).ok(),
            Some(OdraError::user(1, "FirstError"))
        );
    }

    #[test]
    fn user_error_of_another_contract_is_found_in_any_schema() {
        // Second called First, which reverted.
        assert_eq!(
            find("User error: 2", Some("Second")).ok(),
            Some(OdraError::user(2, "OnlyFirstError"))
        );
        assert_eq!(
            find("User error: 1", Some("Unknown")).ok(),
            Some(OdraError::user(1, "FirstError"))
        );
    }

    #[test]
    fn unknown_odra_error_is_an_error() {
        assert!(call("User error: 65000").is_err());
        assert!(call("User error: 65535").is_err());
    }

    fn call(error_msg: &str) -> Result<OdraError> {
        super::find(error_msg, None)
    }
}
