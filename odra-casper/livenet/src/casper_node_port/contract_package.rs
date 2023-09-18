use odra_types::casper_types::{ContractHash, URef};
use datasize::DataSize;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Contract definition, metadata, and security container.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, DataSize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContractPackage {
    #[data_size(skip)]
    access_key: URef,
    versions: Vec<ContractVersion>,
    disabled_versions: Vec<DisabledVersion>,
    groups: Vec<Groups>,
    lock_status: ContractPackageStatus
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, DataSize, JsonSchema,
)]
pub struct ContractVersion {
    protocol_version_major: u32,
    contract_version: u32,
    contract_hash: ContractHash
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, DataSize, JsonSchema)]
pub struct DisabledVersion {
    protocol_version_major: u32,
    contract_version: u32
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, DataSize, JsonSchema)]
pub struct Groups {
    group: String,
    #[data_size(skip)]
    keys: Vec<URef>
}

/// A enum to determine the lock status of the contract package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, DataSize, JsonSchema)]
pub enum ContractPackageStatus {
    /// The package is locked and cannot be versioned.
    Locked,
    /// The package is unlocked and can be versioned.
    Unlocked
}

impl Default for ContractPackageStatus {
    fn default() -> Self {
        Self::Unlocked
    }
}
