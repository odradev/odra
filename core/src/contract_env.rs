use crate::args::EntrypointArgument;
use crate::call_def::CallDef;
use crate::casper_types::bytesrepr::{deserialize_from_slice, Bytes, FromBytes, ToBytes};
use crate::casper_types::crypto::PublicKey;
use crate::casper_types::{CLTyped, CLValue, BLAKE2B_DIGEST_LENGTH, U512};
use crate::module::Revertible;
use crate::validator::ValidatorInfo;
pub use crate::ContractContext;
use crate::VmError::{Serialization, TypeMismatch};
use crate::{consts, prelude::*, utils};
use casper_event_standard::{EventInstance, Schema, Schemas, EVENTS_SCHEMA};
use casper_types::CLValueError;
use rand_chacha::rand_core::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;

const INDEX_SIZE: usize = 4;
const KEY_LEN: usize = 64;
const V2_KEY_TAG: &[u8] = b"odra-v2";
pub(crate) type StorageKey = [u8; KEY_LEN];

#[derive(Clone, Copy)]
pub(crate) enum StorageSlot {
    User(u8),
    Internal(u8)
}

impl StorageSlot {
    const V2_USER_TAG: u8 = 0;
    const V2_INTERNAL_TAG: u8 = 1;
    const LEGACY_MAX_INDEX: u8 = 15;

    pub(crate) const fn user(index: u8) -> Self {
        Self::User(index)
    }

    pub(crate) const fn internal(index: u8) -> Self {
        Self::Internal(index)
    }

    const fn raw_index(self) -> u8 {
        match self {
            Self::User(index) | Self::Internal(index) => index
        }
    }

    const fn is_legacy_compatible(self) -> bool {
        self.raw_index() <= Self::LEGACY_MAX_INDEX
    }

    fn encode_v2(self, output: &mut Vec<u8>) {
        match self {
            Self::User(index) => {
                output.push(Self::V2_USER_TAG);
                output.push(index);
            }
            Self::Internal(index) => {
                output.push(Self::V2_INTERNAL_TAG);
                output.push(index);
            }
        }
    }
}

/// Trait that needs to be implemented by all contract refs.
pub trait ContractRef {
    /// Creates a new instance of the Contract Ref.
    fn new(env: Rc<ContractEnv>, address: Address) -> Self;
    /// Returns the address of the contract.
    fn address(&self) -> &Address;
    /// Creates a new contract reference with attached tokens, based on the current instance.
    ///
    /// If there are tokens attached to the current instance, the tokens will be attached
    /// to the next contract call.
    fn with_tokens(&self, tokens: U512) -> Self;
}

/// Represents the environment accessible in the contract context.
///
/// The `ContractEnv` struct provides methods for interacting with the contract environment,
/// such as accessing storage, calling other contracts, and handling various contract-related operations.
///
/// The `ContractEnv` is available for the user to use in the module code.
#[derive(Clone)]
pub struct ContractEnv {
    legacy_index: Option<u32>,
    path: Vec<StorageSlot>,
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
            legacy_index: Some(index),
            path: Vec::new(),
            mapping_data: Vec::new(),
            backend
        }
    }

    /// Returns the current storage key for the contract environment.
    pub(crate) fn current_key(&self) -> StorageKey {
        let mut result = [0u8; KEY_LEN];
        let key = self.key_bytes();
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
        self.slot_child(StorageSlot::user(index))
    }

    /// Returns an internal child contract environment with the specified index.
    pub(crate) fn internal_child(&self, index: u8) -> Self {
        self.slot_child(StorageSlot::internal(index))
    }

    fn slot_child(&self, slot: StorageSlot) -> Self {
        let mut path = self.path.clone();
        path.push(slot);

        Self {
            legacy_index: self.next_legacy_index(slot),
            path,
            mapping_data: self.mapping_data.clone(),
            backend: self.backend.clone()
        }
    }

    fn next_legacy_index(&self, slot: StorageSlot) -> Option<u32> {
        if !slot.is_legacy_compatible() {
            return None;
        }

        self.legacy_index
            .map(|index| index.wrapping_shl(4) | slot.raw_index() as u32)
    }

    fn key_bytes(&self) -> Vec<u8> {
        match self.legacy_index {
            Some(index) => {
                let mut key = Vec::with_capacity(INDEX_SIZE + self.mapping_data.len());
                key.extend_from_slice(index.to_be_bytes().as_ref());
                key.extend_from_slice(&self.mapping_data);
                key
            }
            None => {
                let path_bytes = self.encoded_path();
                let path_len = path_bytes.len() as u32;
                let mut key = Vec::with_capacity(
                    V2_KEY_TAG.len() + INDEX_SIZE + path_bytes.len() + self.mapping_data.len()
                );
                key.extend_from_slice(V2_KEY_TAG);
                key.extend_from_slice(path_len.to_be_bytes().as_ref());
                key.extend_from_slice(&path_bytes);
                key.extend_from_slice(&self.mapping_data);
                key
            }
        }
    }

    fn encoded_path(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.path.len() * 2);
        for slot in &self.path {
            slot.encode_v2(&mut bytes);
        }
        bytes
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
                .map_err(|_| ExecutionError::Formatting)
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
            .map_err(|_| ExecutionError::Formatting)
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

    /// Initializes the empty dictionary with the given name.
    pub fn init_dictionary<U: AsRef<str>>(&self, dictionary_name: U) {
        let dictionary_name = dictionary_name.as_ref();
        self.backend.borrow().init_dictionary(dictionary_name);
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

    /// Returns the current block time in milliseconds.
    pub fn get_block_time(&self) -> u64 {
        let backend = self.backend.borrow();
        backend.get_block_time()
    }

    /// Returns the current block time in milliseconds.
    pub fn get_block_time_millis(&self) -> u64 {
        let backend = self.backend.borrow();
        backend.get_block_time()
    }

    /// Returns the current block time in seconds.
    pub fn get_block_time_secs(&self) -> u64 {
        let backend = self.backend.borrow();
        backend.get_block_time().checked_div(1000).unwrap()
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

    /// Emits an event with the specified data using the native mechanism.
    pub fn emit_native_event<T: ToBytes + EventInstance>(&self, event: T) {
        let backend = self.backend.borrow();
        let result = event.to_bytes().map_err(ExecutionError::from);
        let bytes = result.unwrap_or_revert(self);
        backend.emit_native_event(&bytes.into())
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

    /// Delegate tokens to a validator
    ///
    /// # Arguments
    ///
    /// * `validator` - The validator to delegate to
    /// * `amount` - The amount of tokens to delegate
    pub fn delegate(&self, validator: PublicKey, amount: U512) {
        self.backend.borrow().delegate(validator, amount)
    }

    /// Undelegate tokens from a validator
    ///
    /// # Arguments
    ///
    /// * `validator` - The validator to undelegate from
    /// * `amount` - The amount of tokens to undelegate
    pub fn undelegate(&self, validator: PublicKey, amount: U512) {
        self.backend.borrow().undelegate(validator, amount)
    }

    /// Returns the amount of tokens delegated to a validator
    ///
    /// # Arguments
    ///
    /// * `validator` - The validator to get the delegated amount from
    ///
    /// # Returns
    ///
    /// The amount of tokens delegated to the validator
    pub fn delegated_amount(&self, validator: PublicKey) -> U512 {
        self.backend.borrow().delegated_amount(validator)
    }

    /// Returns information about the validator
    ///
    /// # Arguments
    /// - validator - The validator to query
    ///
    /// # Returns
    /// Option<ValidatorBid>
    pub fn get_validator_info(&self, validator: PublicKey) -> Option<ValidatorInfo> {
        self.backend.borrow().get_validator_info(validator)
    }

    /// Returns a vector of pseudorandom bytes of the specified size.
    /// There is no guarantee that the returned bytes are in any way cryptographically secure.
    pub fn pseudorandom_bytes(&self, size: usize) -> Vec<u8> {
        let seed_bytes = self.backend.borrow().pseudorandom_bytes();

        if size <= seed_bytes.len() {
            return seed_bytes[..size].to_vec();
        }

        // Use initial random bytes as seed for ChaCha8
        let mut result = seed_bytes.to_vec();
        let mut rng = ChaCha8Rng::from_seed(seed_bytes);
        let additional_bytes = size - result.len();
        let mut extra = vec![0u8; additional_bytes];
        rng.fill_bytes(&mut extra);
        result.extend_from_slice(&extra);

        result
    }

    /// Returns a pseudorandom integer.
    pub fn pseudorandom_number(&self, high: U512) -> U512 {
        let seed_bytes = self.backend.borrow().pseudorandom_bytes();
        let mut rng = ChaCha8Rng::from_seed(seed_bytes);
        let bits = high.bits();
        let bytes_len = bits.div_ceil(8);
        let max = U512::from(1u64) << bits; // 2^bits
        let limit = max - (max % high);
        loop {
            let mut bytes = vec![0u8; bytes_len];
            rng.fill_bytes(&mut bytes);
            let candidate = U512::from_big_endian(&bytes);

            if candidate < limit {
                return candidate % high;
            }
            // else: reject and try again
        }
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

    /// Migrates the schemas in the contract storage to the new schemas.
    pub fn migrate_schemas(&self, new_schemas: BTreeMap<String, Schema>) {
        let mut old_schemas: Schemas = self.env.get_named_value(EVENTS_SCHEMA).unwrap_or_default();

        for (name, new_schema) in new_schemas.iter() {
            match old_schemas.0.get(name) {
                // If the schema is not present in the old schemas, we add it.
                None => {
                    old_schemas.0.insert(name.clone(), new_schema.clone());
                }
                // If an existing schema is different from the new one, we revert.
                Some(old_schema) => {
                    if old_schema != new_schema {
                        self.env.revert(ExecutionError::SchemaMismatch);
                    }
                }
            }
        }

        // Store the updated schemas back to the contract storage.
        self.env.set_named_value(EVENTS_SCHEMA, old_schemas);
    }

    /// Emits an event with the specified data.
    pub fn emit_event<T: ToBytes + EventInstance>(&self, event: T) {
        self.env.emit_event(event);
    }
}
