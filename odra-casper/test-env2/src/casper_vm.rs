use std::env;
use std::path::PathBuf;

use casper_engine_test_support::{ARG_AMOUNT, DEFAULT_PAYMENT, DEFAULT_ACCOUNT_INITIAL_BALANCE, DEFAULT_CHAINSPEC_REGISTRY, DEFAULT_GENESIS_CONFIG, DEFAULT_GENESIS_CONFIG_HASH, DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder};
use casper_execution_engine::core::engine_state::{GenesisAccount, RunGenesisRequest};
use odra_core::{CallDef, ContractContext, HostContext};
use odra_types::{Address, CLTyped, EventData, ExecutionError, FromBytes, OdraError, PublicKey, U512};
use odra::prelude::collections::BTreeMap;
use odra::contract_env::revert;
use odra_casper_shared::consts::*;
use odra_types::casper_types::account::AccountHash;
use odra_types::casper_types::{BlockTime, Key, Motes, runtime_args, SecretKey, StoredValue};
use odra_types::casper_types::bytesrepr::{ToBytes, Bytes};
use odra_types::RuntimeArgs;

pub struct CasperVm {
    pub(crate) accounts: Vec<Address>,
    pub(crate) key_pairs: BTreeMap<Address, (SecretKey, PublicKey)>,
    pub(crate) active_account: Address,
    pub(crate) context: InMemoryWasmTestBuilder,
    pub(crate) block_time: BlockTime,
    pub(crate) calls_counter: u32,
    pub(crate) error: Option<OdraError>,
    pub(crate) attached_value: Option<U512>,
    pub(crate) gas_used: BTreeMap<AccountHash, U512>,
    pub(crate) gas_cost: Vec<(String, U512)>
}


impl CasperVm {
    /// Create a new instance with predefined accounts.
    pub fn active_account_hash(&self) -> AccountHash {
        *self.active_account.as_account_hash().unwrap()
    }

    pub fn next_hash(&mut self) -> [u8; 32] {
        let seed = self.calls_counter;
        self.calls_counter += 1;
        let mut hash = [0u8; 32];
        hash[0] = seed as u8;
        hash[1] = (seed >> 8) as u8;
        hash
    }

    /// Read a value from Account's named keys.
    pub fn get_account_value<T: CLTyped + FromBytes + ToBytes>(
        &self,
        hash: AccountHash,
        name: &str
    ) -> Result<T, String> {
        let result: Result<StoredValue, String> =
            self.context
                .query(None, Key::Account(hash), &[name.to_string()]);

        result.map(|value| value.as_cl_value().unwrap().clone().into_t().unwrap())
    }

    pub fn get_active_account_result<T: CLTyped + FromBytes>(&self) -> T {
        let active_account = self.active_account_hash();
        let bytes: Bytes = self
            .get_account_value(active_account, RESULT_KEY)
            .unwrap_or_default();
        T::from_bytes(bytes.inner_bytes()).unwrap().0
    }

    pub fn collect_gas(&mut self) {
        *self
            .gas_used
            .entry(*self.active_account.as_account_hash().unwrap())
            .or_insert_with(U512::zero) += *DEFAULT_PAYMENT;
    }

    /// Returns the cost of the last deploy.
    /// Keep in mind that this may be different from the cost of the deploy on the live network.
    /// This is NOT the amount of gas charged - see [last_call_contract_gas_used].
    pub fn last_call_contract_gas_cost(&self) -> U512 {
        self.context.last_exec_gas_cost().value()
    }

    /// Returns the amount of gas used for last call.
    pub fn last_call_contract_gas_used(&self) -> U512 {
        *DEFAULT_PAYMENT
    }

    /// Returns total gas used by the account.
    pub fn total_gas_used(&self, address: Address) -> U512 {
        match &address {
            Address::Account(address) => self.gas_used.get(address).cloned().unwrap_or_default(),
            Address::Contract(address) => panic!("Contract {} can't burn gas.", address)
        }
    }

    /// Returns the report of the gas used during the whole lifetime of the CasperVM.
    pub fn gas_report(&self) -> Vec<(String, U512)> {
        self.gas_cost.clone()
    }

    /// Returns the public key that corresponds to the given Account Address.
    pub fn public_key(&self, address: &Address) -> PublicKey {
        let (_, public_key) = self.key_pairs.get(address).unwrap();
        public_key.clone()
    }

    /// Cryptographically signs a message as a given account.
    pub fn sign_message(&self, message: &Bytes, address: &Address) -> Bytes {
        let (secret_key, public_key) = self.key_pairs.get(address).unwrap();
        let signature = odra_types::casper_types::crypto::sign(message, secret_key, public_key)
            .to_bytes()
            .unwrap();
        Bytes::from(signature)
    }
}

#[cfg(test)]
mod tests {
    use crate::casper_vm::CasperVm;
    use odra_core::HostContext;

    #[test]
    fn test_initialize() {
        let vm = CasperVm::new();
        assert_eq!(vm.accounts.len(), 20);
        assert_eq!(vm.key_pairs.len(), 20);
        assert_eq!(vm.active_account, vm.accounts[0]);
        vm.new_contract("erc20", )
    }
}