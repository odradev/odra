use std::cell::RefCell;
use std::panic::{self, AssertUnwindSafe};
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use anyhow::Result;
use odra_core::callstack::CallstackElement;
use odra_core::entry_point_callback::EntryPointsCaller;
use odra_core::EventError;
use odra_core::{
    callstack,
    casper_types::{
        bytesrepr::{Bytes, FromBytes, ToBytes},
        PublicKey, SecretKey, U512
    },
    Address, ExecutionError
};
use odra_core::{CallDef, OdraResult};
use odra_core::{ContractContainer, ContractRegister};
use odra_core::{OdraError, VmError};

use super::odra_vm_state::OdraVmState;

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

        self.handle_call_result(result)
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
    pub fn get_named_arg(&self, name: &str) -> Vec<u8> {
        match self.state.read().unwrap().callstack_tip() {
            CallstackElement::Account(_) => todo!(),
            CallstackElement::ContractCall { call_def, .. } => {
                call_def.args().get(name).unwrap().inner_bytes().to_vec()
            }
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

    /// Sets the value of the dictionary item.
    pub fn set_dict_value(&self, dict: &[u8], key: &[u8], value: Bytes) {
        self.state.write().unwrap().set_dict_value(dict, key, value);
    }

    /// Gets the value of the dictionary item.
    ///
    /// Returns `None` if the dictionary or the key does not exist.
    /// If the dictionary or the key does not exist, the virtual machine is in error state.
    pub fn get_dict_value(&self, dict: &[u8], key: &[u8]) -> Option<Bytes> {
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

        let mut state = self.state.write().unwrap();
        if state.reduce_balance(from, amount).is_err() {
            self.revert(OdraError::VmError(VmError::BalanceExceeded))
        }
        if state.increase_balance(to, amount).is_err() {
            self.revert(OdraError::VmError(VmError::BalanceExceeded))
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
        if state.reduce_balance(from, amount).is_err() {
            return Err(OdraError::VmError(VmError::BalanceExceeded));
        }
        if state.increase_balance(to, amount).is_err() {
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

    fn handle_call_result(&self, result: OdraResult<Bytes>) -> Bytes {
        let mut state = self.state.write().unwrap();
        let result = match result {
            Ok(data) => data,
            Err(err) => {
                state.set_error(err);
                Bytes::new()
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
            Bytes::new()
        }
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
    use odra_core::casper_types::{RuntimeArgs, U512};
    use odra_core::host::HostEnv;
    use odra_core::Address;
    use odra_core::CallDef;
    use odra_core::{ExecutionError, OdraError, VmError};

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
    fn test_call_non_existing_contract() {
        // given an empty vm
        let instance = OdraVm::default();

        let address = utils::contract_address_from_u32(42);

        // when call a contract
        let call_def = CallDef::new(TEST_ENTRY_POINT, false, RuntimeArgs::new());
        instance.call_contract(address, call_def);

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

        instance.call_contract(contract_address, call_def);

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
        let dict = b"dict";
        let key: [u8; 2] = [1, 2];
        let value = 32u8.to_bytes().map(Bytes::from).unwrap();
        instance.set_dict_value(dict, &key, value.clone());

        // then the value can be read
        assert_eq!(instance.get_dict_value(dict, &key), Some(value));
        // then the value under the key in unknown dict does not exist
        assert_eq!(instance.get_dict_value(b"other_dict", &key), None);
        // then the value under unknown key does not exist
        assert_eq!(instance.get_dict_value(dict, &[]), None);
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
        let entry_point = EntryPoint::new(String::from(entry_point_name), vec![]);
        EntryPointsCaller::new(env, vec![entry_point], |_, _| Ok(test_call_result()))
    }
}
