//! Livenet contract environment.
use crate::livenet_host::read_failed;
use crate::panic_hook;
use blake2::digest::VariableOutput;
use blake2::Blake2bVar;
use odra_casper_rpc_client::casper_client::CasperClient;
use odra_casper_rpc_client::log;
use odra_core::callstack::{Callstack, CallstackElement};
use odra_core::casper_types::{bytesrepr::Bytes, crypto, CLValue, PublicKey, Signature, U512};
use odra_core::prelude::*;
use odra_core::validator::ValidatorInfo;
use odra_core::{CallDef, ContractContext, ContractEnv, ContractRegister, VmError};
use std::io::Write;
use std::rc::Weak;
use std::sync::RwLock;

/// Livenet contract environment struct.
pub struct LivenetContractEnv {
    casper_client: Rc<RefCell<CasperClient>>,
    callstack: Rc<RefCell<Callstack>>,
    contract_register: Rc<RwLock<ContractRegister>>,
    /// The environment wrapping this context, handed to nested calls. Weak, so that the host
    /// stays the only owner of the environment.
    contract_env: RefCell<Weak<ContractEnv>>,
    /// The error of the last revert, read by the host after the panic unwinds.
    error: Rc<RefCell<Option<OdraError>>>
}

impl ContractContext for LivenetContractEnv {
    fn debug(&self, message: &str) {
        println!("{message}");
    }

    fn get_value(&self, key: &[u8]) -> Option<Bytes> {
        let callstack = self.callstack.borrow();
        let client = self.casper_client.borrow();
        let address = callstack.current().address();
        client
            .get_value(address, key)
            .unwrap_or_else(|e| read_failed("state value", address, e))
    }

    fn set_value(&self, _key: &[u8], _value: Bytes) {
        panic!("Cannot set value in LivenetEnv without a deploy")
    }

    fn get_named_value(&self, name: &str) -> Option<Bytes> {
        let client = self.casper_client.borrow();
        let callstack = self.callstack.borrow();
        let address = callstack.current().address();
        client
            .get_named_value(address, name)
            .unwrap_or_else(|e| read_failed(name, address, e))
    }

    fn set_named_value(&self, _name: &str, _value: CLValue) {
        panic!("Cannot set named value in LivenetEnv without a deploy")
    }

    fn get_dictionary_value(&self, dictionary_name: &str, key: &[u8]) -> Option<Bytes> {
        let callstack = self.callstack.borrow();
        let client = self.casper_client.borrow();
        let address = callstack.current().address();
        client
            .get_dictionary_value(address, dictionary_name, key)
            .unwrap_or_else(|e| read_failed(dictionary_name, address, e))
    }

    fn set_dictionary_value(&self, _dictionary_name: &str, _key: &[u8], _value: CLValue) {
        panic!("Cannot set dictionary value in LivenetEnv without a deploy")
    }

    fn remove_dictionary(&self, _dictionary_name: &str) {
        panic!("Cannot remove dictionary value in LivenetEnv without a deploy")
    }

    fn init_dictionary(&self, _dictionary_name: &str) {
        panic!("Cannot initialize dictionary in LivenetEnv without a deploy")
    }

    fn caller(&self) -> Address {
        *self.callstack.borrow().previous().address()
    }

    fn call_stack(&self) -> Vec<Address> {
        self.callstack.borrow().addresses()
    }

    fn self_address(&self) -> Address {
        *self.callstack.borrow().current().address()
    }

    fn call_contract(&self, address: Address, call_def: CallDef) -> Bytes {
        if call_def.is_mut() {
            panic!("Cannot cross call mutable entrypoint from non-mutable entrypoint")
        }
        let contract_env = self
            .contract_env
            .borrow()
            .upgrade()
            .expect("the host owns the livenet contract env");
        call_locally(
            &self.callstack,
            &self.contract_register,
            (*contract_env).clone(),
            &address,
            call_def
        )
        .unwrap_or_else(|e| self.revert(e))
    }

    fn get_block_time(&self) -> u64 {
        let client = self.casper_client.borrow();
        client.get_block_time().unwrap()
    }

    fn attached_value(&self) -> U512 {
        self.callstack.borrow().attached_value()
    }

    fn self_balance(&self) -> U512 {
        let client = self.casper_client.borrow();
        let callstack = self.callstack.borrow();
        client
            .get_balance(callstack.current().address())
            .unwrap_or_else(|e| panic!("Failed to get balance: {}", e.error_message()))
    }

    fn emit_event(&self, _event: &Bytes) {
        panic!("Cannot emit event in LivenetEnv")
    }

    fn emit_native_event(&self, _event: &Bytes) {
        panic!("Cannot emit native event in LivenetEnv")
    }

    fn transfer_tokens(&self, _to: &Address, _amount: &U512) {
        panic!("Cannot transfer tokens in LivenetEnv")
    }

    fn revert(&self, error: OdraError) -> ! {
        let mut revert_msg = String::from("");
        if let CallstackElement::ContractCall {
            address, call_def, ..
        } = self.callstack.borrow().current()
        {
            revert_msg = format!("{:?}::{}", address, call_def.entry_point());
        }
        log::error(format!("Revert: {:?} - {}", error, revert_msg));
        *self.error.borrow_mut() = Some(error.clone());
        panic_hook::mark_revert();
        panic!("Revert: {:?} - {}", error, revert_msg);
    }

    fn get_named_arg_bytes(&self, name: &str) -> OdraResult<Bytes> {
        self.get_opt_named_arg_bytes(name)
            .ok_or(OdraError::ExecutionError(ExecutionError::MissingArg))
    }

    fn get_opt_named_arg_bytes(&self, name: &str) -> Option<Bytes> {
        match self.callstack.borrow().current() {
            // An account frame has no entry point call, so no arguments; as on CasperVM.
            CallstackElement::Account(_) => None,
            CallstackElement::ContractCall { call_def, .. } => call_def
                .args()
                .get(name)
                .map(|cl_value| cl_value.inner_bytes().to_vec())
                .map(Bytes::from)
        }
    }

    fn handle_attached_value(&self) {
        // no-op
    }

    fn clear_attached_value(&self) {
        // no-op
    }

    fn hash(&self, bytes: &[u8]) -> [u8; 32] {
        let mut result = [0u8; 32];
        let mut hasher = <Blake2bVar as VariableOutput>::new(32).expect("should create hasher");
        let _ = hasher.write(bytes);
        hasher
            .finalize_variable(&mut result)
            .expect("should copy hash to the result array");
        result
    }

    fn delegate(&self, _validator: PublicKey, _amount: U512) {
        panic!("delegate is not supported for LivenetContractEnv")
    }

    fn undelegate(&self, _validator: PublicKey, _amount: U512) {
        panic!("undelegate is not supported for LivenetContractEnv")
    }

    fn delegated_amount(&self, _validator: PublicKey) -> U512 {
        let address = match self.callstack.borrow().current() {
            CallstackElement::Account(acc) => *acc,
            CallstackElement::ContractCall { address, .. } => *address
        };
        let client = self.casper_client.borrow();
        client.delegated_amount(address, _validator)
    }

    fn get_validator_info(&self, validator: PublicKey) -> Option<ValidatorInfo> {
        self.casper_client
            .borrow()
            .get_validator_info(validator)
            .map(|bid| ValidatorInfo::new(bid.staked_amount(), bid.minimum_delegation_amount()))
    }

    fn pseudorandom_bytes(&self) -> [u8; 32] {
        panic!(
            "pseudorandom_bytes is not supported for LivenetContractEnv, it should be run\
        in the context of a deploy to get consistent results"
        )
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

impl LivenetContractEnv {
    /// Creates a new LivenetContractEnv.
    pub fn new(
        casper_client: Rc<RefCell<CasperClient>>,
        callstack: Rc<RefCell<Callstack>>,
        contract_register: Rc<RwLock<ContractRegister>>,
        error: Rc<RefCell<Option<OdraError>>>
    ) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            casper_client,
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
}

/// Executes a non-mutable entry point on this machine in `contract_env`, reading the state
/// from the node.
///
/// The contract runs one frame deeper on the call stack, above its caller. The frame is popped when the call
/// returns and when a revert unwinds it.
pub(crate) fn call_locally(
    callstack: &Rc<RefCell<Callstack>>,
    contract_register: &RwLock<ContractRegister>,
    contract_env: ContractEnv,
    address: &Address,
    call_def: CallDef
) -> OdraResult<Bytes> {
    let contract = contract_register
        .read()
        .expect("Couldn't read contract register.")
        .get(address)
        .cloned();
    let Some(contract) = contract else {
        log::error(unregistered_contract_message(address));
        return Err(OdraError::VmError(VmError::InvalidContractAddress));
    };
    let _frame = CallFrame::push(
        callstack,
        CallstackElement::new_contract_call(
            String::from(contract.name()),
            *address,
            call_def.clone()
        )
    );
    contract.call(contract_env, call_def)
}

/// Explains why a non-mutable call to `address` cannot run: livenet runs it on this machine and
/// has no code for the contract.
pub(crate) fn unregistered_contract_message(address: &Address) -> String {
    format!(
        "No contract code registered for {}: livenet runs non-mutable calls (getters) on this \
         machine and needs the code of the called contract. Register it with \
         `Contract::load(&env, address)` (or `ContractHostRef::new(address, env)`), also when \
         it is called only by the getter of another contract. A contract known only by its \
         interface (`#[odra::external_contract]`) can be called only with transactions \
         (mutable calls).",
        address.to_formatted_string()
    )
}

/// A frame on the call stack, popped when dropped.
pub(crate) struct CallFrame(Rc<RefCell<Callstack>>);

impl CallFrame {
    /// Pushes `element` on the call stack.
    pub(crate) fn push(callstack: &Rc<RefCell<Callstack>>, element: CallstackElement) -> Self {
        callstack.borrow_mut().push(element);
        Self(callstack.clone())
    }
}

impl Drop for CallFrame {
    fn drop(&mut self) {
        self.0.borrow_mut().pop();
    }
}
