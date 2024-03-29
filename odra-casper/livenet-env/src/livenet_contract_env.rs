//! Livenet contract environment.
use blake2::digest::VariableOutput;
use blake2::Blake2bVar;
use odra_casper_rpc_client::casper_client::CasperClient;
use odra_core::callstack::{Callstack, CallstackElement};
use odra_core::casper_types::bytesrepr::Bytes;
use odra_core::casper_types::U512;
use odra_core::prelude::*;
use odra_core::{Address, OdraError};
use odra_core::{CallDef, ContractContext, ContractRegister};
use std::io::Write;
use std::sync::RwLock;

/// Livenet contract environment struct.
pub struct LivenetContractEnv {
    casper_client: Rc<RefCell<CasperClient>>,
    callstack: Rc<RefCell<Callstack>>,
    contract_register: Rc<RwLock<ContractRegister>>
}

impl ContractContext for LivenetContractEnv {
    fn get_value(&self, key: &[u8]) -> Option<Bytes> {
        self.casper_client
            .borrow()
            .get_value(self.callstack.borrow().current().address(), key)
    }

    fn set_value(&self, _key: &[u8], _value: Bytes) {
        panic!("Cannot set value in LivenetEnv without a deploy")
    }

    fn caller(&self) -> Address {
        *self.callstack.borrow().first().address()
    }

    fn self_address(&self) -> Address {
        *self.callstack.borrow().current().address()
    }

    fn call_contract(&self, address: Address, call_def: CallDef) -> Bytes {
        if call_def.is_mut() {
            panic!("Cannot cross call mutable entrypoint from non-mutable entrypoint")
        }

        self.callstack
            .borrow_mut()
            .push(CallstackElement::new_contract_call(
                address,
                call_def.clone()
            ));
        let result = self
            .contract_register
            .read()
            .unwrap()
            .call(&address, call_def);
        self.callstack.borrow_mut().pop();
        result.unwrap()
    }

    fn get_block_time(&self) -> u64 {
        self.casper_client.borrow().get_block_time()
    }

    fn attached_value(&self) -> U512 {
        self.callstack.borrow().attached_value()
    }

    fn self_balance(&self) -> U512 {
        self.casper_client
            .borrow()
            .get_balance(self.callstack.borrow().current().address())
    }

    fn emit_event(&self, _event: &Bytes) {
        panic!("Cannot emit event in LivenetEnv")
    }

    fn transfer_tokens(&self, _to: &Address, _amount: &U512) {
        panic!("Cannot transfer tokens in LivenetEnv")
    }

    fn revert(&self, error: OdraError) -> ! {
        let mut revert_msg = String::from("");
        if let CallstackElement::ContractCall { address, call_def } =
            self.callstack.borrow().current()
        {
            revert_msg = format!("{:?}::{}", address, call_def.entry_point());
        }

        panic!("Revert: {:?} - {}", error, revert_msg);
    }

    fn get_named_arg_bytes(&self, name: &str) -> Bytes {
        self.get_opt_named_arg_bytes(name).unwrap()
    }

    fn get_opt_named_arg_bytes(&self, name: &str) -> Option<Bytes> {
        match self.callstack.borrow().current() {
            CallstackElement::Account(_) => todo!("get_named_arg_bytes"),
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
}

impl LivenetContractEnv {
    /// Creates a new LivenetContractEnv.
    pub fn new(
        casper_client: Rc<RefCell<CasperClient>>,
        callstack: Rc<RefCell<Callstack>>,
        contract_register: Rc<RwLock<ContractRegister>>
    ) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            casper_client,
            callstack,
            contract_register
        }))
    }
}
