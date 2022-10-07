use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use odra_types::bytesrepr::ToBytes;
use odra_types::{
    bytesrepr::Bytes, event::EventError, Address, CLValue, EventData, OdraError, RuntimeArgs,
};
use odra_types::{VmError, U512};

use crate::account::Account;
use crate::context::ExecutionContext;
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
    ) -> Option<Bytes> {
        self.prepare_call(address);
        // Call contract from register.
        if !self.handle_attached_value(address) {
            return None;
        }
        let result = {
            let register = self.contract_register.read().unwrap();
            register.call(address, String::from(entrypoint), args.clone())
        };

        self.clear_attached_value();
        self.handle_call_result(address, result)
    }

    fn call_constructor(
        &self,
        address: &Address,
        entrypoint: &str,
        args: &RuntimeArgs,
    ) -> Option<Bytes> {
        self.prepare_call(address);
        // Call contract from register.
        if !self.handle_attached_value(address) {
            return None;
        }

        let register = self.contract_register.read().unwrap();
        let result = register.call_constructor(address, String::from(entrypoint), args.clone());
        self.clear_attached_value();
        self.handle_call_result(address, result)
    }

    fn handle_attached_value(&self, address: &Address) -> bool {
        let value = self.attached_value();
        let result = self.transfer_tokens(address, value);
        if !result {
            self.state
                .write()
                .unwrap()
                .set_error(OdraError::VmError(VmError::BalanceExceeded));
            self.clear_attached_value();
        }
        result
    }

    fn prepare_call(&self, address: &Address) {
        let mut state = self.state.write().unwrap();
        // If only one address on the call_stack, record snapshot.
        if state.is_in_caller_context() {
            state.take_snapshot();
            state.clear_error();
        }
        // Put the address on stack.
        state.push_address(address);
    }

    fn handle_call_result(
        &self,
        address: &Address,
        result: Result<Option<Bytes>, OdraError>,
    ) -> Option<Bytes> {
        if result.is_err() {
            self.revert_balance(address);
            self.revert_balance(&self.caller());
        }

        let result = match result {
            Ok(data) => data,
            Err(err) => {
                {
                    self.state.write().unwrap().set_error(err);
                }
                None
            }
        };

        // Drop the address from stack.
        {
            self.state.write().unwrap().pop_address();
        }

        let mut state = self.state.write().unwrap();
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

    pub fn attach_value(&self, amount: U512) {
        self.state.write().unwrap().attach_value(amount);
    }

    pub fn attached_value(&self) -> U512 {
        self.state.read().unwrap().attached_value()
    }

    fn clear_attached_value(&self) {
        self.state.write().unwrap().clear_attached_value();
    }

    pub fn get_address(&self, n: usize) -> Address {
        self.state
            .read()
            .unwrap()
            .accounts()
            .get(n)
            .unwrap()
            .address()
    }

    pub fn token_balance(&self, address: Address) -> U512 {
        let register = self.contract_register.read().unwrap();
        let contract_account = register.get_contract_account(address);
        if let Some(account) = contract_account {
            account.balance()
        } else {
            self.state.read().unwrap().get_balance(address)
        }
    }

    pub fn transfer_tokens(&self, to: &Address, amount: U512) -> bool {
        if amount == U512::zero() {
            return true;
        }
        let mut recipient_account: Option<&mut Account> = None;
        let mut caller_account: Option<&mut Account> = None;
        let caller_address = self.caller();

        let mut register = self.contract_register.write().unwrap();
        for (address, account) in register.get_contract_accounts() {
            if address == to {
                recipient_account = Some(account);
            } else if address == &caller_address {
                caller_account = Some(account);
            }
        }

        let mut state = self.state.write().unwrap();
        for account in state.get_accounts_iter_mut() {
            if recipient_account.is_none() && &account.address() == to {
                recipient_account = Some(account);
            } else if caller_account.is_none() && account.address() == caller_address {
                caller_account = Some(account);
            }
        }

        if let Some(account) = caller_account {
            if account.reduce_balance(amount).is_err() {
                return false;
            }
            if let Some(recipient) = recipient_account {
                if recipient.increase_balance(amount).is_err() {
                    return false;
                }
                return true;
            }
        }

        false
    }

    pub fn self_balance(&self) -> U512 {
        let address = self.callee();

        let contract_register = self.contract_register.read().unwrap();
        let contract_account = contract_register.get_contract_account(address);
        if let Some(account) = contract_account {
            account.balance()
        } else {
            self.state.read().unwrap().get_balance(address)
        }
    }

    fn revert_balance(&self, address: &Address) {
        let mut state = self.state.write().unwrap();
        for account in state.get_accounts_iter_mut() {
            if &account.address() == address {
                account.revert_balance();
                return;
            }
        }

        let mut register = self.contract_register.write().unwrap();
        for (contract_address, account) in register.get_contract_accounts() {
            if address == contract_address {
                account.revert_balance();
                return;
            }
        }
    }
}

#[derive(Clone)]
pub struct MockVmState {
    storage: Storage,
    exec_context: ExecutionContext,
    events: HashMap<Address, Vec<EventData>>,
    contract_counter: u32,
    error: Option<OdraError>,
    block_time: u64,
    attached_value: Option<U512>,
    accounts: Vec<Account>,
}

impl MockVmState {
    fn get_backend_name(&self) -> String {
        "MockVM".to_string()
    }

    fn callee(&self) -> Address {
        *self.exec_context.current()
    }

    fn caller(&self) -> Address {
        *self.exec_context.previous()
    }

    fn set_caller(&mut self, address: &Address) {
        self.pop_address();
        self.push_address(address);
    }

    fn set_var(&mut self, key: &str, value: &CLValue) {
        let ctx = self.exec_context.current();
        self.storage
            .set_value(ctx, &key.to_bytes().unwrap(), value.clone());
    }

    fn get_var(&self, key: &str) -> Option<CLValue> {
        let ctx = self.exec_context.current();
        self.storage.get_value(ctx, &key.to_bytes().unwrap())
    }

    fn set_dict_value(&mut self, dict: &str, key: &[u8], value: &CLValue) {
        let ctx = self.exec_context.current();
        self.storage
            .insert_dict_value(ctx, &dict.to_bytes().unwrap(), key, value.clone());
    }

    fn get_dict_value(&self, dict: &str, key: &[u8]) -> Option<CLValue> {
        let ctx = self.exec_context.current();
        self.storage
            .get_dict_value(ctx, &dict.to_bytes().unwrap(), key)
    }

    fn emit_event(&mut self, event_data: &EventData) {
        let contract_address = self.exec_context.current();
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

    fn push_address(&mut self, address: &Address) {
        self.exec_context.push(*address);
    }

    fn pop_address(&mut self) {
        self.exec_context.drop();
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

    fn attach_value(&mut self, amount: U512) {
        self.attached_value = Some(amount);
    }

    fn attached_value(&self) -> U512 {
        self.attached_value.unwrap_or_default()
    }

    fn clear_attached_value(&mut self) {
        self.attached_value = None;
    }

    fn clear_error(&mut self) {
        self.error = None;
    }

    fn error(&self) -> Option<OdraError> {
        self.error.clone()
    }

    fn is_in_caller_context(&self) -> bool {
        self.exec_context.len() == 1
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
        let account = self
            .accounts
            .iter()
            .find(|account| account.address() == address);
        match account {
            Some(account) => account.balance(),
            None => U512::zero(),
        }
    }

    fn get_accounts_iter_mut(&mut self) -> std::slice::IterMut<'_, Account> {
        self.accounts.iter_mut()
    }

    fn accounts(&self) -> &[Account] {
        self.accounts.as_ref()
    }
}

impl Default for MockVmState {
    fn default() -> Self {
        let mut backend = MockVmState {
            storage: Default::default(),
            exec_context: Default::default(),
            events: Default::default(),
            contract_counter: 0,
            error: None,
            block_time: 0,
            attached_value: None,
            accounts: vec![
                Account::new(Address::new(b"alice"), 100_000.into()),
                Account::new(Address::new(b"bob"), 100_000.into()),
                Account::new(Address::new(b"cab"), 100_000.into()),
                Account::new(Address::new(b"dan"), 100_000.into()),
                Account::new(Address::new(b"ed"), 100_000.into()),
                Account::new(Address::new(b"frank"), 100_000.into()),
                Account::new(Address::new(b"garry"), 100_000.into()),
                Account::new(Address::new(b"garry"), 100_000.into()),
                Account::new(Address::new(b"harry"), 100_000.into()),
                Account::new(Address::new(b"ivan"), 100_000.into()),
            ],
        };
        backend.push_address(&backend.accounts.first().unwrap().address());
        backend
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use odra_types::{
        Address, CLValue, EventData, ExecutionError, OdraError, RuntimeArgs, VmError,
    };

    use crate::{EntrypointArgs, EntrypointCall};

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

        let entrypoint: Vec<(String, (EntrypointArgs, EntrypointCall))> = vec![(
            String::from("abc"),
            (vec![], |_, _| Some(vec![1, 1, 1].into())),
        )];
        let constructors = HashMap::new();
        let address = instance.register_contract(
            None,
            constructors,
            entrypoint.into_iter().collect::<HashMap<_, _>>(),
        );

        // when call an existing entrypoint
        let result = instance.call_contract(&address, "abc", &RuntimeArgs::new());

        // then returns the expected value
        assert_eq!(result, Some(vec![1, 1, 1].into()));
    }

    #[test]
    fn test_call_non_existing_contract() {
        // given an empty vm
        let instance = MockVm::default();

        let address = Address::new(b"random");

        // when call a contract
        instance.call_contract(&address, "abc", &RuntimeArgs::new());

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
        let entrypoint: Vec<(String, (EntrypointArgs, EntrypointCall))> = vec![(
            String::from("abc"),
            (vec![], |_, _| Some(vec![1, 1, 1].into())),
        )];
        let address = instance.register_contract(
            None,
            HashMap::new(),
            entrypoint.into_iter().collect::<HashMap<_, _>>(),
        );

        // when call non-existing entrypoint
        instance.call_contract(&address, "cba", &RuntimeArgs::new());

        // then the vm is in error state
        assert_eq!(
            instance.error(),
            Some(OdraError::VmError(VmError::NoSuchMethod("cba".to_string())))
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

    fn push_address(vm: &MockVm, address: &Address) {
        vm.state.write().unwrap().push_address(address);
    }
}
