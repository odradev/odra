use crate::casper_client::{
    ENV_ACCOUNT_PREFIX, ENV_CHAIN_NAME, ENV_NODE_ADDRESS, ENV_SECRET_KEY
};
use odra_core::casper_types::SecretKey;
use std::path::PathBuf;
use casper_client::JsonRpcId;
use crate::utils::get_env_variable;

#[derive(Debug)]
pub struct CasperClientConfiguration {
    pub node_address: String,
    pub rpc_id: JsonRpcId,
    pub chain_name: String,
    pub secret_keys: Vec<SecretKey>,
    pub secret_key_paths: Vec<String>,
}

impl CasperClientConfiguration {
    pub fn from_env() -> Self {
        // Check for additional .env file
        let additional_env_file = std::env::var("ODRA_CASPER_LIVENET_ENV");

        if let Ok(additional_env_file) = additional_env_file {
            let filename = PathBuf::from(additional_env_file).with_extension("env");
            dotenv::from_filename(filename).ok();
        }

        // Load .env
        dotenv::dotenv().ok();

        let node_address = get_env_variable(ENV_NODE_ADDRESS);
        let chain_name = get_env_variable(ENV_CHAIN_NAME);
        let (secret_keys, secret_key_paths) = Self::secret_keys_from_env();
        CasperClientConfiguration {
            node_address,
            rpc_id: JsonRpcId::String("1".to_string()),
            chain_name,
            secret_keys,
            secret_key_paths
        }
    }

    /// Loads secret keys from ENV_SECRET_KEY file and ENV_ACCOUNT_PREFIX files.
    /// e.g. ENV_SECRET_KEY=secret_key.pem, ENV_ACCOUNT_PREFIX=account_1_key.pem
    /// This will load secret_key.pem as account 0 and account_1_key.pem as account 1.
    fn secret_keys_from_env() -> (Vec<SecretKey>, Vec<String>) {
        let mut secret_keys = vec![];
        let mut secret_key_paths = vec![];
        let file_name = get_env_variable(ENV_SECRET_KEY);
        secret_keys.push(
            SecretKey::from_file(file_name.clone()).unwrap_or_else(|_| {
                panic!(
                    "Couldn't load secret key from file {:?}",
                    get_env_variable(ENV_SECRET_KEY)
                )
            })
        );
        secret_key_paths.push(file_name);

        let mut i = 1;
        while let Ok(key_filename) = std::env::var(format!("{}{}", ENV_ACCOUNT_PREFIX, i)) {
            secret_keys.push(SecretKey::from_file(&key_filename).unwrap_or_else(|_| {
                panic!("Couldn't load secret key from file {:?}", key_filename)
            }));
            secret_key_paths.push(key_filename);
            i += 1;
        }
        (secret_keys, secret_key_paths)
    }
}
