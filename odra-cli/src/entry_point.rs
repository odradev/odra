#![allow(dead_code)]
use clap::ArgMatches;
use odra::prelude::{Address, OdraError};
use odra::schema::casper_contract_schema::{Entrypoint, NamedCLType};
use odra::VmError;
use odra::{casper_types::U512, host::HostEnv, CallDef};

use crate::cmd::args::{read_arg, Arg, ArgsError, ARG_PRINT_EVENTS};
use crate::container::ContractProvider;
use crate::custom_types::CustomTypeSet;
use crate::{container, types};

pub(crate) mod cmd_args;
mod runtime_args;
mod utils;

#[derive(Debug, thiserror::Error)]
pub enum CallError {
    #[error("Calling {package_name}::{method} failed with: {message}")]
    ExecutionError {
        package_name: String,
        method: String,
        message: String
    },
    #[error(transparent)]
    ArgsError(#[from] ArgsError),
    #[error(transparent)]
    TypesError(#[from] types::Error),
    #[error("Contract not found")]
    ContractNotFound,
    #[error(transparent)]
    ContractError(#[from] container::ContractError),
    #[error("Entry point '{entry_point}' not found in contract '{contract_name}'")]
    EntryPointNotFound {
        entry_point: String,
        contract_name: String
    },
    #[error("No entry point found in contract '{contract_name}'")]
    NoEntryPointFound { contract_name: String },
    #[error("Invalid gas value: {0}")]
    InvalidGasValue(String)
}

pub fn call<T: ContractProvider>(
    env: &HostEnv,
    contract_name: &str,
    entry_point: &Entrypoint,
    args: &ArgMatches,
    types: &CustomTypeSet,
    contract_provider: &T
) -> Result<String, CallError> {
    let amount = read_arg::<U512>(args, Arg::AttachedValue).unwrap_or_default();

    let runtime_args = runtime_args::compose(entry_point, args, types)?;
    let contract_address = contract_provider
        .address_by_name(contract_name)
        .ok_or(CallError::ContractNotFound)?;

    let method = &entry_point.name;
    let is_mut = entry_point.is_mutable;
    let ty = &entry_point.return_ty;
    let call_def = CallDef::new(method, is_mut, runtime_args).with_amount(amount);
    let use_proxy = ty.0 != NamedCLType::Unit || !call_def.amount().is_zero();

    if is_mut {
        let gas = read_arg(args, Arg::Gas).ok_or(CallError::InvalidGasValue(
            "Failed to read gas value. Use --gas <value> to specify it.".to_string()
        ))?;
        env.set_gas(gas);
    }

    let print_events = is_mut && args.get_flag(ARG_PRINT_EVENTS);
    if print_events {
        prettycli::info("Syncing events for the call...");
    }
    env.set_captures_events(print_events);
    let bytes = env
        .raw_call_contract(contract_address, call_def, use_proxy)
        .map_err(|e| CallError::ExecutionError {
            package_name: contract_name.to_string(),
            method: method.to_string(),
            message: match e {
                OdraError::VmError(VmError::Other(msg)) => msg,
                _ => format!("{:?}", e)
            }
        })?;

    if print_events {
        log_events(env, contract_provider, types, contract_address)?;
    }

    let result = types::decode(bytes.inner_bytes(), ty, types)?;
    Ok(result.0)
}

fn log_events<T: ContractProvider>(
    env: &HostEnv,
    contract_provider: &T,
    types: &CustomTypeSet,
    contract_address: Address
) -> Result<(), CallError> {
    let call_result = env.last_call_result(contract_address).raw_call_result();

    for deployed_contract in contract_provider.all_contracts() {
        let events = call_result.contract_events(&deployed_contract.address());
        if events.is_empty() {
            continue;
        }
        prettycli::info(&format!(
            "Captured {} events for contract '{}'",
            events.len(),
            deployed_contract.key_name()
        ));
        for (i, event) in events.iter().enumerate() {
            prettycli::info(&format!(
                "Event {}: {}",
                i + 1,
                types::decode_event(event, types)?
            ));
        }
    }
    Ok(())
}
