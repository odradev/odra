//! Entry points of a factory contract run natively by the host (OdraVM).
//!
//! A factory compiled to Wasm deploys and upgrades its children on its own; these functions are
//! what the entry points caller generated for a factory module runs instead.

use crate::entry_point_callback::EntryPointsCallerFn;
use crate::prelude::*;
use crate::{factory_dapp, utils, CallDef, ContractEnv, ExecutionEnv, VmError};
use casper_event_standard::EventInstance;
use casper_types::bytesrepr::{Bytes, FromBytes, ToBytes};
use casper_types::RuntimeArgs;

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
    let name: String = execution_env(env).get_named_arg(CONTRACT_NAME_ARG);
    let (address, access) = env.new_child_contract(&name, call_def.args().clone(), child)?;
    env.emit_event(event(name, address));
    utils::try_serialize(&(address, access))
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
    let name: String = execution_env(env).get_named_arg(CONTRACT_NAME_ARG);
    let address = env
        .upgrade_child_contract(&name, call_def.args().clone(), child)?
        .ok_or(OdraError::VmError(VmError::InvalidContractAddress))?;
    env.emit_event(event(name, address));
    utils::try_serialize(&())
}

/// Runs the `batch_upgrade_child_contract` entry point: upgrades every child contract named in
/// the `args` argument with its upgrade arguments. Names the factory does not know are skipped.
pub fn batch_upgrade_child_contract<E: ToBytes + EventInstance>(
    env: &ContractEnv,
    _call_def: &CallDef,
    child: EntryPointsCallerFn,
    event: fn(String, Address) -> E
) -> OdraResult<Bytes> {
    let args: BTreeMap<String, Bytes> = execution_env(env).get_named_arg(BATCH_ARGS_ARG);
    for (name, args_bytes) in args {
        let (upgrade_args, _) = RuntimeArgs::from_bytes(&args_bytes)
            .map_err(|err| OdraError::ExecutionError(err.into()))?;
        if let Some(address) = env.upgrade_child_contract(&name, upgrade_args, child)? {
            env.emit_event(event(name, address));
        }
    }
    utils::try_serialize(&())
}

/// Runs the `get_dapp_registry` entry point: returns the dapp registry the factory was pointed to
/// with `set_factory_dapp_registry`. Without one, it runs the `get_dapp_registry` of the child
/// module (`child`, if the module has the function) against the factory's storage, as the wasm a
/// factory shares with its children does, and fails with [ExecutionError::DappRegistryNotSet]
/// otherwise.
pub fn get_dapp_registry(
    env: &ContractEnv,
    call_def: &CallDef,
    child: Option<EntryPointsCallerFn>
) -> OdraResult<Bytes> {
    match factory_dapp::dapp_registry(env) {
        Some(registry) => utils::try_serialize(&registry),
        None => match child {
            Some(child) => child().call(env.clone(), call_def.clone()),
            None => Err(OdraError::ExecutionError(
                ExecutionError::DappRegistryNotSet
            ))
        }
    }
}

/// Runs the `set_factory_dapp_registry` entry point: points the factory to the registry named by
/// the `registry` argument and emits the event `event` builds from the previous and the new
/// registry. Only the holder of the factory access may call it, see
/// [ContractContext::caller_holds_factory_access](crate::ContractContext::caller_holds_factory_access).
pub fn set_dapp_registry<E: ToBytes + EventInstance>(
    env: &ContractEnv,
    event: fn(Option<Address>, Address) -> E
) -> OdraResult<Bytes> {
    if !env.caller_holds_factory_access()? {
        return Err(OdraError::VmError(VmError::InvalidContext));
    }
    let registry: Address = execution_env(env).get_named_arg(factory_dapp::REGISTRY_ARG);
    let previous_registry = factory_dapp::set_dapp_registry(env, registry);
    env.emit_event(event(previous_registry, registry));
    utils::try_serialize(&())
}

/// The execution environment of the factory's call, which reads the named arguments of the call
/// the way a generated entry point does: a missing or malformed one reverts.
fn execution_env(env: &ContractEnv) -> ExecutionEnv {
    ExecutionEnv::new(Rc::new(env.clone()))
}
