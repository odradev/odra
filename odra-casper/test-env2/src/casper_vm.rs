use std::env;
use std::path::PathBuf;
use casper_contract::contract_api::runtime;
use casper_contract::contract_api::runtime::revert;
use casper_contract::contract_api::system::{create_purse, transfer_from_purse_to_account};
use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_engine_test_support::{DEFAULT_ACCOUNT_INITIAL_BALANCE, DEFAULT_CHAINSPEC_REGISTRY, DEFAULT_GENESIS_CONFIG, DEFAULT_GENESIS_CONFIG_HASH, DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder};
use casper_execution_engine::core::engine_state::{GenesisAccount, RunGenesisRequest};
use odra_core::{CallDef, ContractContext, HostContext, InitializeBackend, ModuleCaller, OdraResult};
use odra_types::{Address, EventData, ExecutionError, OdraError, PublicKey, U512};
use odra::prelude::collections::BTreeMap;
use odra_casper_shared::consts;
use odra_types::casper_types::account::AccountHash;
use odra_types::casper_types::{BlockTime, Motes, runtime_args, SecretKey};

pub struct CasperVm {
    accounts: Vec<Address>,
    key_pairs: BTreeMap<Address, (SecretKey, PublicKey)>,
    active_account: Address,
    context: InMemoryWasmTestBuilder,
    block_time: BlockTime,
    calls_counter: u32,
    error: Option<OdraError>,
    attached_value: Option<U512>,
    gas_used: BTreeMap<AccountHash, U512>,
    gas_cost: Vec<(String, U512)>
}

impl InitializeBackend for CasperVm {
    /// Create a new instance with predefined accounts.
    fn new() -> Self {
        let mut genesis_config = DEFAULT_GENESIS_CONFIG.clone();
        let mut accounts: Vec<Address> = Vec::new();
        let mut key_pairs = BTreeMap::new();
        for i in 0..20 {
            // Create keypair.
            let secret_key = SecretKey::ed25519_from_bytes([i; 32]).unwrap();
            let public_key = PublicKey::from(&secret_key);

            // Create an AccountHash from a public key.
            let account_addr = AccountHash::from(&public_key);

            // Create a GenesisAccount.
            let account = GenesisAccount::account(
                public_key.clone(),
                Motes::new(U512::from(DEFAULT_ACCOUNT_INITIAL_BALANCE)),
                None
            );
            genesis_config.ee_config_mut().push_account(account);

            accounts.push(account_addr.try_into().unwrap());
            key_pairs.insert(account_addr.try_into().unwrap(), (secret_key, public_key));
        }
        let run_genesis_request = RunGenesisRequest::new(
            *DEFAULT_GENESIS_CONFIG_HASH,
            genesis_config.protocol_version(),
            genesis_config.take_ee_config(),
            DEFAULT_CHAINSPEC_REGISTRY.clone()
        );

        let chainspec_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/chainspec.toml");
        let mut builder = InMemoryWasmTestBuilder::new_with_chainspec(chainspec_path, None);

        builder.run_genesis(&run_genesis_request).commit();

        Self {
            active_account: accounts[0],
            context: builder,
            accounts,
            block_time: BlockTime::new(0),
            calls_counter: 0,
            error: None,
            attached_value: None,
            gas_used: BTreeMap::new(),
            gas_cost: Vec::new(),
            key_pairs
        }
    }
}

impl HostContext for CasperVm {
    fn set_caller(&self, caller: Address) {
        todo!()
    }

    fn get_account(&self, index: usize) -> Address {
        todo!()
    }

    fn advance_block_time(&self, time_diff: u64) {
        todo!()
    }

    fn get_event(&self, contract_address: Address, index: i32) -> Option<odra_types::EventData> {
        todo!()
    }

    fn attach_value(&self, amount: U512) {
        todo!()
    }
}

impl ContractContext for CasperVm {
    fn get(&self, key: Vec<u8>) -> Option<Vec<u8>> {
        todo!()
    }

    fn set(&self, key: Vec<u8>, value: Vec<u8>) {
        todo!()
    }

    fn get_caller(&self) -> Address {
        todo!()
    }

    fn call_contract(&mut self, address: Address, call_def: CallDef) -> OdraResult<Vec<u8>> {
        self.error = None;
        // TODO: handle unwrap
        let hash = address.as_contract_package_hash().unwrap();

        let session_code = include_bytes!("../resources/proxy_caller_with_return.wasm").to_vec();
        let args = runtime_args! {
            // TODO: convert call_def to RuntimeArgs
        };
        let args_bytes: Vec<u8> = args.to_bytes().unwrap();
        let args = runtime_args! {
            CONTRACT_PACKAGE_HASH_ARG => hash,
            ENTRY_POINT_ARG => entry_point,
            ARGS_ARG => Bytes::from(args_bytes),
            ATTACHED_VALUE_ARG => self.attached_value,
            AMOUNT_ARG => self.attached_value.unwrap_or_default(),
        };

        let deploy_item = DeployItemBuilder::new()
            .with_empty_payment_bytes(runtime_args! {ARG_AMOUNT => *DEFAULT_PAYMENT})
            .with_authorization_keys(&[self.active_account_hash()])
            .with_address(self.active_account_hash())
            .with_session_bytes(session_code, args)
            .with_deploy_hash(self.next_hash())
            .build();

        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy_item)
            .with_block_time(u64::from(self.block_time))
            .build();
        self.context.exec(execute_request).commit();
        self.collect_gas();
        self.gas_cost.push((
            format!("call_entrypoint {}", call_def.entry_point),
            self.last_call_contract_gas_cost()
        ));

        self.attached_value = None;
        if let Some(error) = self.context.get_error() {
            // TODO: handle error
            // let odra_error = parse_error(error);
            panic!("Error: {}", error);
            // self.error = Some(odra_error.clone());
            // self.panic_with_error(odra_error, call_def.entry_point, hash);
        } else {
            self.get_active_account_result()
        }
    }

    fn new_contract(&mut self, contract_id: &str, constructor: ModuleCaller) -> Address {
        self.error = None;
        let wasm_path = format!("resources/{}.wasm", contract_id);
        let session_code = PathBuf::from(wasm_path);
        let args = runtime_args! {
            // TODO: convert constructor to RuntimeArgs
        };

        let deploy_item = DeployItemBuilder::new()
            .with_empty_payment_bytes(runtime_args! {ARG_AMOUNT => *DEFAULT_PAYMENT})
            .with_authorization_keys(&[self.active_account_hash()])
            .with_address(self.active_account_hash())
            .with_session_code(session_code, args.clone())
            .with_deploy_hash(self.next_hash())
            .build();

        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy_item)
            .with_block_time(u64::from(self.block_time))
            .build();
        self.context.exec(execute_request).commit().expect_success();
        self.collect_gas();
        self.gas_cost.push((
            format!("deploy_contract {}", wasm_path),
            self.last_call_contract_gas_cost()
        ));
        todo!()
    }

    fn get_block_time(&self) -> BlockTime {
        self.block_time
    }

    fn callee(&self) -> Address {
        self.active_account
    }

    fn attached_value(&self) -> Option<U512> {
        self.attached_value
    }

    fn emit_event(&self, event: EventData) {
        casper_event_standard::emit(event)
    }

    fn transfer_tokens(&self, from: &Address, to: &Address, amount: U512) {

        let main_purse = match runtime::get_key(consts::CONTRACT_MAIN_PURSE).map(|key| *key.as_uref().unwrap_or_revert()) {
            Some(purse) => purse,
            None => {
                let purse = create_purse();
                runtime::put_key(consts::CONTRACT_MAIN_PURSE, purse.into());
                purse
            }
        };

    match to {
        Address::Account(account) => {
            transfer_from_purse_to_account(main_purse, *account, amount.into(), None)
                .unwrap_or_revert();
        }
        Address::Contract(_) => revert(ExecutionError::can_not_transfer_to_contract())
    };
    }

    fn balance_of(&self, address: &Address) -> U512 {
        todo!()
    }
}