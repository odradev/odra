use odra_casper_client::casper_client::CasperClient;
use blake2::digest::VariableOutput;
use blake2::{Blake2b, Blake2b512, Blake2bVar, Blake2s256, Digest};
use odra_core::casper_types::BlockTime;
use odra_core::prelude::*;
use odra_core::{casper_types, Address, Bytes, OdraError, ToBytes, U512};
use odra_core::{CallDef, ContractContext};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::io::Write;


pub struct LivenetContractEnv {
    casper_client: Rc<RefCell<CasperClient>>,
    callstack: Rc<RefCell<Vec<Address>>>,
}

impl ContractContext for LivenetContractEnv {
    fn get_value(&self, key: &[u8]) -> Option<Bytes> {
        self.casper_client.borrow().get_value(key)
    }

    fn set_value(&self, key: &[u8], value: Bytes) {
        panic!("Cannot set value in LivenetEnv")
    }

    fn caller(&self) -> Address {
        todo!()
    }

    fn self_address(&self) -> Address {
        todo!()
    }

    fn call_contract(&self, address: Address, call_def: CallDef) -> Bytes {
        todo!()
    }

    fn get_block_time(&self) -> u64 {
        todo!()
    }

    fn attached_value(&self) -> U512 {
        todo!()
    }

    fn emit_event(&self, event: &Bytes) {
        panic!("Cannot emit event in LivenetEnv")
    }

    fn transfer_tokens(&self, to: &Address, amount: &U512) {
        panic!("Cannot transfer tokens in LivenetEnv")
    }

    fn revert(&self, error: OdraError) -> ! {
        dbg!(error);
        todo!()
    }

    fn get_named_arg_bytes(&self, name: &str) -> Bytes {
        todo!()
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
    pub fn new(casper_client: Rc<RefCell<CasperClient>>, callstack: Rc<RefCell<Vec<Address>>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self { casper_client, callstack }))
    }
}