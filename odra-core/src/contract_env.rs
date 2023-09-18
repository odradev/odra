use casper_types::bytesrepr::{FromBytes, ToBytes};
use odra::prelude::Rc;
use odra::prelude::RefCell;
use odra::types::{Address, Balance, EventData};
use odra_types::{ExecutionError, OdraError};
use odra_types::VmError::Deserialization;
use crate::call_def::CallDef;
pub use crate::ContractContext;
use crate::module::ModuleCaller;
use crate::odra_result::{OdraResult};
use crate::path_stack::PathStack;

pub struct ContractEnv {
    path_stack: PathStack,
    backend: Rc<RefCell<dyn ContractContext>>,
}

impl ContractEnv {
    pub fn new(backend: Rc<RefCell<dyn ContractContext>>) -> ContractEnv {
        ContractEnv {
            path_stack: PathStack::new(),
            backend,
        }
    }

    // pub fn clone_with<T: OdraType>(&self, key: T) -> Self {
    //     let mut path_stack = self.path_stack.clone();
    //     path_stack.push(key);
    //     Env { path_stack, backend: self.backend.clone() }
    // }

    pub fn clone_empty(&self) -> Self {
        ContractEnv {
            path_stack: PathStack::new(),
            backend: self.backend.clone(),
        }
    }

    // pub fn module<M: Module, T: OdraType>(&self, path: T) -> M {
    //     M::new(self.clone_with(path))
    // }

    pub fn get_or_none<T: ToBytes, V: FromBytes>(&self, key: T) -> OdraResult<Option<V>> {
        let key = self.path_stack.get_key(Some(key));
        let backend = self.backend.borrow();
        match backend.get(key) {
            Some(bytes) => match V::from_bytes(&bytes) {
                Ok((value, _bytes)) => Ok(Some(value)),
                Err(_err) => Err(OdraError::VmError(Deserialization)),
            },
            None => Ok(None),
        }
    }

    pub fn get<T: ToBytes, V: FromBytes>(&self, key: T) -> OdraResult<V> {
        if let Some(result) = self.get_or_none(&key)? {
            return Ok(result);
        }

        // TODO: resolve the key in a safer way
        Err(OdraError::ExecutionError(ExecutionError::key_not_found()))
    }

    pub fn set<T: ToBytes, V: ToBytes>(&mut self, key: T, value: V) {
        let key = self.path_stack.get_key(Some(key));
        let backend = self.backend.borrow();
        backend.set(key, value.to_bytes().unwrap());
    }

    pub fn caller(&self) -> Address {
        let backend = self.backend.borrow();
        backend.get_caller()
    }

    pub fn call_contract<T: FromBytes>(&self, address: Address, call: CallDef) -> OdraResult<T> {
        let backend = self.backend.borrow();
        let bytes = backend.call_contract(self.clone_empty(), address, call)?;
        Ok(T::from_bytes(&bytes).unwrap().0)
    }

    pub fn new_contract(&mut self, caller: ModuleCaller) -> Address {
        let backend = self.backend.borrow();
        backend.new_contract(caller)
    }

    pub fn self_address(&self) -> Address {
        let backend = self.backend.borrow();
        backend.callee()
    }

    pub fn get_block_time(&self) -> u64 {
        let backend = self.backend.borrow();
        backend.get_block_time()
    }

    pub fn attached_value(&self) -> Option<Balance> {
        let backend = self.backend.borrow();
        backend.attached_value()
    }

    pub fn balance_of(&self, address: &Address) -> Balance {
        let backend = self.backend.borrow();
        backend.balance_of(address)
    }
}