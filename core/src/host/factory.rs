//! Entry points of a factory contract run natively by the host (OdraVM).
//!
//! A factory compiled to Wasm deploys and upgrades its children on its own; these functions are
//! what the entry points caller generated for a factory module runs instead.

use crate::entry_point_callback::EntryPointsCallerFn;
use crate::prelude::*;
use crate::{CallDef, ContractEnv, VmError};
use casper_event_standard::EventInstance;
use casper_types::bytesrepr::{Bytes, FromBytes, ToBytes};
use casper_types::{CLTyped, RuntimeArgs};

const CONTRACT_NAME_ARG: &str = "contract_name";
const BATCH_ARGS_ARG: &str = "args";

/// Runs the `new_contract` entry point: deploys a child contract named by the `contract_name`
/// argument, initialized with the rest of the call arguments, and emits the event `event`
/// builds from the name and the address of the child.
///
/// Returns the serialized address and access URef of the child.
pub fn new_contract<E: ToBytes + EventInstance>(
    env: &ContractEnv,
    call_def: &CallDef,
    child: EntryPointsCallerFn,
    event: fn(String, Address) -> E
) -> OdraResult<Bytes> {
    let name: String = named_arg(call_def, CONTRACT_NAME_ARG)?;
    let (address, access) = env.new_child_contract(&name, call_def.args().clone(), child)?;
    env.emit_event(event(name, address));
    to_bytes(&(address, access))
}

/// Runs the `upgrade_child_contract` entry point: upgrades the child contract named by the
/// `contract_name` argument, passing it the rest of the call arguments, and emits the event
/// `event` builds from the name and the address of the child.
pub fn upgrade_child_contract<E: ToBytes + EventInstance>(
    env: &ContractEnv,
    call_def: &CallDef,
    child: EntryPointsCallerFn,
    event: fn(String, Address) -> E
) -> OdraResult<Bytes> {
    let name: String = named_arg(call_def, CONTRACT_NAME_ARG)?;
    let address = env
        .upgrade_child_contract(&name, call_def.args().clone(), child)?
        .ok_or(OdraError::VmError(VmError::InvalidContractAddress))?;
    env.emit_event(event(name, address));
    to_bytes(&())
}

/// Runs the `batch_upgrade_child_contract` entry point: upgrades every child contract named in
/// the `args` argument with its upgrade arguments. Names the factory does not know are skipped.
pub fn batch_upgrade_child_contract<E: ToBytes + EventInstance>(
    env: &ContractEnv,
    call_def: &CallDef,
    child: EntryPointsCallerFn,
    event: fn(String, Address) -> E
) -> OdraResult<Bytes> {
    let args: BTreeMap<String, Bytes> = named_arg(call_def, BATCH_ARGS_ARG)?;
    for (name, args_bytes) in args {
        let (upgrade_args, _) = RuntimeArgs::from_bytes(&args_bytes)
            .map_err(|err| OdraError::ExecutionError(err.into()))?;
        if let Some(address) = env.upgrade_child_contract(&name, upgrade_args, child)? {
            env.emit_event(event(name, address));
        }
    }
    to_bytes(&())
}

fn named_arg<T: CLTyped + FromBytes>(call_def: &CallDef, name: &str) -> OdraResult<T> {
    call_def
        .get(name)
        .ok_or(OdraError::ExecutionError(ExecutionError::MissingArg))
}

fn to_bytes<T: ToBytes>(value: &T) -> OdraResult<Bytes> {
    value
        .to_bytes()
        .map(Into::into)
        .map_err(|err| OdraError::ExecutionError(err.into()))
}
