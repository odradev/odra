use crate::storage::{DictionaryStorage, NamedKeyStorage};
use odra::casper_types::{U256, U512};
use odra::prelude::*;
use odra_modules::erc20::Erc20;

/// Contract designed to benchmark the Odra framework.
#[odra::module]
pub struct Benchmark {
    variable: Var<bool>,
    struct_variable: Var<StructVariable>,
    mapping: Mapping<u32, bool>,
    list: List<u32>,
    submodule: SubModule<Erc20>,
    named_key: SubModule<NamedKeyStorage>,
    dictionary: SubModule<DictionaryStorage>
}

#[odra::module]
impl Benchmark {
    pub fn init(&mut self) {
        self.variable.set(false);
    }

    pub fn set_variable(&mut self, value: bool) {
        self.variable.set(value);
    }

    pub fn get_variable(&self) -> bool {
        self.variable.get_or_default()
    }

    pub fn set_struct_variable(&mut self, value: StructVariable) {
        self.struct_variable.set(value);
    }

    pub fn get_struct_variable(&self) -> StructVariable {
        self.struct_variable.get_or_default()
    }

    pub fn set_mapping(&mut self, key: u32, value: bool) {
        self.mapping.set(&key, value);
    }

    pub fn get_mapping(&self, key: u32) -> bool {
        self.mapping.get_or_default(&key)
    }

    pub fn set_named_key(&mut self, value: String) {
        self.named_key.set(value);
    }

    pub fn get_named_key(&self) -> String {
        self.named_key.get()
    }

    pub fn set_dictionary(&mut self, key: String, value: U256) {
        self.dictionary.set(key, value);
    }

    pub fn get_dictionary(&self, key: String) -> U256 {
        self.dictionary.get_or_default(key)
    }

    pub fn push_list(&mut self, value: u32) {
        self.list.push(value);
    }

    pub fn get_list(&self, index: u32) -> u32 {
        self.list.get_or_default(index)
    }

    pub fn init_submodule(&mut self) {
        self.submodule.init(
            "PLS".to_string(),
            "Plascoin".to_string(),
            18,
            Some(1_000_000_000.into())
        );
    }

    pub fn call_submodule(&self) -> U256 {
        self.submodule.total_supply()
    }

    #[odra(payable)]
    pub fn call_payable(&self) {
        // Do nothing, collect sweet $$$
    }

    pub fn transfer_back(&mut self, amount: U512) {
        self.env().transfer_tokens(&self.env().caller(), &amount);
    }

    pub fn emit_event(&self) {
        self.env().emit_event(SomeEvent {
            message: "Hello, world!".to_string()
        });
    }
}

#[odra::odra_type]
#[derive(Default)]
pub struct StructVariable {
    pub yes_or_no: bool,
    pub number: u32,
    pub title: String
}

#[odra::event]
pub struct SomeEvent {
    pub message: String
}
