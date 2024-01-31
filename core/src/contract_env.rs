use crate::call_def::CallDef;
use crate::casper_types::bytesrepr::{Bytes, FromBytes, ToBytes};
use crate::casper_types::{CLTyped, U512};
pub use crate::ContractContext;
use crate::{consts, prelude::*, ExecutionError};
use crate::{utils, UnwrapOrRevert};
use crate::{Address, OdraError};
use casper_types::crypto::PublicKey;

const INDEX_SIZE: usize = 4;
const KEY_LEN: usize = 64;
pub(crate) type StorageKey = [u8; KEY_LEN];

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
        let hashed_key = self.backend.borrow().hash(&key);
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
            .map(|bytes| deserialize_bytes(bytes, self))
    }

    /// Sets the value associated with the given key in the contract storage.
    pub fn set_value<T: ToBytes + CLTyped>(&self, key: &[u8], value: T) {
        let result = value.to_bytes().map_err(ExecutionError::from);
        let bytes = result.unwrap_or_revert(self);
        self.backend.borrow().set_value(key, bytes.into());
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
        deserialize_bytes(bytes, self)
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

    /// Reverts the contract execution with the specified error.
    pub fn revert<E: Into<OdraError>>(&self, error: E) -> ! {
        let backend = self.backend.borrow();
        backend.revert(error.into())
    }

    /// Emits an event with the specified data.
    pub fn emit_event<T: ToBytes>(&self, event: T) {
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
    pub fn hash<T: ToBytes>(&self, value: T) -> [u8; 32] {
        let bytes = value
            .to_bytes()
            .map_err(ExecutionError::from)
            .unwrap_or_revert(self);
        self.backend.borrow().hash(&bytes)
    }
}

/// Represents the environment accessible in the contract execution context.
///
/// `ExecutionEnv` provides pre and post execution methods for the contract, such as performing non-reentrant checks
/// and handling the attached value.
pub struct ExecutionEnv {
    env: Rc<ContractEnv>
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
    pub fn get_named_arg<T: FromBytes>(&self, name: &str) -> T {
        let bytes = self.env.backend.borrow().get_named_arg_bytes(name);
        deserialize_bytes(bytes, &self.env)
    }
}

fn deserialize_bytes<T: FromBytes>(bytes: Bytes, env: &ContractEnv) -> T {
    match T::from_bytes(&bytes) {
        Ok((value, remainder)) => {
            if remainder.is_empty() {
                value
            } else {
                env.revert(ExecutionError::LeftOverBytes)
            }
        }
        Err(err) => env.revert(ExecutionError::from(err))
    }
}
