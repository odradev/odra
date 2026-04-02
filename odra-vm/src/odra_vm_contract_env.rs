use crate::vm::OdraVm;
use blake2::digest::VariableOutput;
use blake2::{Blake2b, Blake2b512, Blake2bVar, Blake2s256, Digest};
use odra_core::casper_types::system::auction::ValidatorBid;
use odra_core::casper_types::{
    bytesrepr::{Bytes, ToBytes},
    CLValue, PublicKey, U512
};
use odra_core::consts::RANDOM_BYTES_COUNT;
use odra_core::prelude::*;
use odra_core::validator::ValidatorInfo;
use odra_core::{casper_types, CallDef, ContractContext};
use rand::Rng;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::io::Write;

pub struct OdraVmContractEnv {
    vm: Rc<OdraVm>
}

impl ContractContext for OdraVmContractEnv {
    fn get_value(&self, key: &[u8]) -> Option<Bytes> {
        self.vm.get_var(key)
    }

    fn set_value(&self, key: &[u8], value: Bytes) {
        self.vm.set_var(key, value)
    }

    fn get_named_value(&self, name: &str) -> Option<Bytes> {
        self.vm.get_named_key(name)
    }

    fn set_named_value(&self, name: &str, value: CLValue) {
        self.vm.set_named_key(name, value)
    }

    fn get_dictionary_value(&self, dictionary_name: &str, key: &[u8]) -> Option<Bytes> {
        self.vm.get_dict_value(dictionary_name, key)
    }

    fn set_dictionary_value(&self, dictionary_name: &str, key: &[u8], value: CLValue) {
        self.vm.set_dict_value(dictionary_name, key, value)
    }

    fn remove_dictionary(&self, dictionary_name: &str) {
        self.vm.remove_dictionary(dictionary_name);
    }

    fn init_dictionary(&self, dictionary_name: &str) {
        // no-op, dictionaries are initialized automatically in Odra VM
    }

    fn caller(&self) -> Address {
        self.vm.caller()
    }

    fn self_address(&self) -> Address {
        self.vm.self_address()
    }

    fn call_contract(&self, address: Address, call_def: CallDef) -> Bytes {
        self.vm.call_contract(address, call_def)
    }

    fn get_block_time(&self) -> u64 {
        self.vm.get_block_time()
    }

    fn attached_value(&self) -> U512 {
        self.vm.attached_value()
    }

    fn self_balance(&self) -> U512 {
        self.vm.self_balance()
    }

    fn emit_event(&self, event: &Bytes) {
        self.vm.emit_event(event);
    }

    fn emit_native_event(&self, event: &Bytes) {
        self.vm.emit_native_event(event);
    }

    fn transfer_tokens(&self, to: &Address, amount: &U512) {
        self.vm.transfer_tokens(to, amount)
    }

    fn revert(&self, error: OdraError) -> ! {
        self.vm.revert(error)
    }

    fn get_named_arg_bytes(&self, name: &str) -> OdraResult<Bytes> {
        self.vm.get_named_arg(name).map(Into::into)
    }

    fn get_opt_named_arg_bytes(&self, name: &str) -> Option<Bytes> {
        self.vm.get_named_arg(name).ok().map(Into::into)
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

    fn delegate(&self, validator: PublicKey, amount: U512) {
        let delegator = self.vm.callee();
        self.vm.delegate(validator, delegator, amount);
    }

    fn undelegate(&self, validator: PublicKey, amount: U512) {
        let delegator = self.vm.callee();
        self.vm.undelegate(validator, delegator, amount);
    }

    fn delegated_amount(&self, validator: PublicKey) -> U512 {
        let delegator = self.vm.callee();
        self.vm.delegated_amount(delegator, validator)
    }

    fn get_validator_info(&self, validator: PublicKey) -> Option<ValidatorInfo> {
        self.vm.get_validator_info(validator)
    }

    fn pseudorandom_bytes(&self) -> [u8; RANDOM_BYTES_COUNT] {
        use rand::Rng;
        let mut bytes = [0u8; RANDOM_BYTES_COUNT];
        rand::rng().fill(&mut bytes[..]);
        bytes
    }
}

impl OdraVmContractEnv {
    pub fn new(vm: Rc<OdraVm>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self { vm }))
    }
}
