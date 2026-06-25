use crate::casper_client::{
    ENV_ACCOUNT_PREFIX, ENV_CHAIN_NAME, ENV_CSPR_CLOUD_AUTH_TOKEN, ENV_EVENTS_ADDRESS,
    ENV_GAS_PRICE_TOLERANCE, ENV_LIVENET_ENV_FILE, ENV_NODE_ADDRESS, ENV_SECRET_KEY, ENV_TTL
};
use crate::error::LivenetError;
use crate::log;
use crate::utils::{get_env_variable, get_optional_env_variable};
use casper_client::Verbosity;
use casper_types::TimeDiff;
use odra_core::casper_types::SecretKey;
use std::path::PathBuf;

const DEFAULT_TTL: u32 = 5 * 60; // Seconds.
const DEFAULT_GAS_TOLERANCE: u8 = 1;

#[derive(Debug)]
pub struct CasperClientConfiguration {
    pub node_address: String,
    pub events_url: String,
    pub chain_name: String,
    pub secret_keys: Vec<SecretKey>,
    pub secret_key_paths: Vec<String>,
    pub cspr_cloud_auth_token: Option<String>,
    pub gas_price_tolerance: u8,
    pub ttl: u32
}

impl CasperClientConfiguration {
    pub fn from_env() -> Result<Self, LivenetError> {
        // Check for additional .env file
        let additional_env_file = std::env::var(ENV_LIVENET_ENV_FILE);

        if let Ok(additional_env_file) = additional_env_file {
            let filename = PathBuf::from(additional_env_file).with_extension("env");
            dotenv::from_filename(filename).ok();
        }

        // Load .env
        dotenv::dotenv().ok();

        // Initialize logging from environment variable
        log::init_log_level();

        let node_address = get_env_variable(ENV_NODE_ADDRESS)?;
        let chain_name = get_env_variable(ENV_CHAIN_NAME)?;
        let events_url = get_env_variable(ENV_EVENTS_ADDRESS)?;
        let ttl = get_optional_env_variable(ENV_TTL)
            .and_then(|ttl| ttl.parse::<u32>().ok())
            .unwrap_or(DEFAULT_TTL);
        let gas_price_tolerance = get_optional_env_variable(ENV_GAS_PRICE_TOLERANCE)
            .and_then(|ttl| ttl.parse::<u8>().ok())
            .unwrap_or(DEFAULT_GAS_TOLERANCE);

        let (secret_keys, secret_key_paths) = Self::secret_keys_from_env()?;
        Ok(CasperClientConfiguration {
            node_address,
            chain_name,
            secret_keys,
            secret_key_paths,
            cspr_cloud_auth_token: get_optional_env_variable(ENV_CSPR_CLOUD_AUTH_TOKEN),
            events_url,
            gas_price_tolerance,
            ttl
        })
    }

    /// Loads secret keys from ENV_SECRET_KEY file and ENV_ACCOUNT_PREFIX files.
    /// e.g. ENV_SECRET_KEY=secret_key.pem, ENV_ACCOUNT_PREFIX=account_1_key.pem
    /// This will load secret_key.pem as account 0 and account_1_key.pem as account 1.
    fn secret_keys_from_env() -> Result<(Vec<SecretKey>, Vec<String>), LivenetError> {
        let mut secret_keys = vec![];
        let mut secret_key_paths = vec![];
        let file_name = get_env_variable(ENV_SECRET_KEY)?;
        secret_keys.push(
            SecretKey::from_file(file_name.clone()).map_err(LivenetError::SecrectKeyLoadError)?
        );
        secret_key_paths.push(file_name);

        let mut i = 1;
        while let Ok(key_filename) = std::env::var(format!("{}{}", ENV_ACCOUNT_PREFIX, i)) {
            secret_keys.push(
                SecretKey::from_file(&key_filename).map_err(LivenetError::SecrectKeyLoadError)?
            );
            secret_key_paths.push(key_filename);
            i += 1;
        }
        Ok((secret_keys, secret_key_paths))
    }

    pub fn verbosity(&self) -> u64 {
        Verbosity::Low as u64
    }

    pub fn verbosity_typed(&self) -> Verbosity {
        Verbosity::Low
    }

    pub fn ttl(&self) -> TimeDiff {
        TimeDiff::from_seconds(self.ttl)
    }

    pub fn chain_name(&self) -> &str {
        &self.chain_name
    }

    /// Node rpc address.
    pub fn node_address_rpc(&self) -> String {
        format!("{}/rpc", self.node_address())
    }

    /// Node address.
    pub fn node_address(&self) -> &str {
        &self.node_address
    }

    /// Gas price tolerance
    pub fn gas_price_tolerance(&self) -> u8 {
        self.gas_price_tolerance
    }

    pub fn transaction_url(&self, transaction_id: &str) -> Option<String> {
        match self.chain_name.as_str() {
            "casper-test" => Some(format!(
                "https://testnet.cspr.live/transaction/{}",
                transaction_id
            )),
            "casper" => Some(format!("https://cspr.live/transaction/{}", transaction_id)),
            _ => None
        }
    }
}
