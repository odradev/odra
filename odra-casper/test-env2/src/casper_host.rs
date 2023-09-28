use crate::casper_vm::CasperVm;
use std::env;
use std::path::PathBuf;

use casper_engine_test_support::{ARG_AMOUNT, DEFAULT_PAYMENT, DEFAULT_ACCOUNT_INITIAL_BALANCE, DEFAULT_CHAINSPEC_REGISTRY, DEFAULT_GENESIS_CONFIG, DEFAULT_GENESIS_CONFIG_HASH, DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder};
use casper_execution_engine::core::engine_state::{GenesisAccount, RunGenesisRequest};
use odra_core::{CallDef, ContractContext, HostContext, ModuleCaller};
use odra_types::{Address, CLTyped, EventData, ExecutionError, FromBytes, OdraError, PublicKey, U512};
use odra::prelude::collections::BTreeMap;
use odra::contract_env::revert;
use odra_casper_shared::consts::*;
use odra_types::casper_types::account::AccountHash;
use odra_types::casper_types::{BlockTime, Key, Motes, runtime_args, SecretKey, StoredValue};
use odra_types::casper_types::bytesrepr::{ToBytes, Bytes};
use odra_types::RuntimeArgs;

impl HostContext for CasperVm {
    fn set_caller(&mut self, caller: Address) {
        todo!()
    }

    fn get_account(&self, index: usize) -> Address {
        todo!()
    }

    fn advance_block_time(&mut self, time_diff: u64) {
        todo!()
    }

    fn get_event(&self, contract_address: Address, index: i32) -> Option<odra_types::EventData> {
        todo!()
    }

    fn attach_value(&mut self, amount: U512) {
        todo!()
    }

    fn call_contract(&mut self, address: Address, call_def: CallDef) -> Vec<u8> {
        self.error = None;
        // TODO: handle unwrap
        let hash = *address.as_contract_package_hash().unwrap();

        let session_code = include_bytes!("../resources/proxy_caller_with_return.wasm").to_vec();
        let args = runtime_args! {
            // TODO: convert call_def to RuntimeArgs
        };
        let args_bytes: Vec<u8> = args.to_bytes().unwrap();
        let entry_point = call_def.entry_point.clone();
        let args = runtime_args! {
            CONTRACT_PACKAGE_HASH_ARG => hash,
            ENTRY_POINT_ARG => entry_point,
            ARGS_ARG => Bytes::from(args_bytes),
            ATTACHED_VALUE_ARG => self.attached_value,
            AMOUNT_ARG => self.attached_value.unwrap_or_default(),
        };

        let deploy_item = DeployItemBuilder::new()
            .with_empty_payment_bytes(runtime_args! { ARG_AMOUNT => *DEFAULT_PAYMENT})
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

    fn new_contract(&mut self, contract_id: &str, caller: ModuleCaller) -> Address {
        todo!()
    }
}
