use crate::casper_node_port::contract::Contract;
use odra_core::casper_types::{
    CLValue, EraId, ExecutionResult, ProtocolVersion, PublicKey, Timestamp, U512
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::casper_node_port::hashing::Digest;

use super::{
    account::Account,
    block_hash::{BlockHash, BlockHashAndHeight},
    contract_package::ContractPackage,
    Deploy, DeployHash
};

/// Result for "account_put_deploy" RPC response.
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PutDeployResult {
    /// The RPC API version.
    #[schemars(with = "String")]
    pub api_version: ProtocolVersion,
    /// The deploy hash.
    pub deploy_hash: DeployHash
}

/// Result for "chain_get_state_root_hash" RPC response.
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetStateRootHashResult {
    /// The RPC API version.
    #[schemars(with = "String")]
    pub api_version: ProtocolVersion,
    /// Hex-encoded hash of the state root.
    pub state_root_hash: Option<Digest>
}

/// Params for "info_get_deploy" RPC request.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetDeployParams {
    /// The deploy hash.
    pub deploy_hash: DeployHash,
    /// Whether to return the deploy with the finalized approvals substituted. If `false` or
    /// omitted, returns the deploy with the approvals that were originally received by the node.
    #[serde(default = "finalized_approvals_default")]
    pub finalized_approvals: bool
}

/// The default for `GetDeployParams::finalized_approvals`.
fn finalized_approvals_default() -> bool {
    false
}

/// Result for "info_get_deploy" RPC response.
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetDeployResult {
    /// The RPC API version.
    #[schemars(with = "String")]
    pub api_version: ProtocolVersion,
    /// The deploy.
    pub deploy: Deploy,
    /// The map of block hash to execution result.
    pub execution_results: Vec<JsonExecutionResult>,
    /// The hash and height of the block in which this deploy was executed,
    /// only provided if the full execution results are not know on this node.
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub block_hash_and_height: Option<BlockHashAndHeight>
}

/// The execution result of a single deploy.
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct JsonExecutionResult {
    /// The block hash.
    pub block_hash: BlockHash,
    /// Execution result.
    pub result: ExecutionResult
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
/// Options for dictionary item lookups.
pub enum DictionaryIdentifier {
    /// Lookup a dictionary item via an Account's named keys.
    AccountNamedKey {
        /// The account key as a formatted string whose named keys contains dictionary_name.
        key: String,
        /// The named key under which the dictionary seed URef is stored.
        dictionary_name: String,
        /// The dictionary item key formatted as a string.
        dictionary_item_key: String
    },
    /// Lookup a dictionary item via a Contract's named keys.
    ContractNamedKey {
        /// The contract key as a formatted string whose named keys contains dictionary_name.
        key: String,
        /// The named key under which the dictionary seed URef is stored.
        dictionary_name: String,
        /// The dictionary item key formatted as a string.
        dictionary_item_key: String
    },
    /// Lookup a dictionary item via its seed URef.
    URef {
        /// The dictionary's seed URef.
        seed_uref: String,
        /// The dictionary item key formatted as a string.
        dictionary_item_key: String
    },
    /// Lookup a dictionary item via its unique key.
    Dictionary(String)
}

/// Params for "state_get_dictionary_item" RPC request.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetDictionaryItemParams {
    /// Hash of the state root
    pub state_root_hash: Digest,
    /// The Dictionary query identifier.
    pub dictionary_identifier: DictionaryIdentifier
}

/// Result for "state_get_dictionary_item" RPC response.
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetDictionaryItemResult {
    /// The RPC API version.
    #[schemars(with = "String")]
    pub api_version: ProtocolVersion,
    /// The key under which the value is stored.
    pub dictionary_key: String,
    /// The stored value.
    pub stored_value: StoredValue,
    /// The Merkle proof.
    pub merkle_proof: String
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
#[serde(deny_unknown_fields)]
pub enum GlobalStateIdentifier {
    /// Query using a block hash.
    BlockHash(BlockHash),
    /// Query using a block height.
    BlockHeight(u64),
    /// Query using the state root hash.
    StateRootHash(Digest)
}

/// Params for "query_global_state" RPC
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QueryGlobalStateParams {
    /// The identifier used for the query.
    pub state_identifier: GlobalStateIdentifier,
    /// `casper_types::Key` as formatted string.
    pub key: String,
    /// The path components starting from the key as base.
    #[serde(default)]
    pub path: Vec<String>
}

/// Result for "query_global_state" RPC response.
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QueryGlobalStateResult {
    /// The RPC API version.
    #[schemars(with = "String")]
    pub api_version: ProtocolVersion,
    /// The block header if a Block hash was provided.
    pub block_header: Option<JsonBlockHeader>,
    /// The stored value.
    pub stored_value: StoredValue,
    /// The Merkle proof.
    pub merkle_proof: String
}

/// JSON representation of a block header.
#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct JsonBlockHeader {
    /// The parent hash.
    pub parent_hash: BlockHash,
    /// The state root hash.
    pub state_root_hash: Digest,
    /// The body hash.
    pub body_hash: Digest,
    /// Randomness bit.
    pub random_bit: bool,
    /// Accumulated seed.
    pub accumulated_seed: Digest,
    /// The era end.
    pub era_end: Option<JsonEraEnd>,
    /// The block timestamp.
    pub timestamp: Timestamp,
    /// The block era id.
    pub era_id: EraId,
    /// The block height.
    pub height: u64,
    /// The protocol version.
    pub protocol_version: ProtocolVersion
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct JsonEraEnd {
    era_report: JsonEraReport,
    next_era_validator_weights: Vec<ValidatorWeight>
}

/// Equivocation and reward information to be included in the terminal block.
#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct JsonEraReport {
    equivocators: Vec<PublicKey>,
    rewards: Vec<Reward>,
    inactive_validators: Vec<PublicKey>
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Reward {
    validator: PublicKey,
    amount: u64
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ValidatorWeight {
    validator: PublicKey,
    weight: U512
}

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
pub enum StoredValue {
    /// A CasperLabs value.
    CLValue(CLValue),

    /// An account.
    Account(Account),

    /// A contract definition, metadata, and security container.
    ContractPackage(ContractPackage),

    Contract(Contract)
}
