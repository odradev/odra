use super::balance::AccountBalance;
use super::callstack::{Callstack, CallstackElement};
use super::storage::Storage;
use anyhow::Result;
use odra_core::event::EventError;
use odra_types::casper_types::account::AccountHash;
use odra_types::casper_types::bytesrepr::Error;
use odra_types::{
    Address, Bytes, EventData, ExecutionError, FromBytes, OdraError, PublicKey, SecretKey, ToBytes,
    U512
};
use std::collections::BTreeMap;

pub struct OdraVmState {
    storage: Storage,
    callstack: Callstack,
    events: BTreeMap<Address, Vec<Bytes>>,
    contract_counter: u32,
    pub error: Option<OdraError>,
    block_time: u64,
    pub accounts: Vec<Address>,
    key_pairs: BTreeMap<Address, (SecretKey, PublicKey)>
}

impl OdraVmState {
    pub fn get_backend_name(&self) -> String {
        "MockVM".to_string()
    }

    pub fn callee(&self) -> Address {
        *self.callstack.current().address()
    }

    pub fn caller(&self) -> Address {
        *self.callstack.previous().address()
    }

    pub fn callstack_tip(&self) -> &CallstackElement {
        self.callstack.current()
    }

    pub fn set_caller(&mut self, address: Address) {
        self.pop_callstack_element();
        self.push_callstack_element(CallstackElement::Account(address));
    }

    pub fn set_var(&mut self, key: &[u8], value: Bytes) {
        let ctx = self.callstack.current().address();
        if let Err(error) = self.storage.set_value(ctx, key, value) {
            self.set_error(Into::<ExecutionError>::into(error));
        }
    }

    pub fn get_var(&self, key: &[u8]) -> Result<Option<Bytes>, Error> {
        let ctx = self.callstack.current().address();
        self.storage.get_value(ctx, key)
    }

    pub fn set_dict_value(&mut self, dict: &[u8], key: &[u8], value: Bytes) {
        let ctx = self.callstack.current().address();
        if let Err(error) = self.storage.insert_dict_value(ctx, dict, key, value) {
            self.set_error(Into::<ExecutionError>::into(error));
        }
    }

    pub fn get_dict_value(&self, dict: &[u8], key: &[u8]) -> Result<Option<Bytes>, Error> {
        let ctx = &self.callstack.current().address();
        self.storage.get_dict_value(ctx, dict, key)
    }

    pub fn emit_event(&mut self, event_data: &Bytes) {
        let contract_address = self.callstack.current().address();
        let events = self.events.get_mut(contract_address).map(|events| {
            events.push(event_data.clone());
            events
        });
        if events.is_none() {
            self.events
                .insert(*contract_address, vec![event_data.clone()]);
        }
    }

    pub fn get_event(&self, address: &Address, index: i32) -> Result<Bytes, EventError> {
        let events = self.events.get(address);
        if events.is_none() {
            return Err(EventError::IndexOutOfBounds);
        }
        let events: &Vec<Bytes> = events.unwrap();
        let event_position = odra_utils::event_absolute_position(events.len(), index)
            .ok_or(EventError::IndexOutOfBounds)?;
        let event = events
            .get(event_position)
            .ok_or(EventError::IndexOutOfBounds)?;
        Ok(event.clone())
    }

    pub fn get_events_count(&self, address: &Address) -> u32 {
        self.events
            .get(address)
            .map_or(0, |events| events.len() as u32)
    }

    pub fn attach_value(&mut self, amount: U512) {
        self.callstack.attach_value(amount);
    }

    pub fn push_callstack_element(&mut self, element: CallstackElement) {
        self.callstack.push(element);
    }

    pub fn pop_callstack_element(&mut self) {
        self.callstack.pop();
    }

    pub fn clear_callstack(&mut self) {
        let mut element = self.callstack.pop();
        while element.is_some() {
            let new_element = self.callstack.pop();
            if new_element.is_none() {
                self.callstack.push(element.unwrap());
                return;
            }
            element = new_element;
        }
    }

    pub fn next_contract_address(&mut self) -> Address {
        self.contract_counter += 1;
        Address::contract_from_u32(self.contract_counter)
    }

    pub fn get_contract_namespace(&self) -> String {
        self.contract_counter.to_string()
    }

    pub fn set_error<E>(&mut self, error: E)
    where
        E: Into<OdraError>
    {
        if self.error.is_none() {
            self.error = Some(error.into());
        }
    }

    pub fn attached_value(&self) -> U512 {
        self.callstack.attached_value()
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }

    pub fn error(&self) -> Option<OdraError> {
        self.error.clone()
    }

    pub fn is_in_caller_context(&self) -> bool {
        self.callstack.len() == 1
    }

    pub fn take_snapshot(&mut self) {
        self.storage.take_snapshot();
    }

    pub fn drop_snapshot(&mut self) {
        self.storage.drop_snapshot();
    }

    pub fn restore_snapshot(&mut self) {
        self.storage.restore_snapshot();
    }

    pub fn block_time(&self) -> u64 {
        self.block_time
    }

    pub fn advance_block_time_by(&mut self, milliseconds: u64) {
        self.block_time += milliseconds;
    }

    pub fn balance_of(&self, address: &Address) -> U512 {
        self.storage
            .balance_of(address)
            .map(|b| b.value())
            .unwrap_or_default()
    }

    pub fn all_balances(&self) -> Vec<AccountBalance> {
        self.storage
            .balances
            .iter()
            .fold(Vec::new(), |mut acc, (_, balance)| {
                acc.push(balance.clone());
                acc
            })
    }

    pub fn set_balance(&mut self, address: Address, amount: U512) {
        self.storage
            .set_balance(address, AccountBalance::new(amount));
    }

    pub fn increase_balance(&mut self, address: &Address, amount: &U512) -> Result<()> {
        self.storage.increase_balance(address, amount)
    }

    pub fn reduce_balance(&mut self, address: &Address, amount: &U512) -> Result<()> {
        self.storage.reduce_balance(address, amount)
    }

    pub fn public_key(&self, address: &Address) -> PublicKey {
        let (_, public_key) = self.key_pairs.get(address).unwrap();
        public_key.clone()
    }
}

impl Default for OdraVmState {
    fn default() -> Self {
        let mut addresses: Vec<Address> = Vec::new();
        let mut key_pairs = BTreeMap::<Address, (SecretKey, PublicKey)>::new();
        for i in 0..20 {
            // Create keypair.
            let secret_key = SecretKey::ed25519_from_bytes([i; 32]).unwrap();
            let public_key = PublicKey::from(&secret_key);

            // Create an AccountHash from a public key.
            let account_addr = AccountHash::from(&public_key);

            addresses.push(account_addr.try_into().unwrap());
            key_pairs.insert(account_addr.try_into().unwrap(), (secret_key, public_key));
        }

        let mut balances = BTreeMap::<Address, AccountBalance>::new();
        for address in addresses.clone() {
            balances.insert(address, 100_000_000_000_000_000u64.into());
        }

        let mut backend = OdraVmState {
            storage: Storage::new(balances),
            callstack: Default::default(),
            events: Default::default(),
            contract_counter: 0,
            error: None,
            block_time: 0,
            accounts: addresses.clone(),
            key_pairs
        };
        backend.push_callstack_element(CallstackElement::Account(*addresses.first().unwrap()));
        backend
    }
}