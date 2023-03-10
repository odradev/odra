use anyhow::Result;
use odra_mock_vm_types::{
    Address, Balance, BlockTime, CallArgs, EventData, MockVMSerializationError, MockVMType,
    CONTRACT_ADDRESS_PREFIX
};
use odra_types::{event::EventError, OdraError, VmError};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::balance::AccountBalance;
use crate::callstack::{Callstack, CallstackElement};
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
        constructor: Option<(String, CallArgs, EntrypointCall)>,
        constructors: HashMap<String, (EntrypointArgs, EntrypointCall)>,
        entrypoints: HashMap<String, (EntrypointArgs, EntrypointCall)>
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
                .set_balance(address, Balance::zero());
        }

        // Call constructor if needed.
        if let Some(constructor) = constructor {
            let (constructor_name, args, _) = constructor;
            self.call_constructor(address, &constructor_name, args);
        }
        address
    }

    pub fn call_contract<T: MockVMType>(
        &self,
        address: Address,
        entrypoint: &str,
        args: CallArgs,
        amount: Option<Balance>
    ) -> T {
        self.prepare_call(address, amount);
        // Call contract from register.
        if let Some(amount) = amount {
            let success = self.transfer_tokens(self.caller(), address, amount);
            if !success {
                let bytes =
                    self.handle_call_result(Err(OdraError::VmError(VmError::BalanceExceeded)));
                return T::deser(bytes).unwrap();
            }
        }

        let result =
            self.contract_register
                .read()
                .unwrap()
                .call(&address, String::from(entrypoint), args);

        let result = self.handle_call_result(result);
        dbg!(entrypoint);
        dbg!(result.clone());
        T::deser(result).unwrap()
    }

    fn call_constructor(&self, address: Address, entrypoint: &str, args: CallArgs) -> Vec<u8> {
        self.prepare_call(address, None);
        // Call contract from register.
        let register = self.contract_register.read().unwrap();
        let result = register.call_constructor(&address, String::from(entrypoint), args);
        self.handle_call_result(result)
    }

    fn prepare_call(&self, address: Address, amount: Option<Balance>) {
        let mut state = self.state.write().unwrap();
        // If only one address on the call_stack, record snapshot.
        if state.is_in_caller_context() {
            state.take_snapshot();
            state.clear_error();
        }
        // Put the address on stack.
        let element = CallstackElement::new(address, amount);
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
        self.state.write().unwrap().set_error(error);
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

    pub fn set_caller(&self, caller: Address) {
        self.state.write().unwrap().set_caller(caller);
    }

    pub fn set_var<T: MockVMType>(&self, key: &str, value: T) {
        self.state.write().unwrap().set_var(key, value);
    }

    pub fn get_var<T: MockVMType>(&self, key: &str) -> Option<T> {
        let result = { self.state.read().unwrap().get_var(key) };
        match result {
            Ok(result) => result,
            Err(error) => {
                self.state.write().unwrap().set_error(error);
                None
            }
        }
    }

    pub fn set_dict_value<T: MockVMType>(&self, dict: &str, key: &[u8], value: T) {
        self.state.write().unwrap().set_dict_value(dict, key, value);
    }

    pub fn get_dict_value<T: MockVMType>(&self, dict: &str, key: &[u8]) -> Option<T> {
        let result = { self.state.read().unwrap().get_dict_value(dict, key) };
        match result {
            Ok(result) => result,
            Err(error) => {
                self.state.write().unwrap().set_error(error);
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

    pub fn advance_block_time_by(&self, seconds: BlockTime) {
        self.state.write().unwrap().advance_block_time_by(seconds)
    }

    pub fn attached_value(&self) -> Balance {
        self.state.read().unwrap().attached_value()
    }

    pub fn get_address(&self, n: usize) -> Address {
        self.state.read().unwrap().accounts.get(n).cloned().unwrap()
    }

    pub fn token_balance(&self, address: Address) -> Balance {
        self.state.read().unwrap().get_balance(address)
    }

    pub fn transfer_tokens(&self, from: Address, to: Address, amount: Balance) -> bool {
        if amount == Balance::zero() {
            return true;
        }

        let mut state = self.state.write().unwrap();
        if state.reduce_balance(from, amount).is_err() {
            return false;
        }
        if state.increase_balance(to, amount).is_err() {
            return false;
        }
        true
    }

    pub fn self_balance(&self) -> Balance {
        let address = self.callee();
        self.state.read().unwrap().get_balance(address)
    }
}

#[derive(Clone)]
pub struct MockVmState {
    storage: Storage,
    callstack: Callstack,
    events: HashMap<Address, Vec<EventData>>,
    contract_counter: u32,
    error: Option<OdraError>,
    block_time: u64,
    accounts: Vec<Address>
}

impl MockVmState {
    fn get_backend_name(&self) -> String {
        "MockVM".to_string()
    }

    fn callee(&self) -> Address {
        self.callstack.current().address
    }

    fn caller(&self) -> Address {
        self.callstack.previous().address
    }

    fn set_caller(&mut self, address: Address) {
        self.pop_callstack_element();
        self.push_callstack_element(CallstackElement::new(address, None));
    }

    fn set_var<T: MockVMType>(&mut self, key: &str, value: T) {
        let ctx = &self.callstack.current().address;
        if let Err(err) = self.storage.set_value(ctx, key, value) {
            self.set_error(err);
        }
    }

    fn get_var<T: MockVMType>(&self, key: &str) -> Result<Option<T>, MockVMSerializationError> {
        let ctx = &self.callstack.current().address;
        self.storage.get_value(ctx, key)
    }

    fn set_dict_value<T: MockVMType>(&mut self, dict: &str, key: &[u8], value: T) {
        let ctx = &self.callstack.current().address;
        if let Err(err) = self.storage.insert_dict_value(ctx, dict, key, value) {
            self.set_error(err);
        }
    }

    fn get_dict_value<T: MockVMType>(
        &self,
        dict: &str,
        key: &[u8]
    ) -> Result<Option<T>, MockVMSerializationError> {
        let ctx = &self.callstack.current().address;
        self.storage.get_dict_value(ctx, dict, key)
    }

    fn emit_event(&mut self, event_data: &EventData) {
        let contract_address = &self.callstack.current().address;
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

    fn next_contract_address(&mut self) -> Address {
        self.contract_counter += 1;
        let contract_address_bytes = CONTRACT_ADDRESS_PREFIX.to_be_bytes();
        let contract_counter_bytes = self.contract_counter.to_be_bytes();
        let mut merged: [u8; 8] = [0; 8];

        merged.clone_from_slice(
            [contract_address_bytes, contract_counter_bytes]
                .concat()
                .as_slice()
        );

        Address::try_from(&merged).unwrap()
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

    fn attached_value(&self) -> Balance {
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

    fn advance_block_time_by(&mut self, seconds: u64) {
        self.block_time += seconds;
    }

    fn get_balance(&self, address: Address) -> Balance {
        self.storage
            .balance_of(&address)
            .map(|b| b.value())
            .unwrap_or_default()
    }

    fn set_balance(&mut self, address: Address, amount: Balance) {
        self.storage
            .set_balance(address, AccountBalance::new(amount));
    }

    fn increase_balance(&mut self, address: Address, amount: Balance) -> Result<()> {
        self.storage.increase_balance(&address, amount)
    }

    fn reduce_balance(&mut self, address: Address, amount: Balance) -> Result<()> {
        self.storage.reduce_balance(&address, amount)
    }
}

impl Default for MockVmState {
    fn default() -> Self {
        let addresses = vec![
            Address::try_from(b"alice").unwrap(),
            Address::try_from(b"bob").unwrap(),
            Address::try_from(b"cab").unwrap(),
            Address::try_from(b"dan").unwrap(),
            Address::try_from(b"ed").unwrap(),
            Address::try_from(b"frank").unwrap(),
            Address::try_from(b"garry").unwrap(),
            Address::try_from(b"harry").unwrap(),
            Address::try_from(b"ivan").unwrap(),
        ];

        let mut balances = HashMap::<Address, AccountBalance>::new();
        for address in addresses.clone() {
            balances.insert(address, 100_000.into());
        }

        let mut backend = MockVmState {
            storage: Storage::new(balances),
            callstack: Default::default(),
            events: Default::default(),
            contract_counter: 0,
            error: None,
            block_time: 0,
            accounts: addresses.clone()
        };
        backend.push_callstack_element(CallstackElement::new(*addresses.first().unwrap(), None));
        backend
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use odra_mock_vm_types::{Address, Balance, CallArgs, EventData, MockVMType};
    use odra_types::address::OdraAddress;
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
        let entrypoints = entrypoint.into_iter().collect::<HashMap<_, _>>();
        let constructors = HashMap::new();

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
        let account_address = instance.get_address(0);

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
            instance.call_contract::<u32>(contract_address, &entrypoint, CallArgs::new(), None);

        // then returns the expected value
        assert_eq!(result, call_result);
    }

    #[test]
    fn test_call_non_existing_contract() {
        // given an empty vm
        let instance = MockVm::default();

        let address = Address::try_from(b"random").unwrap();

        // when call a contract
        instance.call_contract::<()>(address, "abc", CallArgs::new(), None);

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
        instance.call_contract::<()>(contract_address, &invalid_entrypoint, CallArgs::new(), None);

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
        let new_caller = Address::try_from(b"new caller").unwrap();
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
        let key = "key";
        let value = 32u8;
        instance.set_var(key, value);

        // then the value can be read
        assert_eq!(instance.get_var(key), Some(value));
        // then the value under unknown key does not exist
        assert_eq!(instance.get_var::<()>("other_key"), None);
    }

    #[test]
    fn test_read_write_dict() {
        // given an empty instance
        let instance = MockVm::default();

        // when set a value
        let dict = "dict";
        let key: [u8; 2] = [1, 2];
        let value = 32u8;
        instance.set_dict_value(dict, &key, value);

        // then the value can be read
        assert_eq!(instance.get_dict_value(dict, &key), Some(value));
        // then the value under the key in unknown dict does not exist
        assert_eq!(instance.get_dict_value::<()>("other_dict", &key), None);
        // then the value under unknown key does not exist
        assert_eq!(instance.get_dict_value::<()>(dict, &[]), None);
    }

    #[test]
    fn events() {
        // given an empty instance
        let instance = MockVm::default();

        let first_contract_address = Address::try_from(b"abc").unwrap();
        // put a contract on stack
        push_address(&instance, &first_contract_address);

        let first_event: EventData = vec![1, 2, 3];
        let second_event: EventData = vec![4, 5, 6];
        instance.emit_event(&first_event);
        instance.emit_event(&second_event);

        let second_contract_address = Address::try_from(b"bca").unwrap();
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
        let contract_address = Address::try_from(b"contract").unwrap();
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
        let caller = instance.get_address(0);
        let caller_balance = instance.token_balance(caller);

        instance.call_contract::<u32>(
            contract_address,
            &entrypoint_name,
            CallArgs::new(),
            Some(caller_balance)
        );

        // then the contract has the caller tokens and the caller balance is zero
        assert_eq!(instance.token_balance(contract_address), caller_balance);
        assert_eq!(instance.token_balance(caller), Balance::zero());
    }

    #[test]
    fn test_call_contract_with_amount_exceeding_balance() {
        // given an instance with a registered contract having one entrypoint
        let instance = MockVm::default();
        let (contract_address, entrypoint_name, _) = setup_contract(&instance);

        let caller = instance.get_address(0);
        let caller_balance = instance.token_balance(caller);

        // when call a contract with the amount exceeding caller's balance
        instance.call_contract::<()>(
            contract_address,
            &entrypoint_name,
            CallArgs::new(),
            Some(caller_balance + 1)
        );

        // then the vm raises an error
        assert_eq!(
            instance.error(),
            Some(OdraError::VmError(VmError::BalanceExceeded))
        );
    }

    fn push_address(vm: &MockVm, address: &Address) {
        let element = CallstackElement::new(*address, None);
        vm.state.write().unwrap().push_callstack_element(element);
    }

    fn setup_contract(instance: &MockVm) -> (Address, String, u32) {
        let entrypoint_name = "abc";
        let result = vec![1, 1, 0, 0];

        let entrypoint: Vec<(String, (EntrypointArgs, EntrypointCall))> = vec![(
            String::from(entrypoint_name),
            (vec![], |_, _| vec![1, 1, 0, 0])
        )];
        let constructors = HashMap::new();
        let contract_address = instance.register_contract(
            None,
            constructors,
            entrypoint.into_iter().collect::<HashMap<_, _>>()
        );

        (
            contract_address,
            String::from(entrypoint_name),
            <u32 as MockVMType>::deser(result).unwrap()
        )
    }
}
