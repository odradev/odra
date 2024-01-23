use crate::casper_node_port::rpcs::GlobalStateIdentifier;
use odra_core::casper_types::account::AccountHash;
use odra_core::casper_types::{ProtocolVersion, URef};
use odra_core::{PublicKey, U512};
use serde::{Deserialize, Serialize};

pub(crate) const QUERY_BALANCE_METHOD: &str = "query_balance";

/// Identifier of a purse.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum PurseIdentifier {
    /// The main purse of the account identified by this public key.
    MainPurseUnderPublicKey(PublicKey),
    /// The main purse of the account identified by this account hash.
    MainPurseUnderAccountHash(AccountHash),
    /// The purse identified by this URef.
    PurseUref(URef)
}

/// Params for "query_balance" RPC request.
#[derive(Serialize, Deserialize, Debug)]
pub struct QueryBalanceParams {
    /// The state identifier used for the query.
    pub state_identifier: Option<GlobalStateIdentifier>,
    /// The identifier to obtain the purse corresponding to balance query.
    pub purse_identifier: PurseIdentifier
}

impl QueryBalanceParams {
    pub(crate) fn new(
        state_identifier: Option<GlobalStateIdentifier>,
        purse_identifier: PurseIdentifier
    ) -> Self {
        QueryBalanceParams {
            state_identifier,
            purse_identifier
        }
    }
}

/// Result for "query_balance" RPC response.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QueryBalanceResult {
    /// The RPC API version.
    pub api_version: ProtocolVersion,
    /// The balance represented in motes.
    pub balance: U512
}
