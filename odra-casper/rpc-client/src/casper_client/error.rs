use anyhow::{anyhow, Result};
use odra_core::ExecutionError;
use serde_json::Value;
use std::{fs, path::PathBuf};

use crate::log;

pub(crate) fn find(contract_name: &str, error_msg: &str) -> Result<String> {
    let schema_path = find_schema_file_path(contract_name)?;
    let schema = fs::read_to_string(schema_path)?;
    let error_num: u16 = error_msg
        .strip_prefix("User error: ")
        .ok_or_else(|| anyhow!("Couldn't parse error message: {:?}", error_msg))?
        .parse()?;

    if is_internal_error(error_num) {
        return Ok(get_internal_error_name(error_num));
    }

    let schema: Value = serde_json::from_str(&schema)?;
    let errors = schema["errors"]
        .as_array()
        .ok_or_else(|| anyhow!("Couldn't get value"))?;

    errors
        .iter()
        .find_map(|err| match_error(err, error_num as u64))
        .ok_or_else(|| anyhow!("Couldn't find error"))
}

fn match_error(val: &Value, error_num: u64) -> Option<String> {
    if val["discriminant"].as_u64() == Some(error_num) {
        val["name"].as_str().map(|s| s.to_string())
    } else {
        None
    }
}

#[inline]
fn is_internal_error(error_num: u16) -> bool {
    error_num >= ExecutionError::UserErrorTooHigh.code()
}

fn get_internal_error_name(error_num: u16) -> String {
    match error_num {
        x if x == ExecutionError::UnwrapError.code() => "ExecutionError::UnwrapError",
        x if x == ExecutionError::AdditionOverflow.code() => "ExecutionError::AdditionOverflow",
        x if x == ExecutionError::SubtractionOverflow.code() => {
            "ExecutionError::SubtractionOverflow"
        }
        x if x == ExecutionError::NonPayable.code() => "ExecutionError::NonPayable",
        x if x == ExecutionError::TransferToContract.code() => "ExecutionError::TransferToContract",
        x if x == ExecutionError::ReentrantCall.code() => "ExecutionError::ReentrantCall",
        x if x == ExecutionError::ContractAlreadyInstalled.code() => {
            "ExecutionError::ContractAlreadyInstalled"
        }
        x if x == ExecutionError::UnknownConstructor.code() => "ExecutionError::UnknownConstructor",
        x if x == ExecutionError::NativeTransferError.code() => {
            "ExecutionError::NativeTransferError"
        }
        x if x == ExecutionError::IndexOutOfBounds.code() => "ExecutionError::IndexOutOfBounds",
        x if x == ExecutionError::ZeroAddress.code() => "ExecutionError::ZeroAddress",
        x if x == ExecutionError::AddressCreationFailed.code() => {
            "ExecutionError::AddressCreationFailed"
        }
        x if x == ExecutionError::EarlyEndOfStream.code() => "ExecutionError::EarlyEndOfStream",
        x if x == ExecutionError::Formatting.code() => "ExecutionError::Formatting",
        x if x == ExecutionError::LeftOverBytes.code() => "ExecutionError::LeftOverBytes",
        x if x == ExecutionError::OutOfMemory.code() => "ExecutionError::OutOfMemory",
        x if x == ExecutionError::NotRepresentable.code() => "ExecutionError::NotRepresentable",
        x if x == ExecutionError::ExceededRecursionDepth.code() => {
            "ExecutionError::ExceededRecursionDepth"
        }
        x if x == ExecutionError::KeyNotFound.code() => "ExecutionError::KeyNotFound",
        x if x == ExecutionError::CouldNotDeserializeSignature.code() => {
            "ExecutionError::CouldNotDeserializeSignature"
        }
        x if x == ExecutionError::TypeMismatch.code() => "ExecutionError::TypeMismatch",
        x if x == ExecutionError::CouldNotSignMessage.code() => {
            "ExecutionError::CouldNotSignMessage"
        }
        x if x == ExecutionError::EmptyDictionaryName.code() => {
            "ExecutionError::EmptyDictionaryName"
        }
        x if x == ExecutionError::MissingArg.code() => "ExecutionError::MissingArg",
        x if x == ExecutionError::MaxUserError.code() => "ExecutionError::MaxUserError",
        x if x == ExecutionError::UserErrorTooHigh.code() => "ExecutionError::UserErrorTooHigh",
        _ => "UnknownError"
    }
    .to_string()
}

fn find_schema_file_path(contract_name: &str) -> Result<PathBuf> {
    #[cfg(test)]
    let mut path = PathBuf::from("resources/test");
    #[cfg(not(test))]
    let mut path = PathBuf::from("resources/casper_contract_schemas");

    path = path
        .join(format!("{}_schema.json", contract_name.to_lowercase()))
        .with_extension("json");
    let mut checked_paths = vec![];
    for _ in 0..2 {
        if path.exists() && path.is_file() {
            log::info(format!("Found schema under {:?}.", path));
            return Ok(path);
        } else {
            checked_paths.push(path.clone());
            path = path.parent().unwrap().to_path_buf();
        }
    }
    log::error(format!("Could not find schema under {:?}.", checked_paths));
    Err(anyhow!("Schema not found"))
}

#[cfg(test)]
mod test {
    use anyhow::Result;

    #[test]
    fn test_reading_errors() {
        // Contract errors
        assert_eq!(
            call("User error: 60017").ok(),
            Some("CannotTargetSelfUser".to_string())
        );
        assert_eq!(
            call("User error: 60010").ok(),
            Some("InsufficientRights".to_string())
        );
        // Odra error
        assert_eq!(
            call("User error: 64537").ok(),
            Some("ExecutionError::UnwrapError".to_string())
        );
        assert_eq!(
            call("User error: 64657").ok(),
            Some("ExecutionError::EmptyDictionaryName".to_string())
        );
        // Unknown user error
        assert!(call("User error: 60300").is_err());
        // Other errors
        assert!(call("Casper Engine error").is_err());
    }

    fn call(error_msg: &str) -> Result<String> {
        super::find("cep18", error_msg)
    }
}
