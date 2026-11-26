use alloc::{collections::BTreeSet, vec::Vec};

use casper_types::account::AccountHash;

use crate::{AccountChange, AccountState, MAX_ASSOCIATED_KEYS};

/// One of the two action thresholds of an account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThresholdKind {
    /// The weight needed to send a transaction.
    Deployment,
    /// The weight needed to change keys and thresholds.
    KeyManagement
}

/// Why a change cannot be applied. The node would reject each of these, so a tool stops before
/// sending the transaction instead of spending gas on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// The change asks for nothing.
    EmptyChange,
    /// A key to set has weight 0; it should be removed instead.
    ZeroWeight(AccountHash),
    /// A threshold of 0 was requested.
    ZeroThreshold(ThresholdKind),
    /// A key is both set and removed.
    SetAndRemoved(AccountHash),
    /// A key to remove is not an associated key.
    RemoveMissingKey(AccountHash),
    /// The account would have more associated keys than the chain allows.
    TooManyKeys {
        /// Keys after the change.
        count: usize
    },
    /// A threshold would be above the total weight of all keys, so nobody could reach it.
    ThresholdAboveTotalWeight {
        /// Which threshold.
        kind: ThresholdKind,
        /// The threshold after the change.
        threshold: u8,
        /// The total weight after the change.
        total_weight: u8
    },
    /// The `deployment` threshold would be above the `key_management` threshold.
    DeploymentAboveKeyManagement {
        /// `deployment` after the change.
        deployment: u8,
        /// `key_management` after the change.
        key_management: u8
    }
}

/// Something about the change the person sending it should confirm. None of these make the
/// transaction fail; they can make the account harder or impossible to use afterwards.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Warning {
    /// One of the signer's own keys is removed.
    OwnKeyRemoved(AccountHash),
    /// After the change, the signer's own keys together do not reach the threshold: they can no
    /// longer send transactions (`Deployment`) or change keys (`KeyManagement`) on their own.
    OwnKeysBelowThreshold {
        /// Which threshold.
        kind: ThresholdKind,
        /// The weight of the own keys after the change.
        weight: u8,
        /// The threshold after the change.
        threshold: u8
    },
    /// The total weight equals `key_management`: losing any key with weight locks key
    /// management for good.
    NoSpareKeyManagementWeight {
        /// The total weight and `key_management` after the change.
        weight: u8
    },
    /// The signer's own keys do not reach the account's current `key_management` threshold,
    /// so the transaction fails unless more associated keys sign it.
    NeedsMoreSignatures {
        /// The weight of the own keys now.
        weight: u8,
        /// The current `key_management` threshold.
        required: u8
    }
}

/// A checked change: the configuration after it and what to warn about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// The configuration before the change.
    pub before: AccountState,
    /// The configuration after the change.
    pub after: AccountState,
    /// Warnings, in a stable order.
    pub warnings: Vec<Warning>
}

/// Checks `change` against the account's current configuration `state`.
///
/// `own_keys` are the keys of whoever sends the change (the keys that will sign it); they are
/// used only for warnings. Every check runs on the configuration after the change, so raising
/// both thresholds at once is fine as long as the result is valid.
pub fn plan(
    state: &AccountState,
    change: &AccountChange,
    own_keys: &BTreeSet<AccountHash>
) -> Result<Plan, ValidationError> {
    if change.is_empty() {
        return Err(ValidationError::EmptyChange);
    }
    if let Some((key, _)) = change.set_keys.iter().find(|(_, weight)| **weight == 0) {
        return Err(ValidationError::ZeroWeight(*key));
    }
    if change.deployment_threshold == Some(0) {
        return Err(ValidationError::ZeroThreshold(ThresholdKind::Deployment));
    }
    if change.key_management_threshold == Some(0) {
        return Err(ValidationError::ZeroThreshold(ThresholdKind::KeyManagement));
    }
    if let Some(key) = change
        .remove_keys
        .iter()
        .find(|key| change.set_keys.contains_key(key))
    {
        return Err(ValidationError::SetAndRemoved(*key));
    }
    if let Some(key) = change
        .remove_keys
        .iter()
        .find(|key| !state.associated_keys.contains_key(key))
    {
        return Err(ValidationError::RemoveMissingKey(*key));
    }

    let mut after = state.clone();
    for (key, weight) in &change.set_keys {
        after.associated_keys.insert(*key, *weight);
    }
    for key in &change.remove_keys {
        after.associated_keys.remove(key);
    }
    if let Some(threshold) = change.deployment_threshold {
        after.deployment = threshold;
    }
    if let Some(threshold) = change.key_management_threshold {
        after.key_management = threshold;
    }

    if after.associated_keys.len() > MAX_ASSOCIATED_KEYS {
        return Err(ValidationError::TooManyKeys {
            count: after.associated_keys.len()
        });
    }
    let total_weight = after.total_weight();
    for (kind, threshold) in [
        (ThresholdKind::Deployment, after.deployment),
        (ThresholdKind::KeyManagement, after.key_management)
    ] {
        if threshold > total_weight {
            return Err(ValidationError::ThresholdAboveTotalWeight {
                kind,
                threshold,
                total_weight
            });
        }
    }
    if after.deployment > after.key_management {
        return Err(ValidationError::DeploymentAboveKeyManagement {
            deployment: after.deployment,
            key_management: after.key_management
        });
    }

    let mut warnings = Vec::new();
    // The engine checks every key management call against the account as it was when the
    // transaction started (its runtime footprint), not as earlier calls left it.
    let signing_weight = state.weight_of(own_keys);
    if signing_weight < state.key_management {
        warnings.push(Warning::NeedsMoreSignatures {
            weight: signing_weight,
            required: state.key_management
        });
    }
    for key in own_keys
        .iter()
        .filter(|key| change.remove_keys.contains(key))
    {
        warnings.push(Warning::OwnKeyRemoved(*key));
    }
    let own_weight = after.weight_of(own_keys);
    for (kind, threshold) in [
        (ThresholdKind::Deployment, after.deployment),
        (ThresholdKind::KeyManagement, after.key_management)
    ] {
        if own_weight < threshold {
            warnings.push(Warning::OwnKeysBelowThreshold {
                kind,
                weight: own_weight,
                threshold
            });
        }
    }
    if total_weight == after.key_management {
        warnings.push(Warning::NoSpareKeyManagementWeight {
            weight: total_weight
        });
    }

    Ok(Plan {
        before: state.clone(),
        after,
        warnings
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn key(n: u8) -> AccountHash {
        AccountHash::new([n; 32])
    }

    fn own(keys: &[u8]) -> BTreeSet<AccountHash> {
        keys.iter().map(|n| key(*n)).collect()
    }

    /// Keys 1, 2 and 3 with weight 1 each, deployment 1, key_management 2.
    fn two_of_three() -> AccountState {
        let mut state = AccountState::new(key(1));
        state.associated_keys.insert(key(2), 1);
        state.associated_keys.insert(key(3), 1);
        state.key_management = 2;
        state
    }

    #[test]
    fn adds_a_key_and_raises_both_thresholds_in_one_change() {
        let change = AccountChange::new()
            .set_key(key(2), 1)
            .deployment(2)
            .key_management(2);
        let plan = plan(&AccountState::new(key(1)), &change, &own(&[1])).unwrap();
        assert_eq!(plan.after.associated_keys.len(), 2);
        assert_eq!((plan.after.deployment, plan.after.key_management), (2, 2));
        assert_eq!(
            plan.warnings,
            vec![
                Warning::OwnKeysBelowThreshold {
                    kind: ThresholdKind::Deployment,
                    weight: 1,
                    threshold: 2
                },
                Warning::OwnKeysBelowThreshold {
                    kind: ThresholdKind::KeyManagement,
                    weight: 1,
                    threshold: 2
                },
                Warning::NoSpareKeyManagementWeight { weight: 2 }
            ]
        );
    }

    #[test]
    fn a_change_that_keeps_own_control_has_no_warnings() {
        let change = AccountChange::new().set_key(key(1), 3).set_key(key(2), 1);
        let plan = plan(&AccountState::new(key(1)), &change, &own(&[1])).unwrap();
        assert!(plan.warnings.is_empty());
        assert_eq!(plan.after.total_weight(), 4);
    }

    #[test]
    fn warns_when_own_key_is_removed() {
        let change = AccountChange::new().remove_key(key(1));
        let plan = plan(&two_of_three(), &change, &own(&[1, 2])).unwrap();
        assert_eq!(
            plan.warnings,
            vec![
                Warning::OwnKeyRemoved(key(1)),
                Warning::OwnKeysBelowThreshold {
                    kind: ThresholdKind::KeyManagement,
                    weight: 1,
                    threshold: 2
                },
                Warning::NoSpareKeyManagementWeight { weight: 2 }
            ]
        );
    }

    #[test]
    fn warns_when_own_keys_cannot_sign_the_change() {
        let change = AccountChange::new().set_key(key(4), 1);
        let plan = plan(&two_of_three(), &change, &own(&[1])).unwrap();
        assert_eq!(
            plan.warnings[0],
            Warning::NeedsMoreSignatures {
                weight: 1,
                required: 2
            }
        );
    }

    #[test]
    fn threshold_above_total_weight_is_an_error() {
        let change = AccountChange::new().key_management(4);
        assert_eq!(
            plan(&two_of_three(), &change, &own(&[1, 2])),
            Err(ValidationError::ThresholdAboveTotalWeight {
                kind: ThresholdKind::KeyManagement,
                threshold: 4,
                total_weight: 3
            })
        );
        // Removing a key can make the existing threshold unreachable too.
        let change = AccountChange::new().remove_key(key(2)).remove_key(key(3));
        assert_eq!(
            plan(&two_of_three(), &change, &own(&[1, 2])),
            Err(ValidationError::ThresholdAboveTotalWeight {
                kind: ThresholdKind::KeyManagement,
                threshold: 2,
                total_weight: 1
            })
        );
    }

    #[test]
    fn deployment_above_key_management_is_an_error() {
        let change = AccountChange::new().deployment(3);
        assert_eq!(
            plan(&two_of_three(), &change, &own(&[1, 2])),
            Err(ValidationError::DeploymentAboveKeyManagement {
                deployment: 3,
                key_management: 2
            })
        );
        // Fine once key_management goes up in the same change.
        let change = AccountChange::new().deployment(3).key_management(3);
        assert!(plan(&two_of_three(), &change, &own(&[1, 2, 3])).is_ok());
    }

    #[test]
    fn invalid_arguments_are_errors() {
        let state = two_of_three();
        let owners = own(&[1, 2]);
        assert_eq!(
            plan(&state, &AccountChange::new(), &owners),
            Err(ValidationError::EmptyChange)
        );
        assert_eq!(
            plan(&state, &AccountChange::new().set_key(key(4), 0), &owners),
            Err(ValidationError::ZeroWeight(key(4)))
        );
        assert_eq!(
            plan(&state, &AccountChange::new().deployment(0), &owners),
            Err(ValidationError::ZeroThreshold(ThresholdKind::Deployment))
        );
        assert_eq!(
            plan(&state, &AccountChange::new().key_management(0), &owners),
            Err(ValidationError::ZeroThreshold(ThresholdKind::KeyManagement))
        );
        assert_eq!(
            plan(
                &state,
                &AccountChange::new().set_key(key(2), 2).remove_key(key(2)),
                &owners
            ),
            Err(ValidationError::SetAndRemoved(key(2)))
        );
        assert_eq!(
            plan(&state, &AccountChange::new().remove_key(key(9)), &owners),
            Err(ValidationError::RemoveMissingKey(key(9)))
        );
    }

    #[test]
    fn too_many_keys_is_an_error() {
        let mut change = AccountChange::new();
        for n in 2..=101u8 {
            change = change.set_key(key(n), 1);
        }
        assert_eq!(
            plan(&AccountState::new(key(1)), &change, &own(&[1])),
            Err(ValidationError::TooManyKeys { count: 101 })
        );
    }

    #[test]
    fn total_weight_saturates_like_the_engine() {
        let mut state = AccountState::new(key(1));
        state.associated_keys.insert(key(1), 200);
        state.associated_keys.insert(key(2), 200);
        assert_eq!(state.total_weight(), 255);
        assert_eq!(state.weight_of([&key(2), &key(2), &key(9)]), 200);
    }

    #[test]
    fn runtime_args_carry_every_field() {
        use crate::{
            DEPLOYMENT_THRESHOLD_ARG, KEY_MANAGEMENT_THRESHOLD_ARG, REMOVE_KEYS_ARG, SET_KEYS_ARG
        };
        let args = AccountChange::new()
            .set_key(key(2), 3)
            .remove_key(key(3))
            .deployment(2)
            .to_runtime_args();
        let set_keys: Vec<(AccountHash, u8)> =
            args.get(SET_KEYS_ARG).unwrap().clone().into_t().unwrap();
        assert_eq!(set_keys, vec![(key(2), 3)]);
        let remove_keys: Vec<AccountHash> =
            args.get(REMOVE_KEYS_ARG).unwrap().clone().into_t().unwrap();
        assert_eq!(remove_keys, vec![key(3)]);
        let deployment: Option<u8> = args
            .get(DEPLOYMENT_THRESHOLD_ARG)
            .unwrap()
            .clone()
            .into_t()
            .unwrap();
        assert_eq!(deployment, Some(2));
        let key_management: Option<u8> = args
            .get(KEY_MANAGEMENT_THRESHOLD_ARG)
            .unwrap()
            .clone()
            .into_t()
            .unwrap();
        assert_eq!(key_management, None);
    }
}
