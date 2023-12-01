use crate::call_def::CallDef;
use crate::key_maker;
pub use crate::ContractContext;
use crate::{prelude::*, ExecutionError};
use crate::{Address, Bytes, CLTyped, FromBytes, OdraError, ToBytes, U512};

pub struct ContractEnv {
    index: u32,
    mapping_data: Vec<u8>,
    backend: Rc<RefCell<dyn ContractContext>>
}

impl ContractEnv {
    pub const fn new(index: u32, backend: Rc<RefCell<dyn ContractContext>>) -> Self {
        Self {
            index,
            mapping_data: Vec::new(),
            backend
        }
    }

    pub fn __duplicate(&self) -> Self {
        Self {
            index: self.index,
            mapping_data: self.mapping_data.clone(),
            backend: self.backend.clone()
        }
    }

    pub(crate) fn current_key(&self) -> Vec<u8> {
        let index_bytes = key_maker::u32_to_hex(self.index);
        let mapping_data_bytes = key_maker::bytes_to_hex(&self.mapping_data);
        let mut key = Vec::new();
        key.extend_from_slice(&index_bytes);
        key.extend_from_slice(&mapping_data_bytes);
        key
    }

    pub(crate) fn add_to_mapping_data(&mut self, data: &[u8]) {
        self.mapping_data.extend_from_slice(data);
    }

    pub(crate) fn child(&self, index: u8) -> Self {
        Self {
            index: (self.index << 4) + index as u32,
            mapping_data: self.mapping_data.clone(),
            backend: self.backend.clone()
        }
    }

    pub fn get_value<T: FromBytes>(&self, key: &[u8]) -> Option<T> {
        self.backend
            .borrow()
            .get_value(key)
            .map(|bytes| T::from_bytes(&bytes).unwrap().0)
    }

    pub fn set_value<T: ToBytes + CLTyped>(&self, key: &[u8], value: T) {
        let bytes = value.to_bytes().unwrap();
        self.backend.borrow().set_value(key, Bytes::from(bytes));
    }

    pub fn caller(&self) -> Address {
        let backend = self.backend.borrow();
        backend.caller()
    }

    pub fn call_contract<T: FromBytes>(&self, address: Address, call: CallDef) -> T {
        let backend = self.backend.borrow();
        let bytes = backend.call_contract(address, call);
        T::from_bytes(&bytes).unwrap().0
    }

    pub fn self_address(&self) -> Address {
        let backend = self.backend.borrow();
        backend.self_address()
    }

    pub fn transfer_tokens(&self, to: &Address, amount: &U512) {
        let backend = self.backend.borrow();
        backend.transfer_tokens(to, amount)
    }

    pub fn get_block_time(&self) -> u64 {
        let backend = self.backend.borrow();
        backend.get_block_time()
    }

    pub fn attached_value(&self) -> U512 {
        let backend = self.backend.borrow();
        backend.attached_value()
    }

    pub fn revert<E: Into<OdraError>>(&self, error: E) -> ! {
        let backend = self.backend.borrow();
        backend.revert(error.into())
    }

    pub fn emit_event<T: ToBytes>(&self, event: T) {
        let backend = self.backend.borrow();
        backend.emit_event(&event.to_bytes().unwrap().into())
    }
}

pub struct ExecutionEnv {
    env: Rc<ContractEnv>
}

impl ExecutionEnv {
    pub fn new(env: Rc<ContractEnv>) -> Self {
        Self { env }
    }

    pub fn non_reentrant_before(&self) {
        let status: bool = self
            .env
            .get_value(crate::consts::REENTRANCY_GUARD.as_slice())
            .unwrap_or_default();
        if status {
            self.env.revert(ExecutionError::ReentrantCall);
        }
        self.env
            .set_value(crate::consts::REENTRANCY_GUARD.as_slice(), true);
    }

    pub fn non_reentrant_after(&self) {
        self.env
            .set_value(crate::consts::REENTRANCY_GUARD.as_slice(), false);
    }

    pub fn handle_attached_value(&self) {
        self.env.backend.borrow().handle_attached_value();
    }

    pub fn clear_attached_value(&self) {
        self.env.backend.borrow().clear_attached_value();
    }

    pub fn get_named_arg<T: FromBytes>(&self, name: &str) -> T {
        let bytes = self.env.backend.borrow().get_named_arg(name);
        T::from_bytes(&bytes).unwrap().0
    }
}
