//! Client for interacting with Casper node.

use crate::casper_client::configuration::CasperClientConfiguration;
use crate::error::LivenetError;
use casper_types::U512;

pub mod accounts;
pub mod configuration;
mod node;
mod queries;
pub mod transaction_watcher;
mod transactions;
mod validators;

/// Environment variable holding a path to a secret key of a main account.
pub const ENV_SECRET_KEY: &str = "ODRA_CASPER_LIVENET_SECRET_KEY_PATH";
/// Environment variable holding an address of the casper node exposing RPC API.
pub const ENV_NODE_ADDRESS: &str = "ODRA_CASPER_LIVENET_NODE_ADDRESS";
/// Environment variable holding the URL of the events stream.
pub const ENV_EVENTS_ADDRESS: &str = "ODRA_CASPER_LIVENET_EVENTS_URL";
/// Environment variable holding a name of the chain.
pub const ENV_CHAIN_NAME: &str = "ODRA_CASPER_LIVENET_CHAIN_NAME";
/// Environment variable holding a filename prefix for additional accounts.
pub const ENV_ACCOUNT_PREFIX: &str = "ODRA_CASPER_LIVENET_KEY_";
/// Environment variable holding cspr.cloud auth token.
pub const ENV_CSPR_CLOUD_AUTH_TOKEN: &str = "CSPR_CLOUD_AUTH_TOKEN";
/// Environment variable holding a path to an additional .env file.
pub const ENV_LIVENET_ENV_FILE: &str = "ODRA_CASPER_LIVENET_ENV";
/// Environment variable holding TTL for transactions.
pub const ENV_TTL: &str = "ODRA_CASPER_LIVENET_TTL";

pub type Result<T> = core::result::Result<T, LivenetError>;

/// Client for interacting with Casper node.
pub struct CasperClient {
    pub configuration: CasperClientConfiguration,
    active_account: usize,
    gas: U512
}

impl CasperClient {
    /// Creates new CasperClient.
    pub fn new(configuration: CasperClientConfiguration) -> Self {
        CasperClient {
            configuration,
            active_account: 0,
            gas: U512::zero()
        }
    }
}

impl Default for CasperClient {
    fn default() -> Self {
        Self::new(CasperClientConfiguration::from_env())
    }
}
