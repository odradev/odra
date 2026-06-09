//! Functions that interact with the casper host environment.
//!
//! This module provides functions for interacting with the casper host environment, including
//! installing contracts, reverting contract execution, accessing named arguments, getting the
//! block time, performing cryptographic operations, manipulating contract storage, transferring
//! tokens, emitting events, and more.
//!
//! Build on top of the [casper_contract] crate.

use crate::consts::{self, FACTORY_GROUP_NAME};
use crate::consts::{CONSTRUCTOR_GROUP_NAME, NATIVE_EVENT_TOPIC, UPGRADER_GROUP_NAME};
use casper_contract::contract_api::runtime::{emit_message, get_immediate_caller};
use casper_contract::contract_api::storage;
use casper_contract::contract_api::system;
use casper_contract::ext_ffi::{
    casper_emit_message, casper_remove_contract_user_group_urefs, casper_verify_signature
};
use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_contract::{
    contract_api::{
        self, runtime,
        system::{
            create_purse, get_purse_balance, transfer_from_purse_to_account,
            transfer_from_purse_to_purse
        }
    },
    ext_ffi
};
use core::mem::MaybeUninit;
use odra_core::casper_types::account::AccountHash;
use odra_core::casper_types::bytesrepr::deserialize;
use odra_core::casper_types::contract_messages::{MessagePayload, MessageTopicOperation};
use odra_core::casper_types::contracts::{
    ContractHash, ContractPackage, ContractPackageHash, ContractVersion
};
use odra_core::casper_types::system::auction::{self, BidAddr, BidKind, ValidatorBid};
use odra_core::casper_types::system::{Caller, CallerInfo};
use odra_core::casper_types::ApiError::User;
use odra_core::casper_types::Key::SmartContract;
use odra_core::casper_types::{self, HashAddr, Signature, StoredValue};
use odra_core::casper_types::{
    api_error, bytesrepr,
    bytesrepr::{Bytes, FromBytes, ToBytes},
    runtime_args, ApiError, CLType, CLTyped, CLValue, EntityAddr, EntityEntryPoint,
    EntryPointAccess, EntryPointPayment, EntryPointType, EntryPoints, Group, Key, NamedKeys,
    PackageAddr, PackageHash, Parameter, Parameters, PublicKey, RuntimeArgs, URef,
    DICTIONARY_ITEM_KEY_MAX_LENGTH, U512, UREF_SERIALIZED_LENGTH
};
use odra_core::consts::{
    ALLOW_KEY_OVERRIDE_ARG, CREATE_UPGRADE_GROUP, IS_FACTORY_UPGRADE_ARG, IS_UPGRADABLE_ARG,
    IS_UPGRADE_ARG, PACKAGE_HASH_KEY_NAME_ARG, PACKAGE_HASH_TO_UPGRADE_ARG, RANDOM_BYTES_COUNT
};
use odra_core::prelude::ExecutionError::{CannotExtractCallerInfo, CannotGetAnImmediateCaller};
use odra_core::validator::ValidatorInfo;
use odra_core::{args, prelude::*, CallDef};
use odra_core::{
    args::EntrypointArgument,
    casper_event_standard::{self, Schema, Schemas}
};

lazy_static::lazy_static! {
    static ref STATE: URef = {
        let key = runtime::get_key(consts::STATE_KEY).unwrap_or_revert();
        let state_uref: URef = *key.as_uref().unwrap_or_revert();
        state_uref
    };

    static ref STATE_BYTES: Vec<u8> = {
        (*STATE).into_bytes().unwrap_or_revert()
    };
}

pub(crate) static mut ATTACHED_VALUE: U512 = U512::zero();
static mut CALLER_OVERRIDE: bool = false;

/// Installs or upgrades a contract based on the provided entry points, events, and initialization arguments.
pub fn install_or_upgrade(
    entry_points: EntryPoints,
    events: Schemas,
    init_args: Option<RuntimeArgs>
) -> ContractPackageHash {
    let is_upgrade = runtime::try_get_named_arg(IS_UPGRADE_ARG).unwrap_or_default();
    if is_upgrade {
        upgrade_contract(entry_points, events, init_args)
    } else {
        install_new_contract(entry_points, events, init_args).0
    }
}

/// Installs a contract from a contract package.
///
/// Create a locked contract stored under a [Key::Hash]. The contract is upgradeable or not, depending on the
/// value of `odra_cfg_is_upgradable` argument.
///
/// If a contract with the same name already exists, it may be overriden depending on the value of `odra_cfg_allow_key_override`
/// argument.
///
/// Along with the contract, named keys with events and state are created.
pub fn install_new_contract(
    entry_points: EntryPoints,
    events: Schemas,
    init_args: Option<RuntimeArgs>
) -> (ContractPackageHash, URef) {
    // Extract named arguments, variables and check if the contract is upgradable.
    // And check if there is an existing contract.
    let package_hash_key_name: String = runtime::get_named_arg(PACKAGE_HASH_KEY_NAME_ARG);
    let package_hash_key = runtime::get_key(&package_hash_key_name);
    let allow_key_override: bool = runtime::get_named_arg(ALLOW_KEY_OVERRIDE_ARG);
    if package_hash_key.is_some() && !allow_key_override {
        revert(ExecutionError::CannotOverrideKeys);
    }
    let is_upgradable: bool = runtime::get_named_arg(IS_UPGRADABLE_ARG);
    let has_init = entry_points
        .get("init")
        .map(|ep| *ep.access() != EntryPointAccess::Template)
        .unwrap_or_default();
    let is_factory = entry_points.has_entry_point("new_contract");

    // Prepare named keys.
    let named_keys = initial_named_keys(events);

    // Prepare message topic
    let mut message_topics = BTreeMap::new();
    message_topics.insert(NATIVE_EVENT_TOPIC.to_string(), MessageTopicOperation::Add);

    // Create new contract.
    let access_uref_key = format!("{}_access_token", package_hash_key_name);
    if is_upgradable {
        storage::new_contract(
            entry_points,
            Some(named_keys),
            Some(package_hash_key_name.clone()),
            Some(access_uref_key.clone()),
            Some(message_topics)
        );
    } else {
        storage::new_locked_contract(
            entry_points,
            Some(named_keys),
            Some(package_hash_key_name.clone()),
            Some(access_uref_key.clone()),
            Some(message_topics)
        );
    };
    // Read package hash from the storage.
    let contract_hash: PackageHash = runtime::get_key(&package_hash_key_name)
        .unwrap_or_revert_with(ApiError::AllocLayout)
        .into_package_hash()
        .unwrap_or_revert_with(ApiError::BufferTooSmall);

    let contract_package_hash = ContractPackageHash::new(contract_hash.value());
    if has_init {
        let init_access = create_contract_user_group(contract_package_hash, CONSTRUCTOR_GROUP_NAME);
        let _: () = runtime::call_versioned_contract(
            contract_package_hash,
            None,
            "init",
            init_args.unwrap_or_default()
        );
        revoke_access_to_user_group(contract_package_hash, CONSTRUCTOR_GROUP_NAME, init_access);
    }

    let upgrade_access = create_contract_user_group(contract_package_hash, UPGRADER_GROUP_NAME);
    storage::remove_contract_user_group_urefs(
        contract_package_hash,
        UPGRADER_GROUP_NAME,
        BTreeSet::from([upgrade_access])
    )
    .unwrap_or_revert();

    if is_factory {
        let factory_group_uref =
            create_contract_user_group(contract_package_hash, FACTORY_GROUP_NAME);
        runtime::put_key(
            &format!("{}_factory_access", package_hash_key_name),
            Key::URef(factory_group_uref)
        );
        return (contract_package_hash, factory_group_uref);
    }

    let access_uref = runtime::get_key(&access_uref_key)
        .unwrap_or_revert_with(ApiError::AllocLayout)
        .into_uref()
        .unwrap_or_revert_with(ApiError::BufferTooSmall);

    (contract_package_hash, access_uref)
}

/// Upgrades a contract within package.
///
/// Creates a contract stored under a [Key::Hash]. The contract is upgradeable or not, depending on the
/// value of `odra_cfg_is_upgradable` argument.
pub fn upgrade_contract(
    entry_points: EntryPoints,
    events: Schemas,
    upgrade_args: Option<RuntimeArgs>
) -> ContractPackageHash {
    // Add `migrate_events` entry point to the contract. It is run during the every upgrade.
    let mut entry_points = entry_points;
    entry_points.add_entry_point(EntityEntryPoint::new(
        "migrate_events",
        Parameters::from([Parameter::new("schemas", CLType::Any)]),
        CLType::Unit,
        EntryPointAccess::Groups(vec![Group::new(UPGRADER_GROUP_NAME)]),
        EntryPointType::Called,
        EntryPointPayment::Caller
    ));

    let args = upgrade_args.unwrap_or_default();
    // Get named arguments.
    let is_factory_upgrade: bool =
        runtime::try_get_named_arg(IS_FACTORY_UPGRADE_ARG).unwrap_or_default();

    let (package_hash_to_upgrade, new_package_hash_key) = if is_factory_upgrade {
        let package_hash_to_upgrade = args
            .get(PACKAGE_HASH_TO_UPGRADE_ARG)
            .cloned()
            .unwrap_or_revert();
        let new_package_hash_key = args
            .get(PACKAGE_HASH_KEY_NAME_ARG)
            .cloned()
            .unwrap_or_revert();
        (
            package_hash_to_upgrade.into_t().unwrap_or_revert(),
            new_package_hash_key.into_t().unwrap_or_revert()
        )
    } else {
        (
            runtime::get_named_arg::<HashAddr>(PACKAGE_HASH_TO_UPGRADE_ARG),
            runtime::get_named_arg::<String>(PACKAGE_HASH_KEY_NAME_ARG)
        )
    };
    let allow_key_override: bool = runtime::get_named_arg(ALLOW_KEY_OVERRIDE_ARG);
    let create_user_group: bool = runtime::get_named_arg(CREATE_UPGRADE_GROUP);

    let has_upgrade = entry_points
        .get("upgrade")
        .map(|e| *e.access() != EntryPointAccess::Template)
        .unwrap_or_default();

    let package_hash = runtime::get_key(&new_package_hash_key);

    if package_hash.is_some() && !allow_key_override {
        revert(ExecutionError::CannotOverrideKeys);
    }

    // Prepare named keys.
    let named_keys = initial_named_keys(events.clone());

    let contract_package_hash = ContractPackageHash::new(package_hash_to_upgrade);
    let previous_contract_hash = get_latest_contract_hash(contract_package_hash);

    // Upgrade!
    storage::add_contract_version(
        contract_package_hash,
        entry_points,
        named_keys,
        BTreeMap::new()
    );

    // Store the new contract package hash under the provided key. We do it in case of user provided a new key.
    runtime::put_key(&new_package_hash_key, Key::from(contract_package_hash));

    // The user group should be already created during installation, but this
    // allows upgrading contracts deployed using previous Odra versions or without Odra.
    if create_user_group {
        let upgrade_access = storage::create_contract_user_group(
            contract_package_hash,
            UPGRADER_GROUP_NAME,
            0,
            BTreeSet::default()
        )
        .unwrap_or_revert();
    }

    // We enable access to upgrader functions ("upgrade" and "migrate_events");
    let new_uref =
        storage::provision_contract_user_group_uref(contract_package_hash, UPGRADER_GROUP_NAME)
            .unwrap_or_revert();

    // Call "migrate_events".
    let _: () = runtime::call_versioned_contract(
        contract_package_hash,
        None,
        "migrate_events",
        runtime_args! {
            "schemas" => events.0
        }
    );

    // Call "upgrade".
    if has_upgrade {
        let _: () = runtime::call_versioned_contract(contract_package_hash, None, "upgrade", args);
    }

    // We disable access to upgrader functions.
    storage::remove_contract_user_group_urefs(
        contract_package_hash,
        UPGRADER_GROUP_NAME,
        BTreeSet::from([new_uref])
    )
    .unwrap_or_revert();

    // Finally, we disable the previous contract version.
    storage::disable_contract_version(contract_package_hash, previous_contract_hash)
        .unwrap_or_revert_with(User(ExecutionError::CannotDisablePreviousVersion.code()));

    contract_package_hash
}

/// Stops a contract execution and reverts the state with a given error.
#[inline(always)]
pub fn revert<E>(error: E) -> !
where
    E: Into<OdraError>
{
    runtime::revert(User(error.into().code()))
}

/// Returns given named argument passed to the host. The result is not deserialized,
/// is returned as a `Vec<u8>`.
pub fn get_named_arg(name: &str) -> Result<Vec<u8>, ApiError> {
    let arg_size = get_named_arg_size(name)?;
    if arg_size > 0 {
        let data_non_null_ptr = contract_api::alloc_bytes(arg_size);
        let ret = unsafe {
            ext_ffi::casper_get_named_arg(
                name.as_bytes().as_ptr(),
                name.len(),
                data_non_null_ptr.as_ptr(),
                arg_size
            )
        };
        if ret != 0 {
            return Err(ApiError::from(ret as u32));
        }
        unsafe {
            Ok(Vec::from_raw_parts(
                data_non_null_ptr.as_ptr(),
                arg_size,
                arg_size
            ))
        }
    } else {
        Ok(Vec::new())
    }
}

/// Gets the current block time.
#[inline(always)]
pub fn get_block_time() -> u64 {
    runtime::get_blocktime().into()
}

/// Hashes the given bytes using the BLAKE2b hash function.
#[inline(always)]
pub fn blake2b(input: &[u8]) -> [u8; 32] {
    runtime::blake2b(input)
}

/// Writes a value under a key to the contract's storage.
pub fn set_value(key: &[u8], value: &[u8]) {
    let uref_ptr = (*STATE_BYTES).as_ptr();
    let uref_size = (*STATE_BYTES).len();

    let dictionary_item_key_size = key.len();
    let dictionary_item_key_ptr = key.as_ptr();

    let cl_value = CLValue::from_t(value.to_vec()).unwrap_or_revert();
    let (value_ptr, value_size, _bytes) = to_ptr(cl_value);

    let result = unsafe {
        let ret = ext_ffi::casper_dictionary_put(
            uref_ptr,
            uref_size,
            dictionary_item_key_ptr,
            dictionary_item_key_size,
            value_ptr,
            value_size
        );
        api_error::result_from(ret)
    };

    result.unwrap_or_revert();
}

/// Gets a value under a key from the contract's storage.
pub fn get_value(key: &[u8]) -> Option<Vec<u8>> {
    let uref_ptr = (*STATE_BYTES).as_ptr();
    let uref_size = (*STATE_BYTES).len();

    let dictionary_item_key_size = key.len();
    let dictionary_item_key_ptr = key.as_ptr();

    let value_size = {
        let mut value_size = MaybeUninit::uninit();
        let ret = unsafe {
            ext_ffi::casper_dictionary_get(
                uref_ptr,
                uref_size,
                dictionary_item_key_ptr,
                dictionary_item_key_size,
                value_size.as_mut_ptr()
            )
        };
        match api_error::result_from(ret) {
            Ok(_) => unsafe { value_size.assume_init() },
            Err(ApiError::ValueNotFound) => return None,
            Err(e) => runtime::revert(e)
        }
    };

    let value_bytes = read_host_buffer(value_size).unwrap_or_revert();
    let value_bytes = Vec::from_bytes(value_bytes.as_slice()).unwrap_or_revert();
    Some(value_bytes.0)
}

/// Writes a value under a named key to the contract's storage.
pub fn set_named_key(name: &str, value: CLValue) {
    match runtime::get_key(name) {
        Some(key) => {
            write(key, value);
        }
        None => {
            let new_uref = write_new(value);
            runtime::put_key(name, new_uref.into());
        }
    };
}

/// Gets a value under a named key from the contract's storage.
pub fn get_named_key(name: &str) -> Option<Bytes> {
    match runtime::get_key(name) {
        Some(key) => read(key),
        None => None
    }
}

/// Writes a value under a key in a dictionary to a contract's storage.
pub fn set_dictionary_value(dictionary_name: &str, key: &[u8], value: CLValue) {
    let dictionary_uref = get_dictionary(dictionary_name);
    let (uref_ptr, uref_size, _bytes1) = to_ptr(dictionary_uref);
    let (dictionary_item_key_ptr, dictionary_item_key_size) = dictionary_item_key_to_ptr(key);

    if dictionary_item_key_size > DICTIONARY_ITEM_KEY_MAX_LENGTH {
        runtime::revert(ApiError::DictionaryItemKeyExceedsLength)
    }

    let (cl_value_ptr, cl_value_size, _bytes) = to_ptr(value);

    let result = unsafe {
        let ret = ext_ffi::casper_dictionary_put(
            uref_ptr,
            uref_size,
            dictionary_item_key_ptr,
            dictionary_item_key_size,
            cl_value_ptr,
            cl_value_size
        );
        api_error::result_from(ret)
    };

    result.unwrap_or_revert()
}

/// Removes the [`Key`] stored under `dictionary_name` in the current context's named keys.
#[inline]
pub fn remove_dictionary(dictionary_name: &str) {
    runtime::remove_key(dictionary_name);
}

/// Initializes an empty dictionary with the given name if it does not exist.
#[inline]
pub fn init_dictionary(dictionary_name: &str) {
    // Reuse `get_dictionary` to create a new dictionary if it does not exist.
    let _ = get_dictionary(dictionary_name);
}

/// Gets a value under a key in a dictionary from the contract's storage.
pub fn get_dictionary_value(dictionary_name: &str, key: &[u8]) -> Option<Bytes> {
    let dictionary_uref = get_dictionary(dictionary_name);
    let (uref_ptr, uref_size, _bytes1) = to_ptr(dictionary_uref);
    let (dictionary_item_key_ptr, dictionary_item_key_size) = dictionary_item_key_to_ptr(key);

    if dictionary_item_key_size > DICTIONARY_ITEM_KEY_MAX_LENGTH {
        runtime::revert(ApiError::DictionaryItemKeyExceedsLength)
    }

    let value_size = {
        let mut value_size = MaybeUninit::uninit();
        let ret = unsafe {
            ext_ffi::casper_dictionary_get(
                uref_ptr,
                uref_size,
                dictionary_item_key_ptr,
                dictionary_item_key_size,
                value_size.as_mut_ptr()
            )
        };
        match api_error::result_from(ret) {
            Ok(_) => unsafe { value_size.assume_init() },
            Err(ApiError::ValueNotFound) => return None,
            Err(e) => runtime::revert(e)
        }
    };

    let value_bytes = read_host_buffer(value_size).unwrap_or_revert();
    Some(Bytes::from(value_bytes))
}

fn get_dictionary(name: &str) -> URef {
    if name.is_empty() {
        runtime::revert(ApiError::MissingKey)
    }

    let dictionary_uref = match runtime::get_key(name) {
        Some(dictionary_key) => *dictionary_key.as_uref().unwrap_or_revert(),
        None => {
            let value_size = {
                let mut value_size = MaybeUninit::uninit();
                let ret = unsafe { ext_ffi::casper_new_dictionary(value_size.as_mut_ptr()) };
                api_error::result_from(ret).unwrap_or_revert();
                unsafe { value_size.assume_init() }
            };
            let value_bytes = read_host_buffer(value_size).unwrap_or_revert();
            let uref: URef = bytesrepr::deserialize(value_bytes).unwrap_or_revert();
            runtime::put_key(name, Key::from(uref));
            uref
        }
    };
    dictionary_uref
}

/// Verifies the signature of the given message against the given public key.
pub fn verify_signature(
    message: &[u8],
    signature: &Signature,
    public_key: &PublicKey
) -> Result<(), ApiError> {
    casper_contract::contract_api::cryptography::verify_signature(message, signature, public_key)
}

/// Transfers native token from the contract caller to the given address.
pub fn transfer_tokens(to: &Address, amount: &U512) {
    let main_purse = get_or_create_main_purse();
    // runtime::ver
    // casper_verify_signature(message_ptr, message_size, signature_ptr, signature_size, public_key_ptr, public_key_size)
    match to {
        Address::Account(account) => {
            transfer_from_purse_to_account(main_purse, *account, *amount, None).unwrap_or_revert();
        }
        // todo: Why?
        Address::Contract(_) => revert(ExecutionError::TransferToContract)
    };
}

/// Writes an event to the contract's storage.
pub fn emit_event(event: &Bytes) {
    casper_event_standard::emit_bytes(event.clone())
}

/// Emits a native event.
pub fn emit_native_event(event: &Bytes) {
    let payload = MessagePayload::Bytes(event.clone());
    emit_message(NATIVE_EVENT_TOPIC, &payload).unwrap_or_revert();
}

/// Gets the immediate session caller of the current execution.
#[inline(always)]
pub fn caller() -> OdraResult<Address> {
    let caller = if unsafe { CALLER_OVERRIDE } {
        caller_info_to_caller(take_nth_caller_from_stack(2))?
    } else {
        caller_info_to_caller(get_immediate_caller().unwrap_or_revert())?
    };
    Ok(Address::from(caller))
}

/// Calls a contract method by Address
#[inline(always)]
pub fn call_contract(address: Address, call_def: CallDef) -> Bytes {
    let package_hash = *address.as_contract_package_hash().unwrap_or_revert();
    let method = call_def.entry_point();
    let mut args = call_def.args().to_owned();
    if call_def.amount() == U512::zero() {
        call_versioned_contract(package_hash, None, method, args)
    } else {
        let cargo_purse = get_or_create_cargo_purse();
        let main_purse = get_main_purse().unwrap_or_revert();

        transfer_from_purse_to_purse(main_purse, cargo_purse, call_def.amount(), None)
            .unwrap_or_revert_with(ApiError::Transfer);
        args.insert(consts::CARGO_PURSE_ARG, cargo_purse)
            .unwrap_or_revert();

        let result = call_versioned_contract(package_hash, None, method, args);
        if !is_purse_empty(cargo_purse) {
            runtime::revert(ApiError::InvalidPurse)
        }
        result
    }
}

/// Gets the address of the currently run contract
#[inline(always)]
pub fn self_address() -> OdraResult<Address> {
    let first_elem = take_nth_caller_from_stack(0);
    let caller = caller_info_to_caller(first_elem)?;
    Ok(Address::from(caller))
}

/// Gets the balance of the current contract.
#[inline(always)]
pub fn self_balance() -> U512 {
    let main_purse = get_or_create_main_purse();
    get_purse_balance(main_purse).unwrap_or_revert()
}

/// Invokes the specified `entry_point_name` of stored logic at a specific`contract_package_hash`
/// address.
///
/// It does it for the most current version of a contract package by default or a specific
/// `contract_version` if one is provided, and passing the provided `runtime_args` to it.
pub fn call_versioned_contract(
    contract_package_hash: ContractPackageHash,
    contract_version: Option<ContractVersion>,
    entry_point_name: &str,
    runtime_args: RuntimeArgs
) -> Bytes {
    let (contract_package_hash_ptr, package_hash_size, _bytes) = to_ptr(contract_package_hash);
    let (contract_version_ptr, contract_version_size, _bytes) = to_ptr(contract_version);
    let (entry_point_name_ptr, entry_point_name_size, _bytes) = to_ptr(entry_point_name);
    let (runtime_args_ptr, runtime_args_size, _bytes) = to_ptr(runtime_args);

    let bytes_written = {
        let mut bytes_written = MaybeUninit::uninit();
        let ret = unsafe {
            ext_ffi::casper_call_versioned_contract(
                contract_package_hash_ptr,
                package_hash_size,
                contract_version_ptr,
                contract_version_size,
                entry_point_name_ptr,
                entry_point_name_size,
                runtime_args_ptr,
                runtime_args_size,
                bytes_written.as_mut_ptr()
            )
        };
        api_error::result_from(ret).unwrap_or_revert();
        unsafe { bytes_written.assume_init() }
    };
    odra_core::casper_types::bytesrepr::Bytes::from(deserialize_contract_result(bytes_written))
}

/// Reads from memory the amount attached to the current call.
pub fn attached_value() -> U512 {
    unsafe { ATTACHED_VALUE }
}

/// Stores in memory the amount attached to the current call.
pub fn set_attached_value(amount: U512) {
    unsafe {
        ATTACHED_VALUE = amount;
    }
}

/// Zeroes the amount attached to the current call.
pub fn clear_attached_value() {
    unsafe { ATTACHED_VALUE = U512::zero() }
}

/// Checks if given named argument exists.
pub fn named_arg_exists(name: &str) -> bool {
    let mut arg_size: usize = 0;
    let ret = unsafe {
        casper_contract::ext_ffi::casper_get_named_arg_size(
            name.as_bytes().as_ptr(),
            name.len(),
            &mut arg_size as *mut usize
        )
    };
    ret == 0
}

/// Transfers attached value to the currently executing contract.
pub fn handle_attached_value() {
    // If the cargo purse argument is not present, do nothing.
    // Attached value is set to zero by default.
    if !named_arg_exists(consts::CARGO_PURSE_ARG) {
        return;
    }

    // Handle attached value.
    let cargo_purse = runtime::get_named_arg(consts::CARGO_PURSE_ARG);
    let amount = get_purse_balance(cargo_purse);
    if let Some(amount) = amount {
        let contract_purse = get_or_create_main_purse();
        transfer_from_purse_to_purse(cargo_purse, contract_purse, amount, None).unwrap_or_revert();
        set_attached_value(amount);
    } else {
        revert(ExecutionError::NativeTransferError)
    }
}

/// Creates a new purse under the `__contract_main_purse` key for the currently executing contract
/// if it doesn't exist, or returns the existing main purse.
///
/// # Returns
///
/// The main purse as a [`URef`] if it already exists, otherwise a new purse is created and returned.
pub fn get_or_create_main_purse() -> URef {
    get_main_purse().unwrap_or_else(|| {
        let purse = create_purse();
        runtime::put_key(consts::CONTRACT_MAIN_PURSE, purse.into());
        purse
    })
}

/// Gets the main purse of the currently executing contract.
///
/// # Returns
///
/// The main purse as a [`URef`] if it exists, otherwise `None` is returned.
#[inline(always)]
pub fn get_main_purse() -> Option<URef> {
    runtime::get_key(consts::CONTRACT_MAIN_PURSE).and_then(|key| key.as_uref().cloned())
}

fn initial_named_keys(schemas: Schemas) -> NamedKeys {
    let mut named_keys = NamedKeys::new();
    named_keys.insert(
        String::from(consts::STATE_KEY),
        Key::URef(new_dictionary_uref(consts::STATE_KEY).unwrap_or_revert())
    );
    named_keys.insert(
        String::from(casper_event_standard::EVENTS_DICT),
        Key::URef(new_dictionary_uref(casper_event_standard::EVENTS_DICT).unwrap_or_revert())
    );
    named_keys.insert(
        String::from(casper_event_standard::EVENTS_LENGTH),
        Key::URef(storage::new_uref(0u32))
    );
    named_keys.insert(
        String::from(casper_event_standard::CES_VERSION_KEY),
        Key::URef(storage::new_uref(casper_event_standard::CES_VERSION))
    );
    named_keys.insert(
        String::from(casper_event_standard::EVENTS_SCHEMA),
        Key::URef(storage::new_uref(schemas))
    );

    named_keys
}

fn deserialize_contract_result(bytes_written: usize) -> Vec<u8> {
    if bytes_written == 0 {
        // If no bytes were written, the host buffer hasn't been set and hence shouldn't be read.
        vec![]
    } else {
        // NOTE: this is a copy of the contents of `read_host_buffer()`.  Calling that directly from
        // here causes several contracts to fail with a Wasmi `Unreachable` error.
        let bytes_non_null_ptr = contract_api::alloc_bytes(bytes_written);
        let mut dest: Vec<u8> = unsafe {
            Vec::from_raw_parts(bytes_non_null_ptr.as_ptr(), bytes_written, bytes_written)
        };
        read_host_buffer_into(&mut dest).unwrap_or_revert();
        dest
    }
}

fn take_nth_caller_from_stack(n: usize) -> CallerInfo {
    runtime::get_call_stack()
        .into_iter()
        .nth_back(n)
        .unwrap_or_revert()
}

fn create_contract_user_group(
    contract_package_hash: ContractPackageHash,
    group_label: &str
) -> URef {
    storage::create_contract_user_group(contract_package_hash, group_label, 1, Default::default())
        .unwrap_or_revert()
        .pop()
        .unwrap_or_revert()
}

fn revoke_access_to_user_group(
    contract_package_hash: ContractPackageHash,
    group_label: &str,
    constructor_access: URef
) {
    let mut urefs = BTreeSet::new();
    urefs.insert(constructor_access);
    storage::remove_contract_user_group_urefs(contract_package_hash, group_label, urefs)
        .unwrap_or_revert();
}

fn is_purse_empty(purse: URef) -> bool {
    get_purse_balance(purse)
        .map(|balance| balance.is_zero())
        .unwrap_or_else(|| true)
}

fn get_or_create_cargo_purse() -> URef {
    match runtime::get_key(consts::CONTRACT_CARGO_PURSE) {
        Some(key) => *key.as_uref().unwrap_or_revert(),
        None => {
            let purse = create_purse();
            runtime::put_key(consts::CONTRACT_CARGO_PURSE, purse.into());
            purse
        }
    }
}

fn to_ptr<T: ToBytes>(t: T) -> (*const u8, usize, Vec<u8>) {
    let bytes = t.into_bytes().unwrap_or_revert();
    let ptr = bytes.as_ptr();
    let size = bytes.len();
    (ptr, size, bytes)
}

fn dictionary_item_key_to_ptr(dictionary_item_key: &[u8]) -> (*const u8, usize) {
    let ptr = dictionary_item_key.as_ptr();
    let size = dictionary_item_key.len();
    (ptr, size)
}

fn write(key: Key, value: CLValue) {
    let (key_ptr, key_size, _bytes) = to_ptr(key);
    let (value_ptr, value_size, _bytes) = to_ptr(value);

    unsafe {
        ext_ffi::casper_write(key_ptr, key_size, value_ptr, value_size);
    }
}

fn write_new(value: CLValue) -> URef {
    let uref_non_null_ptr = contract_api::alloc_bytes(UREF_SERIALIZED_LENGTH);
    let (cl_value_ptr, cl_value_size, _cl_value_bytes) = to_ptr(value);
    let bytes = unsafe {
        ext_ffi::casper_new_uref(uref_non_null_ptr.as_ptr(), cl_value_ptr, cl_value_size); // URef has `READ_ADD_WRITE`
        Vec::from_raw_parts(
            uref_non_null_ptr.as_ptr(),
            UREF_SERIALIZED_LENGTH,
            UREF_SERIALIZED_LENGTH
        )
    };
    deserialize(bytes).unwrap_or_revert()
}

fn read(key: Key) -> Option<Bytes> {
    let (key_ptr, key_size, _bytes) = to_ptr(key);
    let value_size = {
        let mut value_size = MaybeUninit::uninit();
        let ret = unsafe { ext_ffi::casper_read_value(key_ptr, key_size, value_size.as_mut_ptr()) };
        match api_error::result_from(ret) {
            Ok(_) => unsafe { value_size.assume_init() },
            Err(e) => runtime::revert(e)
        }
    };

    let value_bytes = read_host_buffer(value_size).unwrap_or_revert();
    Some(Bytes::from(value_bytes))
}

fn read_host_buffer(size: usize) -> Result<Vec<u8>, ApiError> {
    let mut dest: Vec<u8> = if size == 0 {
        Vec::new()
    } else {
        let bytes_non_null_ptr = contract_api::alloc_bytes(size);
        unsafe { Vec::from_raw_parts(bytes_non_null_ptr.as_ptr(), size, size) }
    };
    read_host_buffer_into(&mut dest)?;
    Ok(dest)
}

fn read_host_buffer_into(dest: &mut [u8]) -> Result<usize, ApiError> {
    let mut bytes_written = MaybeUninit::uninit();
    let ret = unsafe {
        ext_ffi::casper_read_host_buffer(dest.as_mut_ptr(), dest.len(), bytes_written.as_mut_ptr())
    };
    api_error::result_from(ret)?;
    Ok(unsafe { bytes_written.assume_init() })
}

fn get_named_arg_size(name: &str) -> Result<usize, ApiError> {
    let mut arg_size: usize = 0;
    let ret = unsafe {
        ext_ffi::casper_get_named_arg_size(
            name.as_bytes().as_ptr(),
            name.len(),
            &mut arg_size as *mut usize
        )
    };
    match ret {
        0 => Ok(arg_size),
        _ => Err(ApiError::from(ret as u32))
    }
}

fn caller_info_to_caller(info: CallerInfo) -> OdraResult<Caller> {
    let kind = info.kind();
    match kind {
        0 => {
            let account_hash = info
                .get_field_by_index(0)
                .map(|val| val.to_t::<Option<AccountHash>>().unwrap_or_revert())
                .ok_or(ExecutionError::CannotExtractCallerInfo)?
                .ok_or(ExecutionError::CannotExtractCallerInfo)?;
            Ok(Caller::Initiator { account_hash })
        }
        3 => {
            let package_hash = info
                .get_field_by_index(1)
                .map(|val| {
                    val.to_t::<Option<PackageHash>>()
                        .map_err(|_| ExecutionError::CannotExtractCallerInfo)
                })
                .ok_or(ExecutionError::CannotExtractCallerInfo)?
                .map_err(|_| ExecutionError::CannotExtractCallerInfo)?
                .ok_or(ExecutionError::CannotExtractCallerInfo)?;
            let entity_addr = info
                .get_field_by_index(3)
                .map(|val| {
                    val.to_t::<Option<EntityAddr>>()
                        .map_err(|_| ExecutionError::CannotExtractCallerInfo)
                })
                .ok_or(ExecutionError::CannotExtractCallerInfo)?
                .map_err(|_| ExecutionError::CannotExtractCallerInfo)?
                .ok_or(ExecutionError::CannotExtractCallerInfo)?;
            Ok(Caller::Entity {
                package_hash,
                entity_addr
            })
        }
        4 => {
            let contract_package_hash = info
                .get_field_by_index(2)
                .map(|val| {
                    val.to_t::<Option<ContractPackageHash>>()
                        .map_err(|_| ExecutionError::CannotExtractCallerInfo)
                })
                .ok_or(ExecutionError::CannotExtractCallerInfo)?
                .map_err(|_| ExecutionError::CannotExtractCallerInfo)?
                .ok_or(ExecutionError::CannotExtractCallerInfo)?;
            let contract_hash = info
                .get_field_by_index(4)
                .map(|val| val.to_t::<Option<ContractHash>>().unwrap_or_revert())
                .expect("must have index 4 in fields")
                .expect("contract hash must be some");
            Ok(Caller::SmartContract {
                contract_package_hash,
                contract_hash
            })
        }
        _ => revert(ExecutionError::CannotExtractCallerInfo)
    }
}

/// Delegate tokens to a validator
pub fn delegate(validator: PublicKey, amount: U512) {
    let purse = get_main_purse().unwrap_or_revert_with(ApiError::InvalidPurse);
    let contract_hash = system::get_auction();
    let mut args = RuntimeArgs::new();
    args.insert(auction::ARG_DELEGATOR_PURSE, purse)
        .unwrap_or_revert();
    args.insert(auction::ARG_VALIDATOR, validator)
        .unwrap_or_revert();
    args.insert(auction::ARG_AMOUNT, amount).unwrap_or_revert();

    runtime::call_contract::<U512>(contract_hash, auction::METHOD_DELEGATE, args);
}

/// Undelegate tokens from a validator
pub fn undelegate(validator: PublicKey, amount: U512) {
    let purse = get_main_purse().unwrap_or_revert_with(ApiError::InvalidPurse);
    let contract_hash = system::get_auction();
    let mut args = RuntimeArgs::new();
    args.insert(auction::ARG_DELEGATOR_PURSE, purse)
        .unwrap_or_revert();
    args.insert(auction::ARG_VALIDATOR, validator)
        .unwrap_or_revert();
    args.insert(auction::ARG_AMOUNT, amount).unwrap_or_revert();

    runtime::call_contract::<U512>(contract_hash, auction::METHOD_UNDELEGATE, args);
}

/// Retrieves the amount of tokens delegated to the validator by the caller (the contract)
pub fn delegated_amount(public_key: PublicKey) -> U512 {
    let purse = match get_main_purse() {
        Some(p) => p,
        None => return U512::zero()
    };
    let account_hash = public_key.to_account_hash();
    let key = Key::BidAddr(BidAddr::DelegatedPurse {
        validator: account_hash,
        delegator: purse.addr()
    });

    storage::read_from_key(key)
        .ok()
        .and_then(|stored_value| stored_value)
        .and_then(|bid_kind| match bid_kind {
            BidKind::Delegator(purse) => Some(purse.staked_amount()),
            _ => None
        })
        .unwrap_or_else(U512::zero)
}

/// Returns a pseudorandom byte array
pub fn pseudorandom_bytes() -> [u8; RANDOM_BYTES_COUNT] {
    runtime::random_bytes()
}

/// Retrieves ValidatorBid from the storage
pub fn get_validator_info(validator: PublicKey) -> Option<ValidatorInfo> {
    let account_hash = validator.to_account_hash();
    let key = Key::BidAddr(BidAddr::Validator(account_hash));

    storage::read_from_key(key)
        .ok()
        .and_then(|stored_value| stored_value)
        .and_then(|bid_kind| match bid_kind {
            BidKind::Validator(bid) => Some(*bid),
            _ => None
        })
        .map(|validator_bid| {
            ValidatorInfo::new(
                validator_bid.staked_amount(),
                validator_bid.minimum_delegation_amount()
            )
        })
}

/// Retrieves latest contract version from the storage
pub fn get_latest_contract_hash(contract_package_hash: ContractPackageHash) -> ContractHash {
    let key = Key::from(contract_package_hash);

    storage::read_from_key::<ContractPackage>(key)
        .ok()
        .and_then(|opt_contract_package| opt_contract_package)
        .and_then(|contract_package| contract_package.current_contract_hash())
        .unwrap_or_revert_with(ApiError::ContractNotFound)
}

/// Retrieves latest contract version number from the storage
pub fn get_latest_contract_version(contract_package_hash: ContractPackageHash) -> u32 {
    let key = Key::from(contract_package_hash);

    storage::read_from_key::<ContractPackage>(key)
        .ok()
        .and_then(|opt_contract_package| opt_contract_package)
        .and_then(|contract_package| contract_package.current_contract_version())
        .map(|version| version.contract_version())
        .unwrap_or_revert_with(ApiError::ContractNotFound)
}

/// Creates new [`URef`] that represents a seed for a dictionary partition of the global state
/// without putting it under named keys.
pub fn new_dictionary_uref(dictionary_name: &str) -> Result<URef, ApiError> {
    if dictionary_name.is_empty() {
        return Err(ApiError::InvalidArgument);
    }

    let value_size = {
        let mut value_size = MaybeUninit::uninit();
        let ret = unsafe { ext_ffi::casper_new_dictionary(value_size.as_mut_ptr()) };
        api_error::result_from(ret)?;
        unsafe { value_size.assume_init() }
    };
    let value_bytes = read_host_buffer(value_size).unwrap_or_revert();
    let uref: URef = bytesrepr::deserialize(value_bytes).unwrap_or_revert();
    Ok(uref)
}

/// Sets a flag to override the caller for factory contracts.
pub fn override_factory_caller() {
    unsafe {
        CALLER_OVERRIDE = true;
    }
}
