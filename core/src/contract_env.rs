use casper_event_standard::EventInstance;
use casper_types::CLValueError;

use crate::args::EntrypointArgument;
use crate::call_def::CallDef;
use crate::casper_types::bytesrepr::{deserialize_from_slice, Bytes, FromBytes, ToBytes};
use crate::casper_types::crypto::PublicKey;
use crate::casper_types::{CLTyped, CLValue, BLAKE2B_DIGEST_LENGTH, U512};
use crate::module::Revertible;
pub use crate::ContractContext;
use crate::ExecutionError::Formatting;
use crate::VmError::{Serialization, TypeMismatch};
use crate::{consts, prelude::*, ExecutionError};
use crate::{utils, UnwrapOrRevert};
use crate::{Address, OdraError};

const INDEX_SIZE: usize = 4;
const KEY_LEN: usize = 64;
pub(crate) type StorageKey = [u8; KEY_LEN];

/// Trait that needs to be implemented by all contract refs.
pub trait ContractRef {
    /// Creates a new instance of the Contract Ref.
    fn new(env: Rc<ContractEnv>, address: Address) -> Self;
    /// Returns the address of the contract.
    fn address(&self) -> &Address;
}

/// Represents the environment accessible in the contract context.
///
/// The `ContractEnv` struct provides methods for interacting with the contract environment,
/// such as accessing storage, calling other contracts, and handling various contract-related operations.
///
/// The `ContractEnv` is available for the user to use in the module code.
#[derive(Clone)]
pub struct ContractEnv {
    index: u32,
    mapping_data: Vec<u8>,
    backend: Rc<RefCell<dyn ContractContext>>
}

impl Revertible for ContractEnv {
    fn revert<E: Into<OdraError>>(&self, e: E) -> ! {
        self.revert(e)
    }
}

impl ContractEnv {
    /// Creates a new ContractEnv instance.
    pub const fn new(index: u32, backend: Rc<RefCell<dyn ContractContext>>) -> Self {
        Self {
            index,
            mapping_data: Vec::new(),
            backend
        }
    }

    /// Returns the current storage key for the contract environment.
    pub(crate) fn current_key(&self) -> StorageKey {
        let mut result = [0u8; KEY_LEN];
        let mut key = Vec::with_capacity(INDEX_SIZE + self.mapping_data.len());
        key.extend_from_slice(self.index.to_be_bytes().as_ref());
        key.extend_from_slice(&self.mapping_data);
        let hashed_key = self.backend.borrow().hash(key.as_slice());
        utils::hex_to_slice(&hashed_key, &mut result);
        result
    }

    /// Adds the given data to the mapping data of the contract environment.
    pub(crate) fn add_to_mapping_data(&mut self, data: &[u8]) {
        self.mapping_data.extend_from_slice(data);
    }

    /// Returns a child contract environment with the specified index.
    pub(crate) fn child(&self, index: u8) -> Self {
        Self {
            index: (self.index << 4) + index as u32,
            mapping_data: self.mapping_data.clone(),
            backend: self.backend.clone()
        }
    }

    /// Retrieves the value associated with the given key from the contract storage.
    ///
    /// # Returns
    ///
    /// The value associated with the key, if it exists.
    pub fn get_value<T: FromBytes>(&self, key: &[u8]) -> Option<T> {
        self.backend
            .borrow()
            .get_value(key)
            .map(|bytes| deserialize_from_slice(bytes).unwrap_or_revert(self))
    }

    /// Sets the value associated with the given key in the contract storage.
    pub fn set_value<T: ToBytes + CLTyped>(&self, key: &[u8], value: T) {
        let result = value.to_bytes().map_err(ExecutionError::from);
        let bytes = result.unwrap_or_revert(self);
        self.backend.borrow().set_value(key, bytes.into());
    }

    /// Retrieves the value associated with the given named key from the contract storage.
    pub fn get_named_value<T: FromBytes + CLTyped, U: AsRef<str>>(&self, name: U) -> Option<T> {
        let key = name.as_ref();
        let bytes = self.backend.borrow().get_named_value(key);
        bytes.map(|b| deserialize_from_slice(b).unwrap_or_revert(self))
    }

    /// Sets the value associated with the given named key in the contract storage.
    pub fn set_named_value<T: CLTyped + ToBytes, U: AsRef<str>>(&self, name: U, value: T) {
        let key = name.as_ref();
        // todo: map errors to correct Odra errors
        let cl_value = CLValue::from_t(value)
            .map_err(|e| match e {
                CLValueError::Serialization(_) => OdraError::VmError(Serialization),
                CLValueError::Type(e) => OdraError::VmError(TypeMismatch {
                    found: e.found,
                    expected: e.expected
                })
            })
            .unwrap_or_revert(self);
        self.backend.borrow().set_named_value(key, cl_value);
    }

    /// Retrieves the value associated with the given named key from the named dictionary in the contract storage.
    pub fn get_dictionary_value<T: FromBytes + CLTyped, U: AsRef<str>>(
        &self,
        dictionary_name: U,
        key: &[u8]
    ) -> Option<T> {
        let dictionary_name = dictionary_name.as_ref();
        let bytes = self
            .backend
            .borrow()
            .get_dictionary_value(dictionary_name, key);
        bytes.map(|b| {
            deserialize_from_slice(b)
                .map_err(|_| Formatting)
                .unwrap_or_revert(self)
        })
    }

    /// Sets the value associated with the given named key in the named dictionary in the contract storage.
    pub fn set_dictionary_value<T: CLTyped + ToBytes, U: AsRef<str>>(
        &self,
        dictionary_name: U,
        key: &[u8],
        value: T
    ) {
        let dictionary_name = dictionary_name.as_ref();
        let cl_value = CLValue::from_t(value)
            .map_err(|_| Formatting)
            .unwrap_or_revert(self);
        self.backend
            .borrow()
            .set_dictionary_value(dictionary_name, key, cl_value);
    }

    /// Removes the dictionary from the contract storage.
    pub fn remove_dictionary<U: AsRef<str>>(&self, dictionary_name: U) {
        let dictionary_name = dictionary_name.as_ref();
        self.backend.borrow().remove_dictionary(dictionary_name);
    }

    /// Returns the address of the caller of the contract.
    pub fn caller(&self) -> Address {
        let backend = self.backend.borrow();
        backend.caller()
    }

    /// Calls another contract with the specified address and call definition.
    ///
    /// # Returns
    ///
    /// The result of the contract call. If any error occurs during the call, the contract will revert.
    pub fn call_contract<T: FromBytes>(&self, address: Address, call: CallDef) -> T {
        let backend = self.backend.borrow();
        let bytes = backend.call_contract(address, call);
        deserialize_from_slice(bytes).unwrap_or_revert(self)
    }

    /// Returns the address of the current contract.
    pub fn self_address(&self) -> Address {
        let backend = self.backend.borrow();
        backend.self_address()
    }

    /// Transfers tokens to the specified address.
    pub fn transfer_tokens(&self, to: &Address, amount: &U512) {
        let backend = self.backend.borrow();
        backend.transfer_tokens(to, amount)
    }

    /// Returns the current block time as u64 value.
    pub fn get_block_time(&self) -> u64 {
        let backend = self.backend.borrow();
        backend.get_block_time()
    }

    /// Returns the value attached to the contract call.
    pub fn attached_value(&self) -> U512 {
        let backend = self.backend.borrow();
        backend.attached_value()
    }

    /// Returns the CSPR balance of the current contract.
    pub fn self_balance(&self) -> U512 {
        let backend = self.backend.borrow();
        backend.self_balance()
    }

    /// Reverts the contract execution with the specified error.
    pub fn revert<E: Into<OdraError>>(&self, error: E) -> ! {
        let backend = self.backend.borrow();
        backend.revert(error.into())
    }

    /// Emits an event with the specified data.
    pub fn emit_event<T: ToBytes + EventInstance>(&self, event: T) {
        let backend = self.backend.borrow();
        let result = event.to_bytes().map_err(ExecutionError::from);
        let bytes = result.unwrap_or_revert(self);
        backend.emit_event(&bytes.into())
    }

    /// Verifies the signature of a message using the specified signature, public key, and message.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to verify.
    /// * `signature` - The signature to verify.
    /// * `public_key` - The public key to use for verification.
    ///
    /// # Returns
    ///
    /// `true` if the signature is valid, `false` otherwise.
    pub fn verify_signature(
        &self,
        message: &Bytes,
        signature: &Bytes,
        public_key: &PublicKey
    ) -> bool {
        let (signature, _) = casper_types::crypto::Signature::from_bytes(signature.as_slice())
            .unwrap_or_else(|_| self.revert(ExecutionError::CouldNotDeserializeSignature));
        casper_types::crypto::verify(message.as_slice(), &signature, public_key).is_ok()
    }

    /// Hashes the specified value.
    ///
    /// # Returns
    ///
    /// The hash value as a 32-byte array.
    pub fn hash<T: AsRef<[u8]>>(&self, value: T) -> [u8; BLAKE2B_DIGEST_LENGTH] {
        self.backend.borrow().hash(value.as_ref())
    }
}

/// Represents the environment accessible in the contract execution context.
///
/// `ExecutionEnv` provides pre- and post-execution methods for the contract, such as performing non-reentrant checks
/// and handling the attached value.
pub struct ExecutionEnv {
    env: Rc<ContractEnv>
}

impl Revertible for ExecutionEnv {
    fn revert<E: Into<OdraError>>(&self, e: E) -> ! {
        self.env.revert(e)
    }
}

impl ExecutionEnv {
    /// Creates a new ExecutionEnv instance.
    pub fn new(env: Rc<ContractEnv>) -> Self {
        Self { env }
    }

    /// Performs non-reentrant checks before executing a function.
    pub fn non_reentrant_before(&self) {
        // Check if reentrancy guard is set to true
        let status: bool = self
            .env
            .get_value(consts::REENTRANCY_GUARD.as_slice())
            .unwrap_or_default();
        if status {
            // Revert execution with ReentrantCall error
            self.env.revert(ExecutionError::ReentrantCall);
        }
        // Set reentrancy guard to true
        self.env
            .set_value(consts::REENTRANCY_GUARD.as_slice(), true);
    }

    /// Resets the reentrancy guard after executing a function.
    pub fn non_reentrant_after(&self) {
        // Set reentrancy guard to false
        self.env
            .set_value(consts::REENTRANCY_GUARD.as_slice(), false);
    }

    /// Handles the attached value in the execution environment.
    pub fn handle_attached_value(&self) {
        self.env.backend.borrow().handle_attached_value();
    }

    /// Clears the attached value in the execution environment.
    pub fn clear_attached_value(&self) {
        self.env.backend.borrow().clear_attached_value();
    }

    /// Retrieves the value of a named argument from the execution environment.
    ///
    /// # Returns
    ///
    /// The deserialized value of the named argument. If the argument does not exist or deserialization fails,
    /// the contract will revert.
    pub fn get_named_arg<T: FromBytes + EntrypointArgument>(&self, name: &str) -> T {
        if T::is_required() {
            let result = self.env.backend.borrow().get_named_arg_bytes(name);
            match result {
                Ok(bytes) => deserialize_from_slice(bytes).unwrap_or_revert(self),
                Err(err) => self.env.revert(err)
            }
        } else {
            let bytes = self.env.backend.borrow().get_opt_named_arg_bytes(name);
            let result = bytes.map(|bytes| deserialize_from_slice(bytes).unwrap_or_revert(self));
            T::unwrap(result, &self.env)
        }
    }
}
