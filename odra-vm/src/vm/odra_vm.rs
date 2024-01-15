use std::cell::RefCell;
use std::panic::{self, AssertUnwindSafe};
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use anyhow::Result;
use odra_core::call_def::CallDef;
use odra_core::entry_point_callback::EntryPointsCaller;
use odra_core::event::EventError;
use odra_core::{
    casper_types::{
        bytesrepr::{FromBytes, ToBytes},
        U512
    },
    Address, Bytes, ExecutionError, PublicKey, SecretKey
};
use odra_core::{OdraError, VmError};
use odra_core::contract_container::ContractContainer;
use odra_core::contract_register::ContractRegister;

use super::callstack::{CallstackElement, Entrypoint};
use super::odra_vm_state::OdraVmState;

#[derive(Default)]
pub struct OdraVm {
    state: Arc<RwLock<OdraVmState>>,
    contract_register: Arc<RwLock<ContractRegister>>
}

impl OdraVm {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self::default()))
    }

    pub fn register_contract(&self, name: &str, entry_points_caller: EntryPointsCaller) -> Address {
        // Create a new address.
        let address = { self.state.write().unwrap().next_contract_address() };
        // Register new contract under the new address.
        {
            let contract_namespace = self.state.read().unwrap().get_contract_namespace();
            let contract = ContractContainer::new(&contract_namespace, entry_points_caller);
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
        // if let Some(constructor) = constructor {
        //     let (constructor_name, args, _) = constructor;
        //     self.call_constructor(address, &constructor_name, args);
        // }
        address
    }

    pub fn call_contract(&self, address: Address, call_def: CallDef) -> Bytes {
        self.prepare_call(address, &call_def);
        // Call contract from register.
        if call_def.amount > U512::zero() {
            let status = self.checked_transfer_tokens(&self.caller(), &address, &call_def.amount);
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

    // fn call_constructor(&self, address: Address, entrypoint: &str, args: &RuntimeArgs) -> Vec<u8> {
    //     self.prepare_call(address, entrypoint, None);
    //     // Call contract from register.
    //     let register = self.contract_register.read().unwrap();
    //     let result = register.call_constructor(&address, String::from(entrypoint), args);
    //     self.handle_call_result(result)
    // }

    fn prepare_call(&self, address: Address, call_def: &CallDef) {
        let mut state = self.state.write().unwrap();
        // If only one address on the call_stack, record snapshot.
        if state.is_in_caller_context() {
            state.take_snapshot();
            state.clear_error();
        }
        // Put the address on stack.

        let element = CallstackElement::Entrypoint(Entrypoint::new(address, call_def.clone()));
        state.push_callstack_element(element);
    }

    fn handle_call_result(&self, result: Result<Bytes, OdraError>) -> Bytes {
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

    pub fn revert(&self, error: OdraError) -> ! {
        let mut revert_msg = String::from("");
        if let CallstackElement::Entrypoint(ep) = self.callstack_tip() {
            revert_msg = format!("{:?}::{}", ep.address, ep.call_def.entry_point);
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

    pub fn error(&self) -> Option<OdraError> {
        self.state.read().unwrap().error()
    }

    pub fn get_backend_name(&self) -> String {
        self.state.read().unwrap().get_backend_name()
    }

    /// Returns the callee, i.e. the currently executing contract.
    pub fn self_address(&self) -> Address {
        self.state.read().unwrap().callee()
    }

    pub fn caller(&self) -> Address {
        self.state.read().unwrap().caller()
    }

    pub fn callstack_tip(&self) -> CallstackElement {
        self.state.read().unwrap().callstack_tip().clone()
    }

    pub fn get_named_arg(&self, name: &str) -> Vec<u8> {
        match self.state.read().unwrap().callstack_tip() {
            CallstackElement::Account(_) => todo!(),
            CallstackElement::Entrypoint(ep) => {
                ep.call_def.args.get(name).unwrap().inner_bytes().to_vec()
            }
        }
    }

    pub fn set_caller(&self, caller: Address) {
        self.state.write().unwrap().set_caller(caller);
    }

    pub fn set_var(&self, key: &[u8], value: Bytes) {
        self.state.write().unwrap().set_var(key, value);
    }

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

    pub fn set_dict_value(&self, dict: &[u8], key: &[u8], value: Bytes) {
        self.state.write().unwrap().set_dict_value(dict, key, value);
    }

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

    pub fn emit_event(&self, event_data: &Bytes) {
        self.state.write().unwrap().emit_event(event_data);
    }

    pub fn get_event(&self, address: &Address, index: i32) -> Result<Bytes, EventError> {
        self.state.read().unwrap().get_event(address, index)
    }

    pub fn get_events_count(&self, address: &Address) -> u32 {
        self.state.read().unwrap().get_events_count(address)
    }

    pub fn attach_value(&self, amount: U512) {
        self.state.write().unwrap().attach_value(amount);
    }

    pub fn get_block_time(&self) -> u64 {
        self.state.read().unwrap().block_time()
    }

    pub fn advance_block_time_by(&self, milliseconds: u64) {
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

    pub fn balance_of(&self, address: &Address) -> U512 {
        self.state.read().unwrap().balance_of(address)
    }

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
        let address = self.self_address();
        self.state.read().unwrap().balance_of(&address)
    }

    pub fn public_key(&self, address: &Address) -> PublicKey {
        self.state.read().unwrap().public_key(address)
    }

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
}

#[cfg(test)]
mod tests {
    use odra_core::serialize;
    use std::collections::BTreeMap;

    use crate::vm::contract_container::{EntrypointArgs, EntrypointCall};
    use odra_core::call_def::CallDef;
    use odra_core::casper_types::bytesrepr::FromBytes;
    use odra_core::casper_types::{RuntimeArgs, U512};
    use odra_core::OdraAddress;
    use odra_core::{Address, EventData};
    use odra_core::{ExecutionError, OdraError, VmError};

    use super::OdraVm;

    #[test]
    fn contracts_have_different_addresses() {
        // given a new instance
        let instance = OdraVm::default();
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
        let instance = OdraVm::default();
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
        let instance = OdraVm::default();

        let (contract_address, entrypoint, call_result) = setup_contract(&instance);

        // when call an existing entrypoint
        let result = instance.call_contract(
            contract_address,
            CallDef::new(entrypoint, RuntimeArgs::new())
        );

        // then returns the expected value
        assert_eq!(result, serialize(&call_result));
    }

    #[test]
    fn test_call_non_existing_contract() {
        // given an empty vm
        let instance = OdraVm::default();

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
        let instance = OdraVm::default();
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
        let instance = OdraVm::default();

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
        let instance = OdraVm::default();

        // when revert
        instance.revert(ExecutionError::new(1, "err").into());

        // then an error is set
        assert_eq!(instance.error(), Some(ExecutionError::new(1, "err").into()));
    }

    #[test]
    fn test_read_write_value() {
        // given an empty instance
        let instance = OdraVm::default();

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
        let instance = OdraVm::default();

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
        let instance = OdraVm::default();

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
        let instance = OdraVm::default();

        // when push a contract into the stack
        let contract_address = Address::contract_from_u32(100);
        push_address(&instance, &contract_address);

        // then the contract address in the callee
        assert_eq!(instance.self_address(), contract_address);
    }

    #[test]
    fn test_call_contract_with_amount() {
        // given an instance with a registered contract having one entrypoint
        let instance = OdraVm::default();
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
        let instance = OdraVm::default();
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

    fn push_address(vm: &OdraVm, address: &Address) {
        let element = CallstackElement::Account(*address);
        vm.state.write().unwrap().push_callstack_element(element);
    }

    fn setup_contract(instance: &OdraVm) -> (Address, String, u32) {
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
