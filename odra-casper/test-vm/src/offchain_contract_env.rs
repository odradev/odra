//! A read-only contract environment over the Casper VM's global state.
//!
//! It runs `#[odra(offchain)]` functions: the contract's Rust code executes on the host and every
//! storage read is answered from the VM, the way the livenet backend executes getters offline.
use crate::CasperVm;
use odra_core::callstack::{Callstack, CallstackElement};
use odra_core::casper_types::{
    bytesrepr::Bytes, crypto, CLValue, Digest, PublicKey, Signature, U512
};
use odra_core::entry_point_callback::{EntryPoint, EntryPointsCaller};
use odra_core::prelude::*;
use odra_core::validator::ValidatorInfo;
use odra_core::{CallDef, ContractContext, ContractEnv, VmError};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::{Rc, Weak};

/// What the host keeps about a deployed contract to run its offchain functions.
///
/// Deliberately not the `EntryPointsCaller` itself: that one holds a `HostEnv`, and a host that
/// keeps it would own itself and never drop its VM.
pub(crate) struct OffchainContract {
    name: String,
    entry_points: Vec<EntryPoint>,
    call: fn(ContractEnv, CallDef) -> OdraResult<Bytes>
}

impl OffchainContract {
    pub(crate) fn new(name: &str, entry_points_caller: &EntryPointsCaller) -> Self {
        Self {
            name: name.to_string(),
            entry_points: entry_points_caller.entry_points().to_vec(),
            call: entry_points_caller.callback()
        }
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    /// Whether `entry_point` is an `#[odra(offchain)]` function of this contract.
    pub(crate) fn is_offchain(&self, entry_point: &str) -> bool {
        self.entry_points
            .iter()
            .any(|ep| ep.name == entry_point && ep.is_offchain)
    }

    /// The dispatch function, to be called with a fresh contract environment.
    pub(crate) fn callback(&self) -> fn(ContractEnv, CallDef) -> OdraResult<Bytes> {
        self.call
    }
}

/// The offchain contracts of a host, by address.
pub(crate) type OffchainRegister = Rc<RefCell<BTreeMap<Address, OffchainContract>>>;

/// The error an offchain dispatch reports for an unknown contract.
pub(crate) fn no_such_method(entry_point: &str) -> OdraError {
    OdraError::VmError(VmError::NoSuchMethod(entry_point.to_string()))
}

/// Contract environment for offchain functions on the Casper VM.
pub struct CasperOffchainContractEnv {
    vm: Rc<RefCell<CasperVm>>,
    callstack: Rc<RefCell<Callstack>>,
    contract_register: OffchainRegister,
    /// The environment wrapping this context, handed to nested offchain calls. Weak, so that the
    /// host stays the only owner of the environment.
    contract_env: RefCell<Weak<ContractEnv>>,
    /// The error of the last revert, read by the host after the panic unwinds.
    error: Rc<RefCell<Option<OdraError>>>
}

impl CasperOffchainContractEnv {
    /// Creates a new environment sharing the VM, the callstack and the register with the host.
    pub(crate) fn new(
        vm: Rc<RefCell<CasperVm>>,
        callstack: Rc<RefCell<Callstack>>,
        contract_register: OffchainRegister,
        error: Rc<RefCell<Option<OdraError>>>
    ) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            vm,
            callstack,
            contract_register,
            contract_env: RefCell::new(Weak::new()),
            error
        }))
    }

    /// Tells the context which environment wraps it.
    pub(crate) fn set_contract_env(&self, contract_env: &Rc<ContractEnv>) {
        *self.contract_env.borrow_mut() = Rc::downgrade(contract_env);
    }

    fn fresh_contract_env(&self) -> ContractEnv {
        let env = self
            .contract_env
            .borrow()
            .upgrade()
            .expect("the host owns the offchain contract env");
        (*env).clone()
    }

    fn current_address(&self) -> Address {
        *self.callstack.borrow().current().address()
    }

    fn cannot_write(what: &str) -> ! {
        panic!("An offchain function runs on the host and cannot {what}; make it an entry point")
    }
}

impl ContractContext for CasperOffchainContractEnv {
    fn get_value(&self, key: &[u8]) -> Option<Bytes> {
        let address = self.current_address();
        self.vm.borrow().get_storage_value(&address, key)
    }

    fn set_value(&self, _key: &[u8], _value: Bytes) {
        Self::cannot_write("write the state")
    }

    fn get_named_value(&self, name: &str) -> Option<Bytes> {
        let address = self.current_address();
        self.vm.borrow().get_named_value(&address, name)
    }

    fn set_named_value(&self, _name: &str, _value: CLValue) {
        Self::cannot_write("write a named key")
    }

    fn get_dictionary_value(&self, dictionary_name: &str, key: &[u8]) -> Option<Bytes> {
        let address = self.current_address();
        self.vm
            .borrow()
            .get_dictionary_value(&address, dictionary_name, key)
    }

    fn set_dictionary_value(&self, _dictionary_name: &str, _key: &[u8], _value: CLValue) {
        Self::cannot_write("write a dictionary")
    }

    fn remove_dictionary(&self, _dictionary_name: &str) {
        Self::cannot_write("remove a dictionary")
    }

    fn init_dictionary(&self, _dictionary_name: &str) {
        Self::cannot_write("create a dictionary")
    }

    fn caller(&self) -> Address {
        *self.callstack.borrow().first().address()
    }

    fn call_stack(&self) -> Vec<Address> {
        self.callstack.borrow().addresses()
    }

    fn self_address(&self) -> Address {
        self.current_address()
    }

    fn call_contract(&self, address: Address, call_def: CallDef) -> Bytes {
        if call_def.is_mut() {
            Self::cannot_write("call a mutable entry point of another contract")
        }
        let offchain = self
            .contract_register
            .borrow()
            .get(&address)
            .filter(|c| c.is_offchain(call_def.entry_point()))
            .map(|c| (c.name().to_string(), c.callback()));
        if let Some((contract_name, call)) = offchain {
            // Another offchain function: run it here, one frame deeper.
            self.callstack
                .borrow_mut()
                .push(CallstackElement::new_contract_call(
                    contract_name,
                    address,
                    call_def.clone()
                ));
            let result = call(self.fresh_contract_env(), call_def);
            self.callstack.borrow_mut().pop();
            return result.unwrap_or_else(|e| self.revert(e));
        }
        // A real entry point: the VM executes it and the proxy hands back the return value.
        self.vm.borrow_mut().call_contract(&address, call_def, true)
    }

    fn get_block_time(&self) -> u64 {
        self.vm.borrow().block_time()
    }

    fn attached_value(&self) -> U512 {
        self.callstack.borrow().attached_value()
    }

    fn self_balance(&self) -> U512 {
        let address = self.current_address();
        self.vm.borrow().balance_of(&address)
    }

    fn emit_event(&self, _event: &Bytes) {
        Self::cannot_write("emit an event")
    }

    fn emit_native_event(&self, _event: &Bytes) {
        Self::cannot_write("emit a native event")
    }

    fn debug(&self, message: &str) {
        println!("{message}");
    }

    fn transfer_tokens(&self, _to: &Address, _amount: &U512) {
        Self::cannot_write("transfer tokens")
    }

    fn revert(&self, error: OdraError) -> ! {
        let mut revert_msg = String::from("");
        if let CallstackElement::ContractCall {
            address, call_def, ..
        } = self.callstack.borrow().current()
        {
            revert_msg = format!("{:?}::{}", address, call_def.entry_point());
        }
        *self.error.borrow_mut() = Some(error.clone());
        panic!("Revert: {:?} - {}", error, revert_msg);
    }

    fn get_named_arg_bytes(&self, name: &str) -> OdraResult<Bytes> {
        self.get_opt_named_arg_bytes(name)
            .ok_or(OdraError::ExecutionError(ExecutionError::MissingArg))
    }

    fn get_opt_named_arg_bytes(&self, name: &str) -> Option<Bytes> {
        match self.callstack.borrow().current() {
            CallstackElement::Account(_) => None,
            CallstackElement::ContractCall { call_def, .. } => call_def
                .args()
                .get(name)
                .map(|cl_value| cl_value.inner_bytes().to_vec())
                .map(Bytes::from)
        }
    }

    fn handle_attached_value(&self) {
        // no-op: nothing is attached to an offchain call
    }

    fn clear_attached_value(&self) {
        // no-op
    }

    fn hash(&self, bytes: &[u8]) -> [u8; 32] {
        // The same blake2b-256 the wasm host function uses, so storage keys match.
        Digest::hash(bytes).value()
    }

    fn delegate(&self, _validator: PublicKey, _amount: U512) {
        Self::cannot_write("delegate")
    }

    fn undelegate(&self, _validator: PublicKey, _amount: U512) {
        Self::cannot_write("undelegate")
    }

    fn delegated_amount(&self, validator: PublicKey) -> U512 {
        let address = self.current_address();
        self.vm.borrow_mut().delegated_amount(address, validator)
    }

    fn get_validator_info(&self, _validator: PublicKey) -> Option<ValidatorInfo> {
        None
    }

    fn pseudorandom_bytes(&self) -> [u8; 32] {
        panic!("pseudorandom_bytes is not available to an offchain function: there is no transaction to seed it")
    }

    fn verify_signature(
        &self,
        message: &[u8],
        signature: &Signature,
        public_key: &PublicKey
    ) -> bool {
        crypto::verify(message, signature, public_key).is_ok()
    }
}
