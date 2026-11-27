use alloc::{
    collections::{BTreeMap, BTreeSet},
    vec::Vec
};

use casper_types::{account::AccountHash, RuntimeArgs};

use crate::{
    DEPLOYMENT_THRESHOLD_ARG, KEY_MANAGEMENT_THRESHOLD_ARG, REMOVE_KEYS_ARG, SET_KEYS_ARG
};

/// The multisig configuration of an account: its associated keys with their weights, and its
/// two action thresholds.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AccountState {
    /// Associated keys and their weights.
    pub associated_keys: BTreeMap<AccountHash, u8>,
    /// The weight the signers of a transaction need to reach to send it.
    pub deployment: u8,
    /// The weight the signers need to reach to change keys and thresholds.
    pub key_management: u8
}

impl AccountState {
    /// A new account: its own key with weight 1 and both thresholds at 1.
    pub fn new(account_hash: AccountHash) -> Self {
        let mut associated_keys = BTreeMap::new();
        associated_keys.insert(account_hash, 1);
        AccountState {
            associated_keys,
            deployment: 1,
            key_management: 1
        }
    }

    /// The total weight of all associated keys. Like the execution engine, the sum saturates
    /// at 255.
    pub fn total_weight(&self) -> u8 {
        self.weight_of(self.associated_keys.keys())
    }

    /// The weight the given keys bring together. Keys that are not associated count 0; the sum
    /// saturates at 255.
    pub fn weight_of<'a>(&self, keys: impl IntoIterator<Item = &'a AccountHash>) -> u8 {
        keys.into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .filter_map(|key| self.associated_keys.get(key))
            .fold(0u8, |sum, weight| sum.saturating_add(*weight))
    }
}

/// A change of an account's multisig configuration, applied in one transaction by the account
/// manager wasm.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AccountChange {
    /// Keys to add, or whose weight to change.
    pub set_keys: BTreeMap<AccountHash, u8>,
    /// Keys to remove.
    pub remove_keys: BTreeSet<AccountHash>,
    /// The new `deployment` threshold, if it changes.
    pub deployment_threshold: Option<u8>,
    /// The new `key_management` threshold, if it changes.
    pub key_management_threshold: Option<u8>
}

impl AccountChange {
    /// No change yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds the key, or changes its weight.
    pub fn set_key(mut self, key: AccountHash, weight: u8) -> Self {
        self.set_keys.insert(key, weight);
        self
    }

    /// Removes the key.
    pub fn remove_key(mut self, key: AccountHash) -> Self {
        self.remove_keys.insert(key);
        self
    }

    /// Sets the `deployment` threshold.
    pub fn deployment(mut self, threshold: u8) -> Self {
        self.deployment_threshold = Some(threshold);
        self
    }

    /// Sets the `key_management` threshold.
    pub fn key_management(mut self, threshold: u8) -> Self {
        self.key_management_threshold = Some(threshold);
        self
    }

    /// Whether the change asks for nothing.
    pub fn is_empty(&self) -> bool {
        self.set_keys.is_empty()
            && self.remove_keys.is_empty()
            && self.deployment_threshold.is_none()
            && self.key_management_threshold.is_none()
    }

    /// Reads the change back from the runtime arguments of the account manager wasm, for a
    /// signer to see what a transaction does. `None` if an argument is missing or has another
    /// type.
    pub fn from_runtime_args(args: &RuntimeArgs) -> Option<Self> {
        let set_keys: Vec<(AccountHash, u8)> = args.get(SET_KEYS_ARG)?.clone().into_t().ok()?;
        let remove_keys: Vec<AccountHash> = args.get(REMOVE_KEYS_ARG)?.clone().into_t().ok()?;
        let deployment_threshold: Option<u8> =
            args.get(DEPLOYMENT_THRESHOLD_ARG)?.clone().into_t().ok()?;
        let key_management_threshold: Option<u8> = args
            .get(KEY_MANAGEMENT_THRESHOLD_ARG)?
            .clone()
            .into_t()
            .ok()?;
        Some(AccountChange {
            set_keys: set_keys.into_iter().collect(),
            remove_keys: remove_keys.into_iter().collect(),
            deployment_threshold,
            key_management_threshold
        })
    }

    /// The runtime arguments of the account manager wasm.
    pub fn to_runtime_args(&self) -> RuntimeArgs {
        let set_keys: Vec<(AccountHash, u8)> = self
            .set_keys
            .iter()
            .map(|(key, weight)| (*key, *weight))
            .collect();
        let remove_keys: Vec<AccountHash> = self.remove_keys.iter().copied().collect();
        let mut args = RuntimeArgs::new();
        // Inserting values of these types cannot fail: they all have a CLType and serialize.
        args.insert(SET_KEYS_ARG, set_keys)
            .expect("set_keys serializes");
        args.insert(REMOVE_KEYS_ARG, remove_keys)
            .expect("remove_keys serializes");
        args.insert(DEPLOYMENT_THRESHOLD_ARG, self.deployment_threshold)
            .expect("deployment_threshold serializes");
        args.insert(KEY_MANAGEMENT_THRESHOLD_ARG, self.key_management_threshold)
            .expect("key_management_threshold serializes");
        args
    }
}
