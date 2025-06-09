use std::path::PathBuf;
use std::str::FromStr;

use clap::ArgMatches;
use odra::prelude::OdraError;
use odra::schema::casper_contract_schema::{Entrypoint, NamedCLType};
use odra::VmError;
use odra::{casper_types::U512, host::HostEnv, CallDef};

use crate::{
    args::{self, ARG_ATTACHED_VALUE, ARG_GAS},
    container, types, CustomTypeSet, DeployedContractsContainer
};

#[derive(Debug, thiserror::Error)]
pub enum CallError {
    #[error("Calling {contract_name}::{method} failed with: {message}")]
    ExecutionError {
        contract_name: String,
        method: String,
        message: String
    },
    #[error(transparent)]
    ArgsError(#[from] args::ArgsError),
    #[error(transparent)]
    TypesError(#[from] types::Error),
    #[error("Contract not found")]
    ContractNotFound,
    #[error(transparent)]
    ContractError(#[from] container::ContractError)
}

pub fn call(
    env: &HostEnv,
    contract_name: &str,
    entry_point: &Entrypoint,
    args: &ArgMatches,
    types: &CustomTypeSet,
    contracts_path: Option<PathBuf>
) -> Result<String, CallError> {
    let container = DeployedContractsContainer::load(contracts_path)?;
    let amount = args::read(args, ARG_ATTACHED_VALUE, U512::from_dec_str)?;

    let runtime_args = args::compose(entry_point, args, types)?;
    let contract_address = container
        .address(contract_name)
        .ok_or(CallError::ContractNotFound)?;

    let method = &entry_point.name;
    let is_mut = entry_point.is_mutable;
    let ty = &entry_point.return_ty;
    let call_def = CallDef::new(method, is_mut, runtime_args).with_amount(amount);
    let use_proxy = ty.0 != NamedCLType::Unit || !call_def.amount().is_zero();

    if is_mut {
        let gas = args::read(args, ARG_GAS, FromStr::from_str)?;
        env.set_gas(gas);
    }
    let bytes = env
        .raw_call_contract(contract_address, call_def, use_proxy)
        .map_err(|e| CallError::ExecutionError {
            contract_name: contract_name.to_string(),
            method: method.to_string(),
            message: match e {
                OdraError::VmError(VmError::Other(msg)) => msg,
                _ => format!("{:?}", e)
            }
        })?;
    let result = args::decode(bytes.inner_bytes(), ty, types)?;
    Ok(result.0)
}
