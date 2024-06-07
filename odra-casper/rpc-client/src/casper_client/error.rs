use anyhow::{anyhow, Result};
use odra_core::{ExecutionError, OdraError};
use serde_json::Value;
use std::{fs, path::PathBuf};

use crate::log;

pub(crate) fn find(contract_name: &str, error_msg: &str) -> Result<(String, OdraError)> {
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
        .find_map(|err| match_error(err, error_num))
        .ok_or_else(|| anyhow!("Couldn't find error"))
}

fn match_error(val: &Value, error_num: u16) -> Option<(String, OdraError)> {
    if val["discriminant"].as_u64() == Some(error_num as u64) {
        let odra_error = OdraError::user(error_num);
        val["name"]
            .as_str()
            .map(|s| s.to_string())
            .map(|s| (s, odra_error))
    } else {
        None
    }
}

#[inline]
fn is_internal_error(error_num: u16) -> bool {
    error_num >= ExecutionError::UserErrorTooHigh.code()
}

fn get_internal_error_name(error_num: u16) -> (String, OdraError) {
    match error_num {
        x if x == ExecutionError::UnwrapError.code() => (
            "ExecutionError::UnwrapError".to_string(),
            ExecutionError::UnwrapError.into()
        ),
        x if x == ExecutionError::AdditionOverflow.code() => (
            "ExecutionError::AdditionOverflow".to_string(),
            ExecutionError::AdditionOverflow.into()
        ),
        x if x == ExecutionError::SubtractionOverflow.code() => (
            "ExecutionError::SubtractionOverflow".to_string(),
            ExecutionError::SubtractionOverflow.into()
        ),
        x if x == ExecutionError::NonPayable.code() => (
            "ExecutionError::NonPayable".to_string(),
            ExecutionError::NonPayable.into()
        ),
        x if x == ExecutionError::TransferToContract.code() => (
            "ExecutionError::TransferToContract".to_string(),
            ExecutionError::TransferToContract.into()
        ),
        x if x == ExecutionError::ReentrantCall.code() => (
            "ExecutionError::ReentrantCall".to_string(),
            ExecutionError::ReentrantCall.into()
        ),
        x if x == ExecutionError::ContractAlreadyInstalled.code() => (
            "ExecutionError::ContractAlreadyInstalled".to_string(),
            ExecutionError::ContractAlreadyInstalled.into()
        ),
        x if x == ExecutionError::UnknownConstructor.code() => (
            "ExecutionError::UnknownConstructor".to_string(),
            ExecutionError::UnknownConstructor.into()
        ),
        x if x == ExecutionError::NativeTransferError.code() => (
            "ExecutionError::NativeTransferError".to_string(),
            ExecutionError::NativeTransferError.into()
        ),
        x if x == ExecutionError::IndexOutOfBounds.code() => (
            "ExecutionError::IndexOutOfBounds".to_string(),
            ExecutionError::IndexOutOfBounds.into()
        ),
        x if x == ExecutionError::ZeroAddress.code() => (
            "ExecutionError::ZeroAddress".to_string(),
            ExecutionError::ZeroAddress.into()
        ),
        x if x == ExecutionError::AddressCreationFailed.code() => (
            "ExecutionError::AddressCreationFailed".to_string(),
            ExecutionError::AddressCreationFailed.into()
        ),
        x if x == ExecutionError::EarlyEndOfStream.code() => (
            "ExecutionError::EarlyEndOfStream".to_string(),
            ExecutionError::EarlyEndOfStream.into()
        ),
        x if x == ExecutionError::Formatting.code() => (
            "ExecutionError::Formatting".to_string(),
            ExecutionError::Formatting.into()
        ),
        x if x == ExecutionError::LeftOverBytes.code() => (
            "ExecutionError::LeftOverBytes".to_string(),
            ExecutionError::LeftOverBytes.into()
        ),
        x if x == ExecutionError::OutOfMemory.code() => (
            "ExecutionError::OutOfMemory".to_string(),
            ExecutionError::OutOfMemory.into()
        ),
        x if x == ExecutionError::NotRepresentable.code() => (
            "ExecutionError::NotRepresentable".to_string(),
            ExecutionError::NotRepresentable.into()
        ),
        x if x == ExecutionError::ExceededRecursionDepth.code() => (
            "ExecutionError::ExceededRecursionDepth".to_string(),
            ExecutionError::ExceededRecursionDepth.into()
        ),
        x if x == ExecutionError::KeyNotFound.code() => (
            "ExecutionError::KeyNotFound".to_string(),
            ExecutionError::KeyNotFound.into()
        ),
        x if x == ExecutionError::CouldNotDeserializeSignature.code() => (
            "ExecutionError::CouldNotDeserializeSignature".to_string(),
            ExecutionError::CouldNotDeserializeSignature.into()
        ),
        x if x == ExecutionError::TypeMismatch.code() => (
            "ExecutionError::TypeMismatch".to_string(),
            ExecutionError::TypeMismatch.into()
        ),
        x if x == ExecutionError::CouldNotSignMessage.code() => (
            "ExecutionError::CouldNotSignMessage".to_string(),
            ExecutionError::CouldNotSignMessage.into()
        ),
        x if x == ExecutionError::EmptyDictionaryName.code() => (
            "ExecutionError::EmptyDictionaryName".to_string(),
            ExecutionError::EmptyDictionaryName.into()
        ),
        x if x == ExecutionError::MissingArg.code() => (
            "ExecutionError::MissingArg".to_string(),
            ExecutionError::MissingArg.into()
        ),
        x if x == ExecutionError::MaxUserError.code() => (
            "ExecutionError::MaxUserError".to_string(),
            ExecutionError::MaxUserError.into()
        ),
        x if x == ExecutionError::UserErrorTooHigh.code() => (
            "ExecutionError::UserErrorTooHigh".to_string(),
            ExecutionError::UserErrorTooHigh.into()
        ),
        _ => panic!("Unknown execution error code: {}", error_num)
    }
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
    use odra_core::{ExecutionError, OdraError};

    #[test]
    fn test_reading_errors() {
        // Contract errors
        assert_eq!(
            call("User error: 60017").ok(),
            Some(("CannotTargetSelfUser".to_string(), OdraError::user(60017)))
        );
        assert_eq!(
            call("User error: 60010").ok(),
            Some(("InsufficientRights".to_string(), OdraError::user(60010)))
        );
        // Odra error
        assert_eq!(
            call("User error: 64537").ok(),
            Some((
                "ExecutionError::UnwrapError".to_string(),
                ExecutionError::UnwrapError.into()
            ))
        );
        assert_eq!(
            call("User error: 64657").ok(),
            Some((
                "ExecutionError::EmptyDictionaryName".to_string(),
                ExecutionError::EmptyDictionaryName.into()
            ))
        );
        // Unknown user error
        assert!(call("User error: 60300").is_err());
        // Other errors
        assert!(call("Casper Engine error").is_err());
    }

    fn call(error_msg: &str) -> Result<(String, OdraError)> {
        super::find("cep18", error_msg)
    }
}
