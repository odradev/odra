//! Module for handling Odra errors coming out of the Livenet execution.

use std::{fs, path::PathBuf};

use anyhow::{anyhow, Result};
use serde_json::Value;

use odra_core::{ExecutionError, OdraError};

/// Finds the error message in the contract schema.
pub fn find(contract_name: &str, error_msg: &str) -> Result<(String, OdraError)> {
    if error_msg == "Out of gas error" {
        return Ok(("OutOfGas".to_string(), ExecutionError::OutOfGas.into()));
    }

    let error_num: u16 = error_msg
        .strip_prefix("User error: ")
        .ok_or_else(|| anyhow!("Couldn't parse error message: {:?}", error_msg))?
        .parse()?;

    if is_internal_error(error_num) {
        return Ok(get_internal_error_name(error_num));
    }

    #[cfg(test)]
    let schema_path = PathBuf::from("resources/test");
    #[cfg(not(test))]
    let schema_path = PathBuf::from("resources/casper_contract_schemas");
    let schema_path =
        odra_schema::find_schema_file_path(contract_name, schema_path).map_err(|e| anyhow!(e))?;
    let schema = fs::read_to_string(schema_path)?;

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

macro_rules! match_error {
    ($err:expr) => {
        (stringify!($err).to_string(), $err.into())
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

fn get_internal_error_name(error_num: u16) -> (String, OdraError) {
    match_errors!(
        error_num,
        ExecutionError::UnwrapError,
        ExecutionError::AdditionOverflow,
        ExecutionError::SubtractionOverflow,
        ExecutionError::NonPayable,
        ExecutionError::TransferToContract,
        ExecutionError::ReentrantCall,
        ExecutionError::ContractAlreadyInstalled,
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
        ExecutionError::OutOfGas,
        ExecutionError::MaxUserError,
        ExecutionError::UserErrorTooHigh
    )
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
        assert_eq!(
            call("Out of gas error").ok(),
            Some(("OutOfGas".to_string(), ExecutionError::OutOfGas.into()))
        );
    }

    fn call(error_msg: &str) -> Result<(String, OdraError)> {
        super::find("cep18", error_msg)
    }
}
