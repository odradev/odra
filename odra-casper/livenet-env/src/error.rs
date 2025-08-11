//! Module for handling Odra errors coming out of the Livenet execution.
use std::{fs, path::PathBuf};

use anyhow::{anyhow, Result};
use odra_core::prelude::*;
use serde_json::Value;

/// Finds the error message in the contract schema.
pub fn find(error_msg: &str) -> Result<OdraError> {
    if error_msg == "Out of gas error" {
        return Ok(ExecutionError::OutOfGas.into());
    }

    let error_num: u16 = error_msg
        .strip_prefix("User error: ")
        .ok_or_else(|| anyhow!("Couldn't parse error message: {:?}", error_msg))?
        .parse()?;

    if is_internal_error(error_num) {
        return Ok(get_internal_error_name(error_num));
    }

    let root =
        project_root::get_project_root().map_err(|_| anyhow!("Couldn't get project root"))?;
    let mut current_dir =
        std::env::current_dir().map_err(|_| anyhow!("Couldn't get current directory"))?;
    #[cfg(test)]
    let schema_path = std::path::PathBuf::from("resources/test");
    #[cfg(not(test))]
    let schema_path = std::path::PathBuf::from("resources/casper_contract_schemas");

    while current_dir != root {
        match find_error_in_path(current_dir.join(&schema_path), error_num) {
            Some(odra_error) => return Ok(odra_error),
            None => {
                current_dir = current_dir
                    .parent()
                    .ok_or_else(|| anyhow!("Couldn't get parent directory"))?
                    .to_path_buf()
            }
        }
    }
    match find_error_in_path(current_dir.join(&schema_path), error_num) {
        Some(odra_error) => Ok(odra_error),
        None => Err(anyhow!(
            "Couldn't find error in the contract schema: {}",
            error_msg
        ))
    }
}

fn find_error_in_path(path: PathBuf, error_num: u16) -> Option<OdraError> {
    let schema_path = odra_schema::find_schemas_file_paths(path).ok()?;
    for schema_path in schema_path {
        let schema = fs::read_to_string(schema_path).ok()?;

        let schema: Value = serde_json::from_str(&schema).ok()?;
        let errors = schema["errors"].as_array()?;
        let f = errors.iter().find_map(|err| match_error(err, error_num));
        if let Some(odra_error) = f {
            return Some(odra_error);
        }
    }
    None
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

macro_rules! match_error {
    ($err:expr) => {
        $err.into()
    };
}

macro_rules! match_errors {
    ( $num:expr, $($err:expr),* ) => {
        match $num {
            $(
                x if x == $err.code() => match_error!($err),
            )*
            _ => panic!("Unknown execution error code: {}", $num)
        }
    };
}

fn get_internal_error_name(error_num: u16) -> OdraError {
    match_errors!(
        error_num,
        ExecutionError::UnwrapError,
        ExecutionError::AdditionOverflow,
        ExecutionError::SubtractionOverflow,
        ExecutionError::NonPayable,
        ExecutionError::TransferToContract,
        ExecutionError::ReentrantCall,
        ExecutionError::CannotOverrideKeys,
        ExecutionError::UnknownConstructor,
        ExecutionError::NativeTransferError,
        ExecutionError::IndexOutOfBounds,
        ExecutionError::ZeroAddress,
        ExecutionError::AddressCreationFailed,
        ExecutionError::EarlyEndOfStream,
        ExecutionError::Formatting,
        ExecutionError::LeftOverBytes,
        ExecutionError::OutOfMemory,
        ExecutionError::NotRepresentable,
        ExecutionError::ExceededRecursionDepth,
        ExecutionError::KeyNotFound,
        ExecutionError::CouldNotDeserializeSignature,
        ExecutionError::TypeMismatch,
        ExecutionError::CouldNotSignMessage,
        ExecutionError::EmptyDictionaryName,
        ExecutionError::MissingArg,
        ExecutionError::MissingAddress,
        ExecutionError::OutOfGas,
        ExecutionError::MaxUserError,
        ExecutionError::UserErrorTooHigh
    )
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

    fn call(error_msg: &str) -> Result<OdraError> {
        super::find(error_msg)
    }
}
