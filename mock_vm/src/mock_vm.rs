use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use odra_types::bytesrepr::ToBytes;
use odra_types::{
    bytesrepr::Bytes, event::EventError, Address, CLValue, EventData, OdraError, RuntimeArgs,
};
use odra_types::{VmError, U512};

use crate::balance::Balance;
use crate::callstack::{Callstack, CallstackElement};
use crate::contract_container::{ContractContainer, EntrypointArgs, EntrypointCall};
use crate::contract_register::ContractRegister;
use crate::storage::Storage;

#[derive(Default)]
pub struct MockVm {
    state: Arc<RwLock<MockVmState>>,
    contract_register: Arc<RwLock<ContractRegister>>,
}

impl MockVm {
    pub fn register_contract(
        &self,
        constructor: Option<(String, RuntimeArgs, EntrypointCall)>,
        constructors: HashMap<String, (EntrypointArgs, EntrypointCall)>,
        entrypoints: HashMap<String, (EntrypointArgs, EntrypointCall)>,
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

        // Call init if needed.
        if let Some(constructor) = constructor {
            let (constructor_name, args, _) = constructor;
            self.call_constructor(&address, &constructor_name, &args);
        }
        address
    }

    pub fn call_contract(
        &self,
        address: &Address,
        entrypoint: &str,
        args: &RuntimeArgs,
        amount: Option<U512>,
    ) -> Option<Bytes> {
        self.prepare_call(address, amount);

        // Call contract from register.
        if let Some(amount) = amount {
            let success = self.transfer_tokens(&self.caller(), address, amount);
            if !success {
                return self.handle_call_result(Err(OdraError::VmError(VmError::BalanceExceeded)));
            }
        }

        let result = self.contract_register.read().unwrap().call(
            address,
            String::from(entrypoint),
            args.clone(),
        );

        self.handle_call_result(result)
    }

    fn call_constructor(
        &self,
        address: &Address,
        entrypoint: &str,
        args: &RuntimeArgs,
    ) -> Option<Bytes> {
        self.prepare_call(address, None);
        // Call contract from register.
        let register = self.contract_register.read().unwrap();
        let result = register.call_constructor(address, String::from(entrypoint), args.clone());
        self.handle_call_result(result)
    }

    fn prepare_call(&self, address: &Address, amount: Option<U512>) {
        let mut state = self.state.write().unwrap();
        // If only one address on the call_stack, record snapshot.
        if state.is_in_caller_context() {
            state.take_snapshot();
            state.clear_error();
        }
        // Put the address on stack.
        let element = CallstackElement::new(*address, amount);
        state.push_callstack_element(element);
    }

    fn handle_call_result(&self, result: Result<Option<Bytes>, OdraError>) -> Option<Bytes> {
        let mut state = self.state.write().unwrap();
        let result = match result {
            Ok(data) => data,
            Err(err) => {
                state.set_error(err);
                None
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
            None
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

    pub fn set_caller(&self, caller: &Address) {
        self.state.write().unwrap().set_caller(caller);
    }

    pub fn set_var(&self, key: &str, value: &CLValue) {
        self.state.write().unwrap().set_var(key, value);
    }

    pub fn get_var(&self, key: &str) -> Option<CLValue> {
        self.state.read().unwrap().get_var(key)
    }

    pub fn set_dict_value(&self, dict: &str, key: &[u8], value: &CLValue) {
        self.state.write().unwrap().set_dict_value(dict, key, value);
    }

    pub fn get_dict_value(&self, dict: &str, key: &[u8]) -> Option<CLValue> {
        self.state.read().unwrap().get_dict_value(dict, key)
    }

    pub fn emit_event(&self, event_data: &EventData) {
        self.state.write().unwrap().emit_event(event_data);
    }

    pub fn get_event(&self, address: &Address, index: i32) -> Result<EventData, EventError> {
        self.state.read().unwrap().get_event(address, index)
    }

    pub fn get_block_time(&self) -> u64 {
        self.state.read().unwrap().block_time()
    }

    pub fn advance_block_time_by(&self, seconds: u64) {
        self.state.write().unwrap().advance_block_time_by(seconds)
    }

    pub fn attached_value(&self) -> U512 {
        self.state.read().unwrap().attached_value()
    }

    pub fn get_address(&self, n: usize) -> Address {
        self.state.read().unwrap().accounts.get(n).cloned().unwrap()
    }

    pub fn token_balance(&self, address: Address) -> U512 {
        self.state.read().unwrap().get_balance(address)
    }

    pub fn transfer_tokens(&self, from: &Address, to: &Address, amount: U512) -> bool {
        if amount == U512::zero() {
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

    pub fn self_balance(&self) -> U512 {
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
    accounts: Vec<Address>,
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

    fn set_caller(&mut self, address: &Address) {
        self.pop_callstack_element();
        self.push_callstack_element(CallstackElement::new(*address, None));
    }

    fn set_var(&mut self, key: &str, value: &CLValue) {
        let ctx = &self.callstack.current().address;
        self.storage
            .set_value(ctx, &key.to_bytes().unwrap(), value.clone());
    }

    fn get_var(&self, key: &str) -> Option<CLValue> {
        let ctx = &self.callstack.current().address;
        self.storage.get_value(ctx, &key.to_bytes().unwrap())
    }

    fn set_dict_value(&mut self, dict: &str, key: &[u8], value: &CLValue) {
        let ctx = &self.callstack.current().address;
        self.storage
            .insert_dict_value(ctx, &dict.to_bytes().unwrap(), key, value.clone());
    }

    fn get_dict_value(&self, dict: &str, key: &[u8]) -> Option<CLValue> {
        let ctx = &self.callstack.current().address;
        self.storage
            .get_dict_value(ctx, &dict.to_bytes().unwrap(), key)
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

    fn get_event(&self, address: &Address, index: i32) -> Result<EventData, EventError> {
        let events = self.events.get(address);
        if events.is_none() {
            return Err(EventError::IndexOutOfBounds);
        }
        let events: &Vec<EventData> = events.unwrap();
        let event_position = odra_utils::event_absolute_position(events.len(), index)?;
        Ok(events.get(event_position as usize).unwrap().clone())
    }

    fn push_callstack_element(&mut self, element: CallstackElement) {
        self.callstack.push(element);
    }

    fn pop_callstack_element(&mut self) {
        self.callstack.pop();
    }

    fn next_contract_address(&mut self) -> Address {
        let address = Address::new(&self.contract_counter.to_be_bytes());
        self.contract_counter += 1;
        address
    }

    fn get_contract_namespace(&self) -> String {
        self.contract_counter.to_string()
    }

    fn set_error(&mut self, error: OdraError) {
        if self.error.is_none() {
            self.error = Some(error);
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

    fn advance_block_time_by(&mut self, seconds: u64) {
        self.block_time += seconds;
    }

    fn get_balance(&self, address: Address) -> U512 {
        self.storage
            .balance_of(&address)
            .map(|b| b.value())
            .unwrap_or_default()
    }

    fn set_balance(&mut self, address: Address, amount: U512) {
        self.storage.set_balance(address, Balance::new(amount));
    }

    fn increase_balance(&mut self, address: &Address, amount: U512) -> Result<()> {
        self.storage.increase_balance(address, amount)
    }

    fn reduce_balance(&mut self, address: &Address, amount: U512) -> Result<()> {
        self.storage.reduce_balance(address, amount)
    }
}

impl Default for MockVmState {
    fn default() -> Self {
        let addresses = vec![
            Address::new(b"alice"),
            Address::new(b"bob"),
            Address::new(b"cab"),
            Address::new(b"dan"),
            Address::new(b"ed"),
            Address::new(b"frank"),
            Address::new(b"garry"),
            Address::new(b"garry"),
            Address::new(b"harry"),
            Address::new(b"ivan"),
        ];

        let mut balances = HashMap::<Address, Balance>::new();
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
            accounts: addresses.clone(),
        };
        backend.push_callstack_element(CallstackElement::new(*addresses.first().unwrap(), None));
        backend
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use odra_types::{
        bytesrepr::Bytes, Address, CLValue, EventData, ExecutionError, OdraError, RuntimeArgs,
        VmError, U512,
    };

    use crate::{callstack::CallstackElement, EntrypointArgs, EntrypointCall};

    use super::MockVm;

    #[test]
    fn contracts_have_different_addresses() {
        // given a new instance
        let instance = MockVm::default();

        // when register two contracts with the same entrypoints
        let entrypoint: Vec<(String, (EntrypointArgs, EntrypointCall))> =
            vec![(String::from("abc"), (vec![], |_, _| None))];
        let entrypoints = entrypoint.into_iter().collect::<HashMap<_, _>>();
        let constructors = HashMap::new();

        let address1 = instance.register_contract(None, constructors.clone(), entrypoints.clone());
        let address2 = instance.register_contract(None, constructors, entrypoints);

        // then addresses are different
        assert_ne!(address1, address2);
    }

    #[test]
    fn test_contract_call() {
        // given an instance with a registered contract having one entrypoint
        let instance = MockVm::default();

        let (contract_address, entrypoint, call_result) = setup_contract(&instance);

        // when call an existing entrypoint
        let result =
            instance.call_contract(&contract_address, &entrypoint, &RuntimeArgs::new(), None);

        // then returns the expected value
        assert_eq!(result, call_result);
    }

    #[test]
    fn test_call_non_existing_contract() {
        // given an empty vm
        let instance = MockVm::default();

        let address = Address::new(b"random");

        // when call a contract
        instance.call_contract(&address, "abc", &RuntimeArgs::new(), None);

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
        instance.call_contract(
            &contract_address,
            &invalid_entrypoint,
            &RuntimeArgs::new(),
            None,
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
        let instance = MockVm::default();
        let new_caller = Address::new(b"new caller");
        instance.set_caller(&new_caller);
        // put a contract on stack
        push_address(&instance, &new_caller);

        assert_eq!(instance.caller(), new_caller);
    }

    #[test]
    fn test_revert() {
        let instance = MockVm::default();

        instance.revert(ExecutionError::new(1, "err").into());

        assert_eq!(instance.error(), Some(ExecutionError::new(1, "err").into()));
    }

    #[test]
    fn test_read_write_value() {
        let instance = MockVm::default();
        let key = "key";
        let value = CLValue::from_t(32u8).unwrap();

        instance.set_var(key, &value);

        assert_eq!(instance.get_var(key), Some(value));
        assert_eq!(instance.get_var("other_key"), None);
    }

    #[test]
    fn test_read_write_dict() {
        let instance = MockVm::default();
        let dict = "dict";
        let key: [u8; 2] = [1, 2];
        let value = CLValue::from_t(32u8).unwrap();

        instance.set_dict_value(dict, &key, &value);

        assert_eq!(instance.get_dict_value(dict, &key), Some(value));
        assert_eq!(instance.get_dict_value("other_dict", &key), None);
        assert_eq!(instance.get_dict_value(dict, &[]), None);
    }

    #[test]
    fn events() {
        let instance = MockVm::default();

        let first_contract_address = Address::new(b"contract");
        // put a contract on stack
        push_address(&instance, &first_contract_address);

        let first_event: EventData = vec![1, 2, 3];
        let second_event: EventData = vec![4, 5, 6];
        instance.emit_event(&first_event);
        instance.emit_event(&second_event);

        let second_contract_address = Address::new(b"contract2");
        // put a next contract on stack
        push_address(&instance, &second_contract_address);

        let third_event: EventData = vec![7, 8, 9];
        let fourth_event: EventData = vec![11, 22, 33];
        instance.emit_event(&third_event);
        instance.emit_event(&fourth_event);

        assert_eq!(
            instance.get_event(&first_contract_address, 0),
            Ok(first_event)
        );
        assert_eq!(
            instance.get_event(&first_contract_address, 1),
            Ok(second_event)
        );

        assert_eq!(
            instance.get_event(&second_contract_address, 0),
            Ok(third_event)
        );
        assert_eq!(
            instance.get_event(&second_contract_address, 1),
            Ok(fourth_event)
        );
    }

    #[test]
    fn test_current_contract_address() {
        let instance = MockVm::default();
        let contract_address = Address::new(b"contract");
        // put a contract on stack
        push_address(&instance, &contract_address);

        assert_eq!(instance.callee(), contract_address);
    }

    #[test]
    fn test_call_contract_with_amount() {
        let instance = MockVm::default();

        let (contract_address, entrypoint_name, _) = setup_contract(&instance);

        let caller = instance.get_address(0);
        let caller_balance = instance.token_balance(caller);

        instance.call_contract(
            &contract_address,
            &entrypoint_name,
            &RuntimeArgs::new(),
            Some(caller_balance),
        );

        assert_eq!(instance.token_balance(contract_address), caller_balance);

        assert_eq!(instance.token_balance(caller), U512::zero());
    }

    #[test]
    fn test_call_contract_with_amount_exceeding_balance() {
        let instance = MockVm::default();

        let (contract_address, entrypoint_name, _) = setup_contract(&instance);

        let caller = instance.get_address(0);
        let caller_balance = instance.token_balance(caller);

        instance.call_contract(
            &contract_address,
            &entrypoint_name,
            &RuntimeArgs::new(),
            Some(caller_balance + U512::one()),
        );

        assert_eq!(
            instance.error(),
            Some(OdraError::VmError(VmError::BalanceExceeded))
        );
    }

    fn push_address(vm: &MockVm, address: &Address) {
        let element = CallstackElement::new(*address, None);
        vm.state.write().unwrap().push_callstack_element(element);
    }

    fn setup_contract(instance: &MockVm) -> (Address, String, Option<Bytes>) {
        let entrypoint_name = "abc";
        let result: Bytes = vec![1, 1, 1].into();

        let entrypoint: Vec<(String, (EntrypointArgs, EntrypointCall))> = vec![(
            String::from(entrypoint_name),
            (vec![], |_, _| Some(vec![1, 1, 1].into())),
        )];
        let constructors = HashMap::new();
        let contract_address = instance.register_contract(
            None,
            constructors,
            entrypoint.into_iter().collect::<HashMap<_, _>>(),
        );

        (
            contract_address,
            String::from(entrypoint_name),
            Some(result),
        )
    }
}
