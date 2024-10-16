use std::cell::RefCell;
use std::panic::{self, AssertUnwindSafe};
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use anyhow::Result;
use odra_core::callstack::CallstackElement;
use odra_core::casper_types::bytesrepr::{deserialize, deserialize_from_slice, serialize};
use odra_core::casper_types::{CLType, CLValue};
use odra_core::entry_point_callback::EntryPointsCaller;
use odra_core::prelude::*;
use odra_core::CallDef;
use odra_core::EventError;
use odra_core::VmError;
use odra_core::{
    callstack,
    casper_types::{
        bytesrepr::{Bytes, FromBytes, ToBytes},
        PublicKey, SecretKey, U512
    }
};
use odra_core::{ContractContainer, ContractRegister};

use super::odra_vm_state::OdraVmState;
const NAMED_KEY_PREFIX: &str = "NAMED_KEY";

/// Odra in-memory virtual machine.
#[derive(Default)]
pub struct OdraVm {
    state: Arc<RwLock<OdraVmState>>,
    contract_register: Arc<RwLock<ContractRegister>>
}

impl OdraVm {
    /// Creates a new instance of OdraVm.
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self::default()))
    }

    /// Adds a new contract to the virtual machine.
    pub fn register_contract(&self, name: &str, entry_points_caller: EntryPointsCaller) -> Address {
        // Create a new address.
        let address = { self.state.write().unwrap().next_contract_address() };
        // Register new contract under the new address.
        {
            let contract = ContractContainer::new(entry_points_caller);
            self.contract_register
                .write()
                .unwrap()
                .add(address, contract);
            self.state
                .write()
                .unwrap()
                .set_balance(address, U512::zero());
        }

        address
    }

    pub(crate) fn post_install(&self, address: Address) {
        self.contract_register
            .write()
            .unwrap()
            .post_install(&address);
    }

    /// Calls a contract with the specified address and call definition.
    ///
    /// Returns the result of the call as [Bytes].
    /// If the call fails, the virtual machine is in error state, all the changes are reverted.
    pub fn call_contract(&self, address: Address, call_def: CallDef) -> Bytes {
        self.prepare_call(address, &call_def);
        // Call contract from register.
        if call_def.amount() > U512::zero() {
            let status = self.checked_transfer_tokens(&self.caller(), &address, &call_def.amount());
            if let Err(err) = status {
                self.revert(err);
            }
        }
        let result = self
            .contract_register
            .read()
            .unwrap()
            .call(&address, call_def);

        match result {
            Err(err) => self.revert(err),
            Ok(bytes) => self.handle_call_result(bytes)
        }
    }

    /// Stops the execution of the virtual machine and reverts all the changes.
    pub fn revert(&self, error: OdraError) -> ! {
        let mut revert_msg = String::from("");
        if let CallstackElement::ContractCall { address, call_def } = self.callstack_tip() {
            revert_msg = format!("{:?}::{}", address, call_def.entry_point());
        }

        let mut state = self.state.write().unwrap();
        state.set_error(error.clone());
        state.clear_callstack();
        if state.is_in_caller_context() {
            state.restore_snapshot();
        }
        drop(state);

        panic!("Revert: {:?} - {}", error, revert_msg);
    }

    /// Returns the error of the virtual machine.
    ///
    /// If the virtual machine is not in error state, returns `None`.
    pub fn error(&self) -> Option<OdraError> {
        self.state.read().unwrap().error()
    }

    /// Returns the callee, i.e. the currently executing contract.
    pub fn self_address(&self) -> Address {
        self.state.read().unwrap().callee()
    }

    /// Retrieves from the state the address of the current caller.
    pub fn caller(&self) -> Address {
        self.state.read().unwrap().caller()
    }

    /// Retrieves the first element from the callstack.
    pub fn callstack_tip(&self) -> CallstackElement {
        self.state.read().unwrap().callstack_tip().clone()
    }

    /// Gets the value of the named argument.
    ///
    /// The argument must be present in the call definition.
    pub fn get_named_arg(&self, name: &str) -> OdraResult<Vec<u8>> {
        match self.state.read().unwrap().callstack_tip() {
            CallstackElement::Account(_) => todo!(),
            CallstackElement::ContractCall { call_def, .. } => call_def
                .args()
                .get(name)
                .map(|arg| arg.inner_bytes().to_vec())
                .ok_or_else(|| OdraError::ExecutionError(ExecutionError::MissingArg))
        }
    }

    /// Overrides the current caller address.
    pub fn set_caller(&self, caller: Address) {
        self.state.write().unwrap().set_caller(caller);
    }

    /// Sets the value of the named argument.
    ///
    /// If the global state write fails, the virtual machine is in error state.
    pub fn set_var(&self, key: &[u8], value: Bytes) {
        self.state.write().unwrap().set_var(key, value);
    }

    /// Gets the value of the named variable from the global state.
    ///
    /// Returns `None` if the variable does not exist.
    /// If the global state read fails, the virtual machine is in error state.
    pub fn get_var(&self, key: &[u8]) -> Option<Bytes> {
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

    /// Sets the value of the named key.
    pub fn set_named_key(&self, name: &str, value: CLValue) {
        let key = Self::key_of_named_key(name);
        self.set_var(key.as_bytes(), Bytes::from(value.inner_bytes().as_slice()));
    }

    /// Retrieves the value of the named key.
    pub fn get_named_key(&self, name: &str) -> Option<Bytes> {
        let key = Self::key_of_named_key(name);
        self.get_var(key.as_bytes())
    }

    /// Sets the value of the dictionary item.
    pub fn set_dict_value(&self, dict: &str, key: &[u8], value: CLValue) {
        self.state.write().unwrap().set_dict_value(
            dict.as_bytes(),
            key,
            Bytes::from(value.inner_bytes().as_slice())
        );
    }

    /// Removes the dictionary from the global state.
    pub fn remove_dictionary(&self, dictionary_name: &str) {
        self.state
            .write()
            .unwrap()
            .remove_dictionary(dictionary_name.as_bytes());
    }

    /// Gets the value of the dictionary item.
    ///
    /// Returns `None` if the dictionary or the key does not exist.
    /// If the dictionary or the key does not exist, the virtual machine is in error state.
    pub fn get_dict_value(&self, dict: &str, key: &[u8]) -> Option<Bytes> {
        let result = {
            self.state
                .read()
                .unwrap()
                .get_dict_value(dict.as_bytes(), key)
        };
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

    /// Writes an event data to the global state.
    pub fn emit_event(&self, event_data: &Bytes) {
        self.state.write().unwrap().emit_event(event_data);
    }

    /// Gets the event emitted by the given address at the given index from the global state.
    pub fn get_event(&self, address: &Address, index: u32) -> Result<Bytes, EventError> {
        self.state.read().unwrap().get_event(address, index)
    }

    /// Gets the number of events emitted by the given address from the global state.
    pub fn get_events_count(&self, address: &Address) -> u32 {
        self.state.read().unwrap().get_events_count(address)
    }

    /// Attaches the given amount of tokens to the current call from the global state.
    pub fn attach_value(&self, amount: U512) {
        self.state.write().unwrap().attach_value(amount);
    }

    /// Gets the current block time.
    pub fn get_block_time(&self) -> u64 {
        self.state.read().unwrap().block_time()
    }

    /// Advances the block time by the given number of milliseconds.
    pub fn advance_block_time_by(&self, milliseconds: u64) {
        self.state
            .write()
            .unwrap()
            .advance_block_time_by(milliseconds)
    }

    /// Gets the value attached to the current call.
    pub fn attached_value(&self) -> U512 {
        self.state.read().unwrap().attached_value()
    }

    /// Gets the address of the account at the given index.
    pub fn get_account(&self, n: usize) -> Address {
        self.state.read().unwrap().accounts.get(n).cloned().unwrap()
    }

    /// Reads the balance of the given address from the global state.
    pub fn balance_of(&self, address: &Address) -> U512 {
        self.state.read().unwrap().balance_of(address)
    }

    /// Updates the balances of the given address and the current address in the global state.
    ///
    /// If the global state write fails(wrong address, the passed amount exceeds the disposable balance),
    /// the virtual machine reverts.
    pub fn transfer_tokens(&self, to: &Address, amount: &U512) {
        if amount.is_zero() {
            return;
        }

        let from = &self.self_address();

        let mut transfer_error = None;
        {
            let mut state = self.state.write().unwrap();
            if state.transfer(from, to, amount).is_err() {
                transfer_error = Some(OdraError::VmError(VmError::BalanceExceeded));
            }
        }

        if let Some(result) = transfer_error {
            self.revert(result);
        }
    }

    /// Updates the balances of the given address and the current address in the global state.
    ///
    /// Similar to [transfer_tokens](OdraVm::transfer_tokens), but does not revert if the passed amount exceeds the disposable balance,
    /// but returns an error.
    pub fn checked_transfer_tokens(
        &self,
        from: &Address,
        to: &Address,
        amount: &U512
    ) -> OdraResult<()> {
        if amount.is_zero() {
            return Ok(());
        }

        let mut state = self.state.write().unwrap();
        if state.transfer(from, to, amount).is_err() {
            return Err(OdraError::VmError(VmError::BalanceExceeded));
        }

        Ok(())
    }

    /// Reads the balance of the current contract from the global state.
    pub fn self_balance(&self) -> U512 {
        let address = self.self_address();
        self.state.read().unwrap().balance_of(&address)
    }

    /// Reads the public key of a given address from the global state.
    pub fn public_key(&self, address: &Address) -> PublicKey {
        self.state.read().unwrap().public_key(address)
    }

    /// Signs a message using the secret key associated with the public key of a given address.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to be signed.
    /// * `address` - The address which public key is used to sign the message.
    ///
    /// # Returns
    ///
    /// The signature of the message as a byte array.
    pub fn sign_message(&self, message: &Bytes, address: &Address) -> Bytes {
        let public_key = self.public_key(address);
        let signature = odra_core::casper_types::crypto::sign(
            message,
            self.state.read().unwrap().secret_key(address),
            &public_key
        )
        .to_bytes()
        .unwrap();
        signature.into()
    }

    fn prepare_call(&self, address: Address, call_def: &CallDef) {
        let mut state = self.state.write().unwrap();
        // If only one address on the call_stack, record snapshot.
        if state.is_in_caller_context() {
            state.take_snapshot();
            state.clear_error();
        }
        // Put the address on stack.

        let element = CallstackElement::new_contract_call(address, call_def.clone());
        state.push_callstack_element(element);
    }

    fn handle_call_result(&self, result: Bytes) -> Bytes {
        let mut state = self.state.write().unwrap();

        // Drop the address from stack.
        state.pop_callstack_element();

        // If only one address on the call_stack, drop the snapshot
        if state.is_in_caller_context() {
            state.drop_snapshot();
        }
        result
    }

    fn key_of_named_key(name: &str) -> String {
        let key = format!("{}_{}", NAMED_KEY_PREFIX, name);
        key
    }
}

#[cfg(test)]
mod tests {
    use odra_core::callstack::CallstackElement;
    use odra_core::casper_types::bytesrepr::{Bytes, ToBytes};
    use odra_core::{
        entry_point_callback::{EntryPoint, EntryPointsCaller},
        utils::serialize
    };

    use std::collections::BTreeMap;

    use odra_core::casper_types::bytesrepr::FromBytes;
    use odra_core::casper_types::{CLValue, RuntimeArgs, U512};
    use odra_core::host::HostEnv;
    use odra_core::{prelude::*, CallDef, VmError};

    use crate::vm::utils;
    use crate::{OdraVm, OdraVmHost};

    const TEST_ENTRY_POINT: &str = "abc";

    #[test]
    fn contracts_have_different_addresses() {
        // given a new instance
        let instance = OdraVm::default();
        // when register two contracts with the same entrypoints
        let address1 = instance.register_contract("A", test_caller(TEST_ENTRY_POINT));
        let address2 = instance.register_contract("B", test_caller(TEST_ENTRY_POINT));

        // then addresses are different
        assert_ne!(address1, address2);
    }

    #[test]
    fn addresses_have_different_type() {
        // given an address of a contract and an address of an account
        let instance = OdraVm::default();

        let account_address = instance.get_account(0);
        let contract_address = setup_contract(&instance, TEST_ENTRY_POINT);

        // Then the contract address is a contract
        assert!(contract_address.is_contract());
        // And the account address is not a contract
        assert!(!account_address.is_contract());
    }

    #[test]
    fn test_contract_call() {
        // given an instance with a registered contract having one entrypoint
        let instance = OdraVm::default();
        let contract_address = setup_contract(&instance, TEST_ENTRY_POINT);

        // when call an existing entrypoint
        let result = instance.call_contract(
            contract_address,
            CallDef::new(TEST_ENTRY_POINT, false, RuntimeArgs::new())
        );
        let expected_result: Bytes = test_call_result();

        // then returns the expected value
        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_transfer() {
        // given an empty vm and two addresses
        let instance = OdraVm::default();
        let from = instance.get_account(0);
        let from_balance = instance.balance_of(&from);
        let to = instance.get_account(1);
        let to_balance = instance.balance_of(&to);
        let amount = U512::from(100);

        // when transferring tokens
        instance.transfer_tokens(&to, &amount);

        // then the balance of the sender is decreased and the balance of the receiver is increased
        assert_eq!(instance.balance_of(&from), from_balance - amount);
        assert_eq!(instance.balance_of(&to), to_balance + amount);
    }

    #[test]
    #[should_panic]
    fn test_transfer_too_much() {
        // given an empty vm and two addresses
        let instance = OdraVm::default();
        let from = instance.get_account(0);
        let from_balance = instance.balance_of(&from);
        let to = instance.get_account(1);
        let to_balance = instance.balance_of(&to);
        let amount = from_balance + 1;

        // when transferring tokens the vm should panic
        instance.transfer_tokens(&to, &amount);
    }

    #[test]
    fn test_call_non_existing_contract() {
        // given an empty vm
        let instance = OdraVm::default();

        let address = utils::contract_address_from_u32(42);

        // when call a contract
        let call_def = CallDef::new(TEST_ENTRY_POINT, false, RuntimeArgs::new());
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            instance.call_contract(address, call_def)
        }));

        // then the vm is in error state
        assert_eq!(
            instance.error(),
            Some(OdraError::VmError(VmError::InvalidContractAddress))
        );
    }

    #[test]
    fn test_call_non_existing_entrypoint() {
        // given an instance with a registered contract having one entrypoint
        let instance = OdraVm::default();
        let invalid_entry_point_name = "aaa";
        let contract_address = setup_contract(&instance, TEST_ENTRY_POINT);

        // when call non-existing entrypoint
        let call_def = CallDef::new(invalid_entry_point_name, false, RuntimeArgs::new());
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            instance.call_contract(contract_address, call_def)
        }));

        // then the vm is in error state
        assert_eq!(
            instance.error(),
            Some(OdraError::VmError(VmError::NoSuchMethod(
                invalid_entry_point_name.to_string()
            )))
        );
    }

    #[test]
    fn test_caller_switching() {
        // given an empty instance
        let instance = OdraVm::default();

        // when set a new caller
        let new_caller = utils::account_address_from_str("ff");
        instance.set_caller(new_caller);
        // put a fake contract on stack
        push_address(&instance, &new_caller);

        // then the caller is set
        assert_eq!(instance.caller(), new_caller);
    }

    #[test]
    #[should_panic]
    fn test_revert() {
        let instance = OdraVm::default();
        instance.revert(ExecutionError::User(1).into());
    }

    #[test]
    fn test_read_write_value() {
        // given an empty instance
        let instance = OdraVm::default();

        // when set a value
        let key = b"key";
        let value = 32u8.to_bytes().map(Bytes::from).unwrap();
        instance.set_var(key, value.clone());

        // then the value can be read
        assert_eq!(instance.get_var(key), Some(value));
        // then the value under unknown key does not exist
        assert_eq!(instance.get_var(b"other_key"), None);
    }

    #[test]
    fn test_read_write_dict() {
        // given an empty instance
        let instance = OdraVm::default();

        // when set a value
        let dict = "dict";
        let key = b"key";
        let value = CLValue::from_t("value").unwrap();
        instance.set_dict_value(dict, key, value.clone());

        // then the value can be read
        assert_eq!(
            instance.get_dict_value(dict, key),
            Some(Bytes::from(value.inner_bytes().as_slice()))
        );
        // then the value under the key in unknown dict does not exist
        assert_eq!(instance.get_dict_value("other_dict", key), None);
        // then the value under unknown key does not exist
        assert_eq!(instance.get_dict_value(dict, b"other_key"), None);
    }

    #[test]
    fn test_named_key() {
        // given an empty instance
        let instance = OdraVm::default();

        // when set a value
        let name = "name";
        let value = CLValue::from_t("value").unwrap();
        instance.set_named_key(name, value.clone());

        // then the value can be read
        assert_eq!(
            instance.get_named_key(name),
            Some(Bytes::from(value.inner_bytes().as_slice()))
        );
        // then the value under unknown name does not exist
        assert_eq!(instance.get_named_key("other_name"), None);
    }

    #[test]
    fn events() {
        // given an empty instance
        let instance = OdraVm::default();

        let first_contract_address = utils::account_address_from_str("abc");
        // put a contract on stack
        push_address(&instance, &first_contract_address);

        let first_event: Bytes = vec![1, 2, 3].into();
        let second_event: Bytes = vec![4, 5, 6].into();
        instance.emit_event(&first_event);
        instance.emit_event(&second_event);

        let second_contract_address = utils::account_address_from_str("bca");
        // put a next contract on stack
        push_address(&instance, &second_contract_address);

        let third_event: Bytes = vec![7, 8, 9].into();
        let fourth_event: Bytes = vec![11, 22, 33].into();
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
        // given an empty instance
        let instance = OdraVm::default();
        let contract_address = setup_contract(&instance, TEST_ENTRY_POINT);

        // when push a contract into the stack
        let contract_address = utils::contract_address_from_u32(100);
        push_address(&instance, &contract_address);

        // then the contract address in the callee
        assert_eq!(instance.self_address(), contract_address);
    }

    #[test]
    fn test_call_contract_with_amount() {
        // given an instance with a registered contract having one entrypoint
        let instance = OdraVm::default();
        let contract_address = setup_contract(&instance, TEST_ENTRY_POINT);

        // when call a contract with the whole balance of the caller
        let caller = instance.get_account(0);
        let caller_balance = instance.balance_of(&caller);

        let call_def =
            CallDef::new(TEST_ENTRY_POINT, false, RuntimeArgs::new()).with_amount(caller_balance);
        instance.call_contract(contract_address, call_def);

        // then the contract has the caller tokens and the caller balance is zero
        assert_eq!(instance.balance_of(&contract_address), caller_balance);
        assert_eq!(instance.balance_of(&caller), U512::zero());
    }

    #[test]
    #[should_panic(expected = "VmError(BalanceExceeded)")]
    fn test_call_contract_with_amount_exceeding_balance() {
        // given an instance with a registered contract having one entrypoint
        let instance = OdraVm::default();
        let contract_address = setup_contract(&instance, TEST_ENTRY_POINT);

        let caller = instance.get_account(0);
        let caller_balance = instance.balance_of(&caller);

        // when call a contract with the amount exceeding caller's balance
        let call_def = CallDef::new(TEST_ENTRY_POINT, false, RuntimeArgs::new())
            .with_amount(caller_balance + 1);
        instance.call_contract(contract_address, call_def);
    }

    fn push_address(vm: &OdraVm, address: &Address) {
        let element = CallstackElement::new_account(*address);
        vm.state.write().unwrap().push_callstack_element(element);
    }

    fn test_call_result() -> Bytes {
        vec![1, 1, 0, 0].into()
    }

    fn setup_contract(instance: &OdraVm, entry_point_name: &str) -> Address {
        let caller = test_caller(entry_point_name);
        instance.register_contract("contract", caller)
    }

    fn test_caller(entry_point_name: &str) -> EntryPointsCaller {
        let vm = OdraVm::new();
        let host_env = OdraVmHost::new(vm);
        let env = HostEnv::new(host_env);
        let entry_point = EntryPoint::new_payable(String::from(entry_point_name), vec![]);
        EntryPointsCaller::new(env, vec![entry_point], |_, _| Ok(test_call_result()))
    }
}
