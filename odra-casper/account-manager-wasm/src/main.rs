#![doc = "Account manager session code: changes the associated keys and the action thresholds"]
#![doc = "of the account that sends it, in one transaction. See `odra-casper-account-manager`."]
#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("This binary only supports wasm32 target architecture!");

extern crate alloc;
#[cfg(target_arch = "wasm32")]
extern crate ink_allocator;

#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

use alloc::{collections::BTreeSet, vec::Vec};

use casper_contract::contract_api::{account, runtime};
use casper_types::account::{
    AccountHash, ActionType, AddKeyFailure, SetThresholdFailure, UpdateKeyFailure, Weight
};
use odra_casper_account_manager::{
    Error, DEPLOYMENT_THRESHOLD_ARG, KEY_MANAGEMENT_THRESHOLD_ARG, REMOVE_KEYS_ARG, SET_KEYS_ARG
};

/// Session code cannot read the account's keys or thresholds, so the order is found by trying:
/// every failed call leaves the account unchanged, and the execution engine checks the signers'
/// weight against the account as it was when the transaction started.
///
/// 1. Add each key in `set_keys`, or update its weight if it exists. A lowered weight that
///    would take the total below a current threshold is deferred to step 3.
/// 2. Set the thresholds: `key_management` first, so that raising both works; if it would go
///    below the current `deployment`, both go down and `deployment` is set first.
/// 3. Apply the deferred weights, then remove the keys in `remove_keys`.
///
/// Step 1 only raises the total weight and step 3 only lowers it towards the final total, so a
/// change whose end state is valid never trips the engine's threshold checks on the way.
#[no_mangle]
fn call() {
    let set_keys: Vec<(AccountHash, u8)> = runtime::get_named_arg(SET_KEYS_ARG);
    let remove_keys: Vec<AccountHash> = runtime::get_named_arg(REMOVE_KEYS_ARG);
    let deployment: Option<u8> = runtime::get_named_arg(DEPLOYMENT_THRESHOLD_ARG);
    let key_management: Option<u8> = runtime::get_named_arg(KEY_MANAGEMENT_THRESHOLD_ARG);

    validate(&set_keys, &remove_keys, deployment, key_management);

    let mut deferred = Vec::new();
    for (key, weight) in set_keys {
        let weight = Weight::new(weight);
        match account::add_associated_key(key, weight) {
            Ok(()) => {}
            // The engine checks the key limit before it looks for the key, so an account with
            // the maximum number of keys reports MaxKeysLimit for an existing key too.
            Err(add_failure @ (AddKeyFailure::DuplicateKey | AddKeyFailure::MaxKeysLimit)) => {
                match account::update_associated_key(key, weight) {
                    Ok(()) => {}
                    Err(UpdateKeyFailure::ThresholdViolation) => deferred.push((key, weight)),
                    Err(UpdateKeyFailure::MissingKey) => {
                        revert(Error::from_add_key_failure(add_failure))
                    }
                    Err(failure) => revert(Error::from_update_key_failure(failure))
                }
            }
            Err(failure) => revert(Error::from_add_key_failure(failure))
        }
    }

    match (deployment, key_management) {
        (Some(deployment), Some(key_management)) => {
            match account::set_action_threshold(
                ActionType::KeyManagement,
                Weight::new(key_management)
            ) {
                Ok(()) => set_threshold(ActionType::Deployment, deployment),
                Err(SetThresholdFailure::KeyManagementThreshold) => {
                    set_threshold(ActionType::Deployment, deployment);
                    set_threshold(ActionType::KeyManagement, key_management);
                }
                Err(failure) => revert(Error::from_set_threshold_failure(failure))
            }
        }
        (Some(deployment), None) => set_threshold(ActionType::Deployment, deployment),
        (None, Some(key_management)) => set_threshold(ActionType::KeyManagement, key_management),
        (None, None) => {}
    }

    for (key, weight) in deferred {
        account::update_associated_key(key, weight)
            .unwrap_or_else(|failure| revert(Error::from_update_key_failure(failure)));
    }
    for key in remove_keys {
        account::remove_associated_key(key)
            .unwrap_or_else(|failure| revert(Error::from_remove_key_failure(failure)));
    }
}

/// The checks of the arguments alone, before any change; the same as `plan` makes, except the
/// ones that need the account's state.
fn validate(
    set_keys: &[(AccountHash, u8)],
    remove_keys: &[AccountHash],
    deployment: Option<u8>,
    key_management: Option<u8>
) {
    if set_keys.is_empty()
        && remove_keys.is_empty()
        && deployment.is_none()
        && key_management.is_none()
    {
        revert(Error::EmptyChange);
    }
    if set_keys.iter().any(|(_, weight)| *weight == 0) {
        revert(Error::ZeroWeight);
    }
    if deployment == Some(0) || key_management == Some(0) {
        revert(Error::ZeroThreshold);
    }
    let mut seen = BTreeSet::new();
    let all_keys = set_keys
        .iter()
        .map(|(key, _)| key)
        .chain(remove_keys.iter());
    for key in all_keys {
        if !seen.insert(*key) {
            revert(Error::ConflictingKey);
        }
    }
}

fn set_threshold(action_type: ActionType, threshold: u8) {
    account::set_action_threshold(action_type, Weight::new(threshold))
        .unwrap_or_else(|failure| revert(Error::from_set_threshold_failure(failure)));
}

fn revert(error: Error) -> ! {
    runtime::revert(error)
}
