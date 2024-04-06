//! Livenet implementation of HostContext for HostEnv.
use std::path::PathBuf;
use std::sync::RwLock;
use std::thread::sleep;

use casper_types::{
    bytesrepr::{Bytes, ToBytes},
    PublicKey, RuntimeArgs, U512
};
use casper_types::SecretKey::Ed25519;
use odra_casper_rpc_client::casper_client::{CasperClient, CasperClientConfiguration};
use odra_casper_rpc_client::log::info;
use odra_core::callstack::{Callstack, CallstackElement};
use odra_core::casper_types::{SecretKey, Timestamp};
use odra_core::entry_point_callback::EntryPointsCaller;
use odra_core::{host::HostContext, Address, CallDef, ContractEnv, GasReport, OdraError};
use odra_core::{prelude::*, EventError};
use odra_core::{ContractContainer, ContractRegister};
use tokio::runtime::Runtime;

use crate::livenet_contract_env::LivenetContractEnv;

/// Environment variable holding a path to a secret key of a main account.
pub const ENV_SECRET_KEY: &str = "ODRA_CASPER_LIVENET_SECRET_KEY_PATH";
/// Environment variable holding an address of the casper node exposing RPC API.
pub const ENV_NODE_ADDRESS: &str = "ODRA_CASPER_LIVENET_NODE_ADDRESS";
/// Environment variable holding a name of the chain.
pub const ENV_CHAIN_NAME: &str = "ODRA_CASPER_LIVENET_CHAIN_NAME";
/// Environment variable holding a filename prefix for additional accounts.
pub const ENV_ACCOUNT_PREFIX: &str = "ODRA_CASPER_LIVENET_KEY_";

fn get_env_variable(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|err| {
        odra_casper_rpc_client::log::error(format!(
            "{} must be set. Have you setup your .env file?",
            name
        ));
        panic!("{}", err)
    })
}

/// LivenetHost struct.
pub struct LivenetHost {
    casper_client: Rc<RefCell<CasperClient>>,
    contract_register: Rc<RwLock<ContractRegister>>,
    contract_env: Rc<ContractEnv>,
    callstack: Rc<RefCell<Callstack>>
}

/// Creates a new instance of CasperClientConfiguration
/// by loading configuration from environment variables.
pub fn casper_client_configuration() -> CasperClientConfiguration {
    // Check for additional .env file
    let additional_env_file = std::env::var("ODRA_CASPER_LIVENET_ENV");

    if let Ok(additional_env_file) = additional_env_file {
        let filename = PathBuf::from(additional_env_file).with_extension("env");
        dotenv::from_filename(filename).ok();
    }

    // Load .env
    dotenv::dotenv().ok();

    let secret_keys = load_secret_keys();
    match &secret_keys[0] {
        Ed25519(key) => dbg!(key.as_bytes()),
        _ => panic!(""),
    };

    let node_address = get_env_variable(ENV_NODE_ADDRESS);
    let chain_name = get_env_variable(ENV_CHAIN_NAME);

    CasperClientConfiguration {
        node_address,
        chain_name,
        secret_keys
    }
}

/// Loads secret keys from ENV_SECRET_KEY file and ENV_ACCOUNT_PREFIX files.
/// e.g. ENV_SECRET_KEY=secret_key.pem, ENV_ACCOUNT_PREFIX=account_1_key.pem
/// This will load secret_key.pem as an account 0 and account_1_key.pem as account 1.
pub fn load_secret_keys() -> Vec<SecretKey> {
    let mut secret_keys = vec![];
    secret_keys.push(
        SecretKey::from_file(get_env_variable(ENV_SECRET_KEY)).unwrap_or_else(|_| {
            panic!(
                "Couldn't load secret key from file {:?}",
                get_env_variable(ENV_SECRET_KEY)
            )
        })
    );

    let mut i = 1;
    while let Ok(key_filename) = std::env::var(format!("{}{}", ENV_ACCOUNT_PREFIX, i)) {
        secret_keys.push(
            SecretKey::from_file(&key_filename).unwrap_or_else(|_| {
                panic!("Couldn't load secret key from file {:?}", key_filename)
            })
        );
        i += 1;
    }
    secret_keys
}

impl LivenetHost {
    /// Creates a new instance of LivenetHost.
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self::new_instance()))
    }

    fn new_instance() -> Self {
        let client_config = casper_client_configuration();
        let casper_client: Rc<RefCell<CasperClient>> =
            Rc::new(RefCell::new(CasperClient::new(client_config)));
        let callstack: Rc<RefCell<Callstack>> = Default::default();
        let contract_register = Rc::new(RwLock::new(Default::default()));
        let livenet_contract_env = LivenetContractEnv::new(
            casper_client.clone(),
            callstack.clone(),
            contract_register.clone()
        );
        let contract_env = Rc::new(ContractEnv::new(0, livenet_contract_env));
        Self {
            casper_client,
            contract_register,
            contract_env,
            callstack
        }
    }

    /// Creates a new instance of LivenetHost with a given configuration.
    pub fn new_instance_with_config(config: CasperClientConfiguration) -> Self {
        let casper_client: Rc<RefCell<CasperClient>> =
            Rc::new(RefCell::new(CasperClient::new(config)));
        let callstack: Rc<RefCell<Callstack>> = Default::default();
        let contract_register = Rc::new(RwLock::new(Default::default()));
        let livenet_contract_env = LivenetContractEnv::new(
            casper_client.clone(),
            callstack.clone(),
            contract_register.clone()
        );
        let contract_env = Rc::new(ContractEnv::new(0, livenet_contract_env));
        Self {
            casper_client,
            contract_register,
            contract_env,
            callstack
        }
    }
}

impl HostContext for LivenetHost {
    fn set_caller(&self, caller: Address) {
        self.casper_client.borrow_mut().set_caller(caller);
    }

    fn set_gas(&self, gas: u64) {
        self.casper_client.borrow_mut().set_gas(gas);
    }

    fn caller(&self) -> Address {
        self.casper_client.borrow().caller()
    }

    fn get_account(&self, index: usize) -> Address {
        self.casper_client.borrow().get_account(index)
    }

    fn balance_of(&self, address: &Address) -> U512 {
        let rt = Runtime::new().unwrap();
        let client = self.casper_client.borrow();
        rt.block_on(async { client.get_balance(address).await })
    }

    fn block_time(&self) -> u64 {
        let rt = Runtime::new().unwrap();
        let client = self.casper_client.borrow();
        rt.block_on(async { client.get_block_time().await })
    }

    fn advance_block_time(&self, time_diff: u64) {
        info(format!(
            "advance_block_time called - Waiting for {} ms",
            time_diff
        ));
        sleep(std::time::Duration::from_millis(time_diff));
    }

    fn get_event(&self, contract_address: &Address, index: u32) -> Result<Bytes, EventError> {
        let rt = Runtime::new().unwrap();
        let cleint = self.casper_client.borrow();
        rt.block_on(async { cleint.get_event(contract_address, index).await })
            .map_err(|_| EventError::CouldntExtractEventData)
    }

    fn get_events_count(&self, contract_address: &Address) -> u32 {
        let rt = Runtime::new().unwrap();
        let client = self.casper_client.borrow();
        rt.block_on(async { client.events_count(contract_address).await })
    }

    fn call_contract(
        &self,
        address: &Address,
        call_def: CallDef,
        use_proxy: bool
    ) -> Result<Bytes, OdraError> {
        if !call_def.is_mut() {
            self.callstack
                .borrow_mut()
                .push(CallstackElement::new_contract_call(
                    *address,
                    call_def.clone()
                ));
            let result = self
                .contract_register
                .read()
                .expect("Couldn't read contract register.")
                .call(address, call_def);
            self.callstack.borrow_mut().pop();
            return result;
        }
        let timestamp = Timestamp::now().millis();
        let rt = Runtime::new().unwrap();
        let client = self.casper_client.borrow_mut();
        match use_proxy {
            true => rt.block_on(async {
                client
                    .deploy_entrypoint_call_with_proxy(*address, call_def, timestamp)
                    .await
            }),
            false => {
                let rt = Runtime::new().unwrap();
                rt.block_on(async {
                    client
                        .deploy_entrypoint_call(*address, call_def, timestamp)
                        .await
                });
                Ok(
                    ().to_bytes()
                        .expect("Couldn't serialize (). This shouldn't happen.")
                        .into()
                )
            }
        }
    }

    fn register_contract(&self, address: Address, entry_points_caller: EntryPointsCaller) {
        self.contract_register
            .write()
            .expect("Couldn't write contract register.")
            .add(address, ContractContainer::new(entry_points_caller));
    }

    fn new_contract(
        &self,
        name: &str,
        init_args: RuntimeArgs,
        entry_points_caller: EntryPointsCaller
    ) -> Address {
        let timestamp = Timestamp::now();
        let client = self.casper_client.borrow_mut();
        let rt = Runtime::new().unwrap();
        let address = rt.block_on(async {
            client
                .deploy_wasm(name, init_args, timestamp.millis())
                .await
        });
        self.register_contract(address, entry_points_caller);
        address
    }

    fn contract_env(&self) -> ContractEnv {
        (*self.contract_env).clone()
    }

    fn gas_report(&self) -> GasReport {
        println!("Gas report is unavailable for livenet");
        todo!()
    }

    fn last_call_gas_cost(&self) -> u64 {
        // Todo: implement
        0
    }

    fn sign_message(&self, message: &Bytes, address: &Address) -> Bytes {
        self.casper_client
            .borrow()
            .sign_message(message, address)
            .unwrap()
    }

    fn public_key(&self, address: &Address) -> PublicKey {
        self.casper_client.borrow().address_public_key(address)
    }
}

#[cfg(test)]
mod test {
    use casper_types::SecretKey;
    use casper_types::SecretKey::Ed25519;

    #[test]
    fn test_loading_secret_keys() {
        std::env::set_var(
            "ODRA_CASPER_LIVENET_SECRET_KEY_PATH",
            "../../.keys/secret_key.pem"
        );
        let secret_keys = super::load_secret_keys();
        assert_eq!(secret_keys.len(), 1);

        let key_str = "-----BEGIN PRIVATE KEY-----
MC4CAQAwBQYDK2VwBCIEIHRZr1HEgKVbgchuatwA7dCWDWB7QZe+bpDb5dguIyLE
-----END PRIVATE KEY-----";

        assert_eq!(
            secret_keys[0].to_der().unwrap(),
            secret_key.to_der().unwrap()
        );
    }
}
