use anyhow::Result;
use odra_types::{
    casper_types::{
        account::AccountHash,
        bytesrepr::{Error, FromBytes, ToBytes},
        RuntimeArgs, SecretKey, U512
    },
    Address, BlockTime, EventData, ExecutionError, PublicKey
};
use odra_types::{event::EventError, OdraError, VmError};
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use crate::balance::AccountBalance;
use crate::callstack::{Callstack, CallstackElement, Entrypoint};
use crate::contract_container::{ContractContainer, EntrypointArgs, EntrypointCall};
use crate::contract_register::ContractRegister;
use crate::storage::Storage;

#[derive(Default)]
pub struct MockVm {
    state: Arc<RwLock<MockVmState>>,
    contract_register: Arc<RwLock<ContractRegister>>
}

impl MockVm {
    pub fn register_contract(
        &self,
        constructor: Option<(String, &RuntimeArgs, EntrypointCall)>,
        constructors: BTreeMap<String, (EntrypointArgs, EntrypointCall)>,
        entrypoints: BTreeMap<String, (EntrypointArgs, EntrypointCall)>
    ) -> Address {
        // Create a new address.
        let address = { self.state.write().unwrap().next_contract_address() };
        // Register new contract under the new address.
        {
            let contract_namespace = self.state.read().unwrap().get_contract_namespace();
            let contract = ContractContainer::new(&contract_namespace, entrypoints, constructors);
            self.contract_register
                .write()
                .unwrap()
                .add(address, contract);
            self.state
                .write()
                .unwrap()
                .set_balance(address, U512::zero());
        }

        // Call constructor if needed.
        if let Some(constructor) = constructor {
            let (constructor_name, args, _) = constructor;
            self.call_constructor(address, &constructor_name, args);
        }
        address
    }

    pub fn call_contract<T: ToBytes + FromBytes>(
        &self,
        address: Address,
        entrypoint: &str,
        args: &RuntimeArgs,
        amount: Option<U512>
    ) -> T {
        self.prepare_call(address, entrypoint, amount);
        // Call contract from register.
        if let Some(amount) = amount {
            let status = self.checked_transfer_tokens(&self.caller(), &address, &amount);
            if let Err(err) = status {
                self.revert(err.clone());
                panic!("{:?}", err);
            }
        }

        let result =
            self.contract_register
                .read()
                .unwrap()
                .call(&address, String::from(entrypoint), args);

        let result = self.handle_call_result(result);
        T::from_vec(result).unwrap().0
    }

    fn call_constructor(&self, address: Address, entrypoint: &str, args: &RuntimeArgs) -> Vec<u8> {
        self.prepare_call(address, entrypoint, None);
        // Call contract from register.
        let register = self.contract_register.read().unwrap();
        let result = register.call_constructor(&address, String::from(entrypoint), args);
        self.handle_call_result(result)
    }

    fn prepare_call(&self, address: Address, entrypoint: &str, amount: Option<U512>) {
        let mut state = self.state.write().unwrap();
        // If only one address on the call_stack, record snapshot.
        if state.is_in_caller_context() {
            state.take_snapshot();
            state.clear_error();
        }
        // Put the address on stack.

        let element = CallstackElement::Entrypoint(Entrypoint::new(address, entrypoint, amount));
        state.push_callstack_element(element);
    }

    fn handle_call_result(&self, result: Result<Vec<u8>, OdraError>) -> Vec<u8> {
        let mut state = self.state.write().unwrap();
        let result = match result {
            Ok(data) => data,
            Err(err) => {
                state.set_error(err);
                vec![]
            }
        };

        // Drop the address from stack.
        state.pop_callstack_element();

        if state.error.is_none() {
            // If only one address on the call_stack, drop the snapshot
            if state.is_in_caller_context() {
                state.drop_snapshot();
            }
            result
        } else {
            // If only one address on the call_stack an an error occurred, restore the snapshot
            if state.is_in_caller_context() {
                state.restore_snapshot();
            };
            vec![]
        }
    }

    pub fn revert(&self, error: OdraError) {
        let mut state = self.state.write().unwrap();
        state.set_error(error);
        state.clear_callstack();
        if state.is_in_caller_context() {
            state.restore_snapshot();
        }
    }

    pub fn error(&self) -> Option<OdraError> {
        self.state.read().unwrap().error()
    }

    pub fn get_backend_name(&self) -> String {
        self.state.read().unwrap().get_backend_name()
    }

    /// Returns the callee, i.e. the currently executing contract.
    pub fn callee(&self) -> Address {
        self.state.read().unwrap().callee()
    }

    pub fn caller(&self) -> Address {
        self.state.read().unwrap().caller()
    }

    pub fn callstack_tip(&self) -> CallstackElement {
        self.state.read().unwrap().callstack_tip().clone()
    }

    pub fn set_caller(&self, caller: Address) {
        self.state.write().unwrap().set_caller(caller);
    }

    pub fn set_var<T: ToBytes>(&self, key: &[u8], value: T) {
        self.state.write().unwrap().set_var(key, value);
    }

    pub fn get_var<T: FromBytes>(&self, key: &[u8]) -> Option<T> {
        let result = { self.state.read().unwrap().get_var(key) };
        match result {
            Ok(result) => result,
            Err(error) => {
                self.state
                    .write()
                    .unwrap()
                    .set_error(Into::<ExecutionError>::into(error));
                None
            }
        }
    }

    pub fn set_dict_value<T: ToBytes>(&self, dict: &[u8], key: &[u8], value: T) {
        self.state.write().unwrap().set_dict_value(dict, key, value);
    }

    pub fn get_dict_value<T: FromBytes>(&self, dict: &[u8], key: &[u8]) -> Option<T> {
        let result = { self.state.read().unwrap().get_dict_value(dict, key) };
        match result {
            Ok(result) => result,
            Err(error) => {
                self.state
                    .write()
                    .unwrap()
                    .set_error(Into::<ExecutionError>::into(error));
                None
            }
        }
    }

    pub fn emit_event(&self, event_data: &EventData) {
        self.state.write().unwrap().emit_event(event_data);
    }

    pub fn get_event(&self, address: Address, index: i32) -> Result<EventData, EventError> {
        self.state.read().unwrap().get_event(address, index)
    }

    pub fn get_block_time(&self) -> BlockTime {
        self.state.read().unwrap().block_time()
    }

    pub fn advance_block_time_by(&self, milliseconds: BlockTime) {
        self.state
            .write()
            .unwrap()
            .advance_block_time_by(milliseconds)
    }

    pub fn attached_value(&self) -> U512 {
        self.state.read().unwrap().attached_value()
    }

    pub fn get_account(&self, n: usize) -> Address {
        self.state.read().unwrap().accounts.get(n).cloned().unwrap()
    }

    pub fn token_balance(&self, address: Address) -> U512 {
        self.state.read().unwrap().get_balance(address)
    }

    pub fn transfer_tokens(&self, from: &Address, to: &Address, amount: &U512) {
        if amount.is_zero() {
            return;
        }

        let mut state = self.state.write().unwrap();
        if state.reduce_balance(from, amount).is_err() {
            self.revert(OdraError::VmError(VmError::BalanceExceeded))
        }
        if state.increase_balance(to, amount).is_err() {
            self.revert(OdraError::VmError(VmError::BalanceExceeded))
        }
    }

    pub fn checked_transfer_tokens(
        &self,
        from: &Address,
        to: &Address,
        amount: &U512
    ) -> Result<(), OdraError> {
        if amount.is_zero() {
            return Ok(());
        }

        let mut state = self.state.write().unwrap();
        if state.reduce_balance(from, amount).is_err() {
            return Err(OdraError::VmError(VmError::BalanceExceeded));
        }
        if state.increase_balance(to, amount).is_err() {
            return Err(OdraError::VmError(VmError::BalanceExceeded));
        }
        Ok(())
    }

    pub fn self_balance(&self) -> U512 {
        let address = self.callee();
        self.state.read().unwrap().get_balance(address)
    }

    pub fn public_key(&self, address: &Address) -> PublicKey {
        self.state.read().unwrap().public_key(address)
    }
}

pub struct MockVmState {
    storage: Storage,
    callstack: Callstack,
    events: BTreeMap<Address, Vec<EventData>>,
    contract_counter: u32,
    error: Option<OdraError>,
    block_time: u64,
    accounts: Vec<Address>,
    key_pairs: BTreeMap<Address, (SecretKey, PublicKey)>
}

impl MockVmState {
    fn get_backend_name(&self) -> String {
        "MockVM".to_string()
    }

    fn callee(&self) -> Address {
        *self.callstack.current().address()
    }

    fn caller(&self) -> Address {
        *self.callstack.previous().address()
    }

    fn callstack_tip(&self) -> &CallstackElement {
        self.callstack.current()
    }

    fn set_caller(&mut self, address: Address) {
        self.pop_callstack_element();
        self.push_callstack_element(CallstackElement::Account(address));
    }

    fn set_var<T: ToBytes>(&mut self, key: &[u8], value: T) {
        let ctx = self.callstack.current().address();
        if let Err(error) = self.storage.set_value(ctx, key, value) {
            self.set_error(Into::<ExecutionError>::into(error));
        }
    }

    fn get_var<T: FromBytes>(&self, key: &[u8]) -> Result<Option<T>, Error> {
        let ctx = self.callstack.current().address();
        self.storage.get_value(ctx, key)
    }

    fn set_dict_value<T: ToBytes>(&mut self, dict: &[u8], key: &[u8], value: T) {
        let ctx = self.callstack.current().address();
        if let Err(error) = self.storage.insert_dict_value(ctx, dict, key, value) {
            self.set_error(Into::<ExecutionError>::into(error));
        }
    }

    fn get_dict_value<T: FromBytes>(&self, dict: &[u8], key: &[u8]) -> Result<Option<T>, Error> {
        let ctx = &self.callstack.current().address();
        self.storage.get_dict_value(ctx, dict, key)
    }

    fn emit_event(&mut self, event_data: &EventData) {
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

    fn get_event(&self, address: Address, index: i32) -> Result<EventData, EventError> {
        let events = self.events.get(&address);
        if events.is_none() {
            return Err(EventError::IndexOutOfBounds);
        }
        let events: &Vec<EventData> = events.unwrap();
        let event_position = odra_utils::event_absolute_position(events.len(), index)
            .ok_or(EventError::IndexOutOfBounds)?;
        Ok(events.get(event_position).unwrap().clone())
    }

    fn push_callstack_element(&mut self, element: CallstackElement) {
        self.callstack.push(element);
    }

    fn pop_callstack_element(&mut self) {
        self.callstack.pop();
    }

    fn clear_callstack(&mut self) {
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

    fn next_contract_address(&mut self) -> Address {
        self.contract_counter += 1;
        Address::contract_from_u32(self.contract_counter)
    }

    fn get_contract_namespace(&self) -> String {
        self.contract_counter.to_string()
    }

    fn set_error<E>(&mut self, error: E)
    where
        E: Into<OdraError>
    {
        if self.error.is_none() {
            self.error = Some(error.into());
        }
    }

    fn attached_value(&self) -> U512 {
        self.callstack.current_amount()
    }

    fn clear_error(&mut self) {
        self.error = None;
    }

    fn error(&self) -> Option<OdraError> {
        self.error.clone()
    }

    fn is_in_caller_context(&self) -> bool {
        self.callstack.len() == 1
    }

    fn take_snapshot(&mut self) {
        self.storage.take_snapshot();
    }

    fn drop_snapshot(&mut self) {
        self.storage.drop_snapshot();
    }

    fn restore_snapshot(&mut self) {
        self.storage.restore_snapshot();
    }

    fn block_time(&self) -> u64 {
        self.block_time
    }

    fn advance_block_time_by(&mut self, milliseconds: u64) {
        self.block_time += milliseconds;
    }

    fn get_balance(&self, address: Address) -> U512 {
        self.storage
            .balance_of(&address)
            .map(|b| b.value())
            .unwrap_or_default()
    }

    fn set_balance(&mut self, address: Address, amount: U512) {
        self.storage
            .set_balance(address, AccountBalance::new(amount));
    }

    fn increase_balance(&mut self, address: &Address, amount: &U512) -> Result<()> {
        self.storage.increase_balance(address, amount)
    }

    fn reduce_balance(&mut self, address: &Address, amount: &U512) -> Result<()> {
        self.storage.reduce_balance(address, amount)
    }

    fn public_key(&self, address: &Address) -> PublicKey {
        let (_, public_key) = self.key_pairs.get(address).unwrap();
        public_key.clone()
    }
}

impl Default for MockVmState {
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
            balances.insert(address, 100_000_000_000_000u64.into());
        }

        let mut backend = MockVmState {
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use odra_types::casper_types::bytesrepr::FromBytes;
    use odra_types::casper_types::{RuntimeArgs, U512};
    use odra_types::OdraAddress;
    use odra_types::{Address, EventData};
    use odra_types::{ExecutionError, OdraError, VmError};

    use crate::{callstack::CallstackElement, EntrypointArgs, EntrypointCall};

    use super::MockVm;

    #[test]
    fn contracts_have_different_addresses() {
        // given a new instance
        let instance = MockVm::default();
        // when register two contracts with the same entrypoints
        let entrypoint: Vec<(String, (EntrypointArgs, EntrypointCall))> =
            vec![(String::from("abc"), (vec![], |_, _| vec![]))];
        let entrypoints = entrypoint.into_iter().collect::<BTreeMap<_, _>>();
        let constructors = BTreeMap::new();

        let address1 = instance.register_contract(None, constructors.clone(), entrypoints.clone());
        let address2 = instance.register_contract(None, constructors, entrypoints);

        // then addresses are different
        assert_ne!(address1, address2);
    }

    #[test]
    fn addresses_have_different_type() {
        // given an address of a contract and an address of an account
        let instance = MockVm::default();
        let (contract_address, _, _) = setup_contract(&instance);
        let account_address = instance.get_account(0);

        // Then the contract address is a contract
        assert!(contract_address.is_contract());
        // And the account address is not a contract
        assert!(!account_address.is_contract());
    }

    #[test]
    fn test_contract_call() {
        // given an instance with a registered contract having one entrypoint
        let instance = MockVm::default();

        let (contract_address, entrypoint, call_result) = setup_contract(&instance);

        // when call an existing entrypoint
        let result =
            instance.call_contract::<u32>(contract_address, &entrypoint, &RuntimeArgs::new(), None);

        // then returns the expected value
        assert_eq!(result, call_result);
    }

    #[test]
    fn test_call_non_existing_contract() {
        // given an empty vm
        let instance = MockVm::default();

        let address = Address::contract_from_u32(42);

        // when call a contract
        instance.call_contract::<()>(address, "abc", &RuntimeArgs::new(), None);

        // then the vm is in error state
        assert_eq!(
            instance.error(),
            Some(OdraError::VmError(VmError::InvalidContractAddress))
        );
    }

    #[test]
    fn test_call_non_existing_entrypoint() {
        // given an instance with a registered contract having one entrypoint
        let instance = MockVm::default();
        let (contract_address, entrypoint, _) = setup_contract(&instance);

        // when call non-existing entrypoint
        let invalid_entrypoint = entrypoint.chars().take(1).collect::<String>();
        instance.call_contract::<()>(
            contract_address,
            &invalid_entrypoint,
            &RuntimeArgs::new(),
            None
        );

        // then the vm is in error state
        assert_eq!(
            instance.error(),
            Some(OdraError::VmError(VmError::NoSuchMethod(
                invalid_entrypoint
            )))
        );
    }

    #[test]
    fn test_caller_switching() {
        // given an empty instance
        let instance = MockVm::default();

        // when set a new caller
        let new_caller = Address::account_from_str("ff");
        instance.set_caller(new_caller);
        // put a fake contract on stack
        push_address(&instance, &new_caller);

        // then the caller is set
        assert_eq!(instance.caller(), new_caller);
    }

    #[test]
    fn test_revert() {
        // given an empty instance
        let instance = MockVm::default();

        // when revert
        instance.revert(ExecutionError::new(1, "err").into());

        // then an error is set
        assert_eq!(instance.error(), Some(ExecutionError::new(1, "err").into()));
    }

    #[test]
    fn test_read_write_value() {
        // given an empty instance
        let instance = MockVm::default();

        // when set a value
        let key = b"key";
        let value = 32u8;
        instance.set_var(key, value);

        // then the value can be read
        assert_eq!(instance.get_var(key), Some(value));
        // then the value under unknown key does not exist
        assert_eq!(instance.get_var::<()>(b"other_key"), None);
    }

    #[test]
    fn test_read_write_dict() {
        // given an empty instance
        let instance = MockVm::default();

        // when set a value
        let dict = b"dict";
        let key: [u8; 2] = [1, 2];
        let value = 32u8;
        instance.set_dict_value(dict, &key, value);

        // then the value can be read
        assert_eq!(instance.get_dict_value(dict, &key), Some(value));
        // then the value under the key in unknown dict does not exist
        assert_eq!(instance.get_dict_value::<()>(b"other_dict", &key), None);
        // then the value under unknown key does not exist
        assert_eq!(instance.get_dict_value::<()>(dict, &[]), None);
    }

    #[test]
    fn events() {
        // given an empty instance
        let instance = MockVm::default();

        let first_contract_address = Address::account_from_str("abc");
        // put a contract on stack
        push_address(&instance, &first_contract_address);

        let first_event: EventData = vec![1, 2, 3];
        let second_event: EventData = vec![4, 5, 6];
        instance.emit_event(&first_event);
        instance.emit_event(&second_event);

        let second_contract_address = Address::account_from_str("bca");
        // put a next contract on stack
        push_address(&instance, &second_contract_address);

        let third_event: EventData = vec![7, 8, 9];
        let fourth_event: EventData = vec![11, 22, 33];
        instance.emit_event(&third_event);
        instance.emit_event(&fourth_event);

        assert_eq!(
            instance.get_event(first_contract_address, 0),
            Ok(first_event)
        );
        assert_eq!(
            instance.get_event(first_contract_address, 1),
            Ok(second_event)
        );

        assert_eq!(
            instance.get_event(second_contract_address, 0),
            Ok(third_event)
        );
        assert_eq!(
            instance.get_event(second_contract_address, 1),
            Ok(fourth_event)
        );
    }

    #[test]
    fn test_current_contract_address() {
        // given an empty instance
        let instance = MockVm::default();

        // when push a contract into the stack
        let contract_address = Address::contract_from_u32(100);
        push_address(&instance, &contract_address);

        // then the contract address in the callee
        assert_eq!(instance.callee(), contract_address);
    }

    #[test]
    fn test_call_contract_with_amount() {
        // given an instance with a registered contract having one entrypoint
        let instance = MockVm::default();
        let (contract_address, entrypoint_name, _) = setup_contract(&instance);

        // when call a contract with the whole balance of the caller
        let caller = instance.get_account(0);
        let caller_balance = instance.token_balance(caller);

        instance.call_contract::<u32>(
            contract_address,
            &entrypoint_name,
            &RuntimeArgs::new(),
            Some(caller_balance)
        );

        // then the contract has the caller tokens and the caller balance is zero
        assert_eq!(instance.token_balance(contract_address), caller_balance);
        assert_eq!(instance.token_balance(caller), U512::zero());
    }

    #[test]
    #[should_panic(expected = "VmError(BalanceExceeded)")]
    fn test_call_contract_with_amount_exceeding_balance() {
        // given an instance with a registered contract having one entrypoint
        let instance = MockVm::default();
        let (contract_address, entrypoint_name, _) = setup_contract(&instance);

        let caller = instance.get_account(0);
        let caller_balance = instance.token_balance(caller);

        // when call a contract with the amount exceeding caller's balance
        instance.call_contract::<()>(
            contract_address,
            &entrypoint_name,
            &RuntimeArgs::new(),
            Some(caller_balance + 1)
        );
    }

    fn push_address(vm: &MockVm, address: &Address) {
        let element = CallstackElement::Account(*address);
        vm.state.write().unwrap().push_callstack_element(element);
    }

    fn setup_contract(instance: &MockVm) -> (Address, String, u32) {
        let entrypoint_name = "abc";
        let result = vec![1, 1, 0, 0];

        let entrypoint: Vec<(String, (EntrypointArgs, EntrypointCall))> = vec![(
            String::from(entrypoint_name),
            (vec![], |_, _| vec![1, 1, 0, 0])
        )];
        let constructors = BTreeMap::new();
        let contract_address = instance.register_contract(
            None,
            constructors,
            entrypoint.into_iter().collect::<BTreeMap<_, _>>()
        );

        (
            contract_address,
            String::from(entrypoint_name),
            <u32 as FromBytes>::from_vec(result).unwrap().0
        )
    }
}
