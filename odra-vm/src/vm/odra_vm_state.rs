use super::balance::AccountBalance;
use super::storage::Storage;
use super::utils;
use anyhow::Result;
use odra_core::callstack::{Callstack, CallstackElement};
use odra_core::casper_types::account::AccountHash;
use odra_core::casper_types::bytesrepr::Error;
use odra_core::casper_types::crypto::gens::public_key_arb;
use odra_core::casper_types::{
    bytesrepr::{Bytes, FromBytes, ToBytes},
    PublicKey, SecretKey, U512
};
use odra_core::crypto::generate_key_pairs;
use odra_core::prelude::*;
use odra_core::EventError;
use std::collections::BTreeMap;
use std::fmt::format;

// TODO: Set it to a value corresponding to the auction delay in the Casper VM
pub const ODRA_VM_AUCTION_DELAY: u64 = 41000;

/// Struct holding the information about a transfer that is awaiting to be processed.
/// It should be executed when the block_time is greater than the block_time of the transfer.
#[derive(Clone)]
pub struct AwaitingTransfer {
    pub from: Address,
    pub to: Address,
    pub amount: U512,
    pub block_time: u64
}

/// Struct representing the state of the Odra VM.
pub struct OdraVmState {
    storage: Storage,
    callstack: Callstack,
    events: BTreeMap<Address, Vec<Bytes>>,
    native_events: BTreeMap<Address, Vec<Bytes>>,
    contract_counter: u32,
    pub error: Option<OdraError>,
    block_time: u64,
    pub accounts: Vec<Address>,
    pub validators: BTreeMap<PublicKey, U512>,
    pub validator_account: BTreeMap<PublicKey, Address>,
    pub delegations: BTreeMap<PublicKey, BTreeMap<Address, U512>>,
    pub awaiting_transfers: Vec<AwaitingTransfer>,
    key_pairs: BTreeMap<Address, (SecretKey, PublicKey)>
}

impl OdraVmState {
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
        self.push_callstack_element(CallstackElement::new_account(address));
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

    pub fn remove_dictionary(&mut self, dict: &[u8]) {
        let ctx = self.callstack.current().address();
        self.storage.remove_dict(ctx, dict);
    }

    pub fn get_dict_value(&self, dict: &[u8], key: &[u8]) -> Result<Option<Bytes>, Error> {
        let ctx = &self.callstack.current().address();
        self.storage.get_dict_value(ctx, dict, key)
    }

    pub fn emit_event(&mut self, event_data: &Bytes) {
        let contract_address = self.callstack.current().address();
        #[allow(clippy::manual_inspect)]
        let events = self.events.get_mut(contract_address).map(|events| {
            events.push(event_data.clone());
            events
        });
        if events.is_none() {
            self.events
                .insert(*contract_address, vec![event_data.clone()]);
        }
    }

    pub fn emit_native_event(&mut self, event_data: &Bytes) {
        let contract_address = self.callstack.current().address();
        #[allow(clippy::manual_inspect)]
        let events = self.native_events.get_mut(contract_address).map(|events| {
            events.push(event_data.clone());
            events
        });
        if events.is_none() {
            self.native_events
                .insert(*contract_address, vec![event_data.clone()]);
        }
    }

    pub fn get_event(&self, address: &Address, index: u32) -> Result<Bytes, EventError> {
        if !address.is_contract() {
            return Err(EventError::ContractDoesntSupportEvents);
        }
        let events = self.events.get(address);
        if events.is_none() {
            return Err(EventError::IndexOutOfBounds);
        }
        let events = events.unwrap();
        let event = events
            .get(index as usize)
            .ok_or(EventError::IndexOutOfBounds)?;
        Ok(event.clone())
    }

    pub fn get_native_event(&self, address: &Address, index: u32) -> Result<Bytes, EventError> {
        if !address.is_contract() {
            return Err(EventError::ContractDoesntSupportEvents);
        }
        let events = self.native_events.get(address);
        if events.is_none() {
            return Err(EventError::IndexOutOfBounds);
        }
        let events = events.unwrap();
        let event = events
            .get(index as usize)
            .ok_or(EventError::IndexOutOfBounds)?;
        Ok(event.clone())
    }

    // TODO: Reduce duplication
    pub fn get_events_count(&self, address: &Address) -> Result<u32, EventError> {
        if !address.is_contract() {
            return Err(EventError::ContractDoesntSupportEvents);
        }
        let events = self.events.get(address);
        if events.is_none() {
            return Err(EventError::CouldntExtractEventData);
        }
        Ok(events.unwrap().len() as u32)
    }

    pub fn get_native_events_count(&self, address: &Address) -> Result<u32, EventError> {
        if !address.is_contract() {
            return Err(EventError::ContractDoesntSupportEvents);
        }
        let events = self.native_events.get(address);
        if events.is_none() {
            return Err(EventError::CouldntExtractEventData);
        }
        Ok(events.unwrap().len() as u32)
    }

    pub fn delegated_amount(&self, validator: PublicKey, delegator: Address) -> U512 {
        let validators_delegations = self.delegations.get(&validator).unwrap();
        let delegators_amount = validators_delegations
            .get(&delegator)
            .cloned()
            .unwrap_or_default();
        delegators_amount
    }

    pub fn delegate(&mut self, validator: PublicKey, delegator: Address, amount: U512) {
        let validators_delegations = self
            .delegations
            .entry(validator.clone())
            .or_insert_with(BTreeMap::new);
        let delegation = validators_delegations
            .get(&delegator)
            .cloned()
            .unwrap_or_default();
        validators_delegations.insert(delegator, delegation + amount);

        let validators_total_amount = self.validators.get(&validator).cloned().unwrap_or_default();
        self.validators
            .insert(validator.clone(), validators_total_amount + amount);

        let validator_account = self.validator_account.get(&validator).cloned().unwrap();

        self.transfer(&delegator, &validator_account, &amount)
            .unwrap();
    }

    pub fn undelegate(&mut self, validator: PublicKey, delegator: Address, amount: U512) {
        let validators_delegations = self
            .delegations
            .entry(validator.clone())
            .or_insert_with(BTreeMap::new);
        let delegation = validators_delegations
            .get(&delegator)
            .cloned()
            .unwrap_or_default();
        validators_delegations.insert(delegator, delegation.checked_sub(amount).unwrap());

        let validators_total_amount = self.validators.get(&validator).cloned().unwrap_or_default();
        self.validators.insert(
            validator.clone(),
            validators_total_amount.checked_sub(amount).unwrap()
        );

        let transfer = AwaitingTransfer {
            from: self.validator_account[&validator],
            to: delegator,
            amount,
            block_time: self.block_time + self.unbonding_period()
        };

        self.awaiting_transfers.push(transfer);
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
        utils::contract_address_from_u32(self.contract_counter)
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
        self.callstack.size() == 1
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

    pub fn advance_with_auctions(&mut self, milliseconds: u64) {
        let time_between_auctions = self.auction_delay();

        // Calculate how many auctions we can run based on time_diff
        let num_auctions = milliseconds / time_between_auctions;

        // Run auctions and distribute rewards one at a time
        // to each validator which has a delegation
        for _ in 0..num_auctions {
            self.validators
                .iter_mut()
                .for_each(|(validator, total_amount)| {
                    if total_amount.is_zero() {
                        return;
                    }

                    let mut new_total_amount = *total_amount;

                    let delegations = self.delegations.get_mut(validator).unwrap();
                    delegations.iter_mut().for_each(|(address, amount)| {
                        let reward = *total_amount / 1000;
                        *amount += reward;
                        new_total_amount += reward;
                    });

                    *total_amount = new_total_amount;
                });
        }

        // Update the block time
        self.block_time += milliseconds;

        // Process awaiting transfers
        self.awaiting_transfers
            .clone()
            .into_iter()
            .for_each(|transfer| {
                if self.block_time >= transfer.block_time {
                    self.transfer(&transfer.from, &transfer.to, &transfer.amount)
                        .unwrap();
                }
            });

        // Remove the processed transfers from the list
        self.awaiting_transfers
            .retain(|transfer| self.block_time < transfer.block_time);
    }

    pub fn auction_delay(&self) -> u64 {
        ODRA_VM_AUCTION_DELAY
    }

    pub fn unbonding_period(&self) -> u64 {
        self.auction_delay() * 7
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

    pub fn transfer(&mut self, from: &Address, to: &Address, amount: &U512) -> Result<()> {
        self.storage.transfer(from, to, amount)
    }

    pub fn public_key(&self, address: &Address) -> PublicKey {
        let (_, public_key) = self.key_pairs.get(address).unwrap();
        public_key.clone()
    }

    pub fn secret_key(&self, address: &Address) -> &SecretKey {
        let (secret_key, _) = self.key_pairs.get(address).unwrap();
        secret_key
    }
}

impl Default for OdraVmState {
    fn default() -> Self {
        let accounts: Vec<Address> = Vec::new();
        let key_pairs = generate_key_pairs(20);
        let accounts: Vec<Address> = key_pairs.keys().copied().collect();
        let mut balances = BTreeMap::<Address, AccountBalance>::new();
        for address in accounts.clone() {
            balances.insert(address, 10_000_000_000_000_000_000u64.into());
        }

        // last 5 key pairs are validators
        let validators = key_pairs
            .iter()
            .clone()
            .rev()
            .take(5)
            .map(|(_, pk)| (pk.1.clone(), U512::zero()))
            .collect::<BTreeMap<PublicKey, U512>>();

        let validator_accounts = key_pairs
            .iter()
            .clone()
            .rev()
            .take(5)
            .map(|(address, pk)| (pk.1.clone(), *address))
            .collect::<BTreeMap<PublicKey, Address>>();

        let mut backend = OdraVmState {
            storage: Storage::new(balances),
            callstack: Default::default(),
            events: Default::default(),
            native_events: Default::default(),
            contract_counter: 0,
            error: None,
            block_time: 0,
            accounts: accounts.clone(),
            validators,
            validator_account: validator_accounts,
            delegations: Default::default(),
            awaiting_transfers: Default::default(),
            key_pairs
        };
        backend.push_callstack_element(CallstackElement::Account(*accounts.first().unwrap()));
        backend
    }
}
