use clap::ArgMatches;
use odra_core::{
    casper_types::U512,
    host::HostEnv,
    CallDef,
};
use odra_schema::casper_contract_schema::{Entrypoint, NamedCLType};

use crate::{args::{self, ARG_ATTACHED_VALUE}, container, types, CustomTypeSet, DeployedContractsContainer};

pub const DEFAULT_GAS: u64 = 20_000_000_000;

#[derive(Debug, thiserror::Error)]
pub enum CallError {
    #[error("Execution error: {0}")]
    ExecutionError(String),
    #[error(transparent)]
    ArgsError(#[from] args::ArgsError),
    #[error(transparent)]
    TypesError(#[from] types::Error),
    #[error("Contract not found")]
    ContractNotFound,
    #[error(transparent)]
    ContractError(#[from] container::ContractError),
}

pub fn call(
    env: &HostEnv,
    contract_name: &str,
    entry_point: &Entrypoint,
    args: &ArgMatches,
    types: &CustomTypeSet,
) -> Result<String, CallError> {
    let container = DeployedContractsContainer::load()?;
    let amount = args
        .try_get_one::<String>(ARG_ATTACHED_VALUE)
        .ok()
        .flatten()
        .map(|s| U512::from_dec_str(s).map_err(|_| types::Error::SerializationError))
        .unwrap_or(Ok(U512::zero()))?;

    let runtime_args = args::compose(&entry_point, args, types)?;
    let contract_address = container
        .address(contract_name)
        .ok_or(CallError::ContractNotFound)?;

    let method = &entry_point.name;
    let is_mut = entry_point.is_mutable;
    let ty = &entry_point.return_ty;
    let call_def = CallDef::new(method, is_mut, runtime_args).with_amount(amount);
    let use_proxy = ty.0 != NamedCLType::Unit || !call_def.amount().is_zero();

    if is_mut {
        env.set_gas(DEFAULT_GAS);
    }
    let bytes = env
        .raw_call_contract(contract_address, call_def, use_proxy)
        .map_err(|e| CallError::ExecutionError(format!("{:?}", e)))?;
    let result = args::decode(bytes.inner_bytes(), ty, types)?;
    Ok(result.0)
}
