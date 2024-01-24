use odra_core::casper_types::{
    ContractPackageHash, ContractWasmHash, EntryPoint, NamedKey, ProtocolVersion
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Details of a smart contract.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Contract {
    contract_package_hash: ContractPackageHash,
    contract_wasm_hash: ContractWasmHash,
    named_keys: Vec<NamedKey>,
    entry_points: Vec<EntryPoint>,
    protocol_version: ProtocolVersion
}

impl Contract {
    /// Returns the contract package hash of the contract.
    pub fn contract_package_hash(&self) -> &ContractPackageHash {
        &self.contract_package_hash
    }

    /// Returns the contract Wasm hash of the contract.
    pub fn contract_wasm_hash(&self) -> &ContractWasmHash {
        &self.contract_wasm_hash
    }

    /// Returns the named keys of the contract.
    pub fn named_keys(&self) -> impl Iterator<Item = &NamedKey> {
        self.named_keys.iter()
    }

    /// Returns the entry-points of the contract.
    pub fn entry_points(&self) -> impl Iterator<Item = &EntryPoint> {
        self.entry_points.iter()
    }

    /// Returns the protocol version of the contract.
    pub fn protocol_version(&self) -> &ProtocolVersion {
        &self.protocol_version
    }
}
