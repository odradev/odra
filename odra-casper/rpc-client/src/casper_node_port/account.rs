use datasize::DataSize;
use odra_core::casper_types::{account::AccountHash, URef};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use odra_core::casper_types::addressable_entity::NamedKeys;

/// Structure representing a user's account, stored in global state.
#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize, DataSize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Account {
    account_hash: AccountHash,
    #[data_size(skip)]
    named_keys: NamedKeys,
    #[data_size(skip)]
    main_purse: URef,
    associated_keys: Vec<AssociatedKey>,
    action_thresholds: ActionThresholds
}

impl Account {
    pub fn named_keys(&self) -> impl Iterator<Item = &NamedKey> {
        self.named_keys.iter()
    }
}

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize, DataSize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct AssociatedKey {
    account_hash: AccountHash,
    weight: u8
}

/// Thresholds that have to be met when executing an action of a certain type.
#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize, DataSize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ActionThresholds {
    deployment: u8,
    key_management: u8
}
