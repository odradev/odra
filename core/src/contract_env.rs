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

const KEY_LEN: usize = 64;
pub(crate) type StorageKey = [u8; KEY_LEN];

/// Maximum nesting depth for module paths.
pub(crate) const MAX_PATH_LEN: usize = 8;

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
    path: [u8; MAX_PATH_LEN],
    path_len: u8,
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
    pub const fn new(backend: Rc<RefCell<dyn ContractContext>>) -> Self {
        Self {
            path: [0u8; MAX_PATH_LEN],
            path_len: 0,
            mapping_data: Vec::new(),
            backend
        }
    }

    /// Returns the index bytes for the current path, using the appropriate encoding.
    ///
    /// Two encoding modes exist to support backward compatibility:
    ///
    /// **Legacy encoding** (default, when all path indices fit in 4 bits):
    /// Packs indices into a `u32` using 4-bit left shifts, identical to the original
    /// `(parent << 4) + child` formula. Produces 4 big-endian bytes. This ensures
    /// deployed contracts with ≤15 fields per module get the same storage keys.
    ///
    /// **Path encoding** (indices > 15, or V2 mode):
    /// Emits `[0xFF, path_len, path[0], ..., path[n]]`. The `0xFF` prefix cannot
    /// collide with legacy keys (whose first byte never exceeds `0x0F`). The
    /// `path_len` byte makes the boundary with appended `mapping_data` unambiguous,
    /// preventing collisions between e.g. a `Var` at a deeper path and a `Mapping`
    /// at a shallower path with matching key bytes.
    ///
    /// **Why `path_len` is necessary — collision example:**
    ///
    /// Consider two fields whose path bytes and mapping data concatenate identically:
    /// - Field A: `Var` at path `[3, 5]` (depth 2), no mapping data.
    /// - Field B: `Mapping` at path `[3]` (depth 1), mapping key serializes to `[5]`.
    ///
    /// The final hash input is `index_bytes ++ mapping_data`.
    ///
    /// Without `path_len` (hypothetical `[0xFF, path..., mapping_data...]`):
    /// - A → `[0xFF, 3, 5]`, B → `[0xFF, 3] ++ [5]` = `[0xFF, 3, 5]` — **collision!**
    ///
    /// With `path_len` (actual `[0xFF, path_len, path..., mapping_data...]`):
    /// - A → `[0xFF, 2, 3, 5]`, B → `[0xFF, 1, 3] ++ [5]` = `[0xFF, 1, 3, 5]` — **distinct.**
    pub(crate) fn index_bytes(&self) -> Vec<u8> {
        let path = &self.path[..self.path_len as usize];
        // Legacy: pack indices into u32 via 4-bit shifts (e.g. path [3, 15] → 0x3F).
        // Only used when all indices fit in a nibble, preserving old storage keys.
        if path.iter().all(|&idx| idx <= 15) {
            let index: u32 = path.iter().fold(0u32, |acc, &idx| (acc << 4) + idx as u32);
            index.to_be_bytes().to_vec()
        } else {
            // Path encoding: [0xFF, len, idx_0, idx_1, ...]. Used for fields 16+.
            let mut bytes = Vec::with_capacity(2 + path.len());
            bytes.push(0xFF);
            bytes.push(self.path_len);
            bytes.extend_from_slice(path);
            bytes
        }
    }

    /// Returns the current storage key for the contract environment.
    pub(crate) fn current_key(&self) -> StorageKey {
        let mut result = [0u8; KEY_LEN];
        let index_bytes = self.index_bytes();
        let mut key = Vec::with_capacity(index_bytes.len() + self.mapping_data.len());
        key.extend_from_slice(&index_bytes);
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
        let mut new_path = self.path;
        let Some(slot) = new_path.get_mut(self.path_len as usize) else {
            self.revert(ExecutionError::PathIndexOutOfBounds)
        };
        *slot = index;
        
        Self {
            path: new_path,
            path_len: self.path_len + 1,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract_context::MockContractContext;

    fn make_env() -> ContractEnv {
        let mut ctx = MockContractContext::new();
        ctx.expect_hash().returning(|input| {
            let mut result = [0u8; 32];
            for (i, byte) in input.iter().enumerate() {
                if i < 32 {
                    result[i] = *byte;
                }
            }
            result
        });
        ContractEnv::new(Rc::new(RefCell::new(ctx)))
    }

    fn legacy_u32_for_path(path: &[u8]) -> u32 {
        path.iter().fold(0u32, |acc, &idx| (acc << 4) + idx as u32)
    }

    #[test]
    fn encoding_matches_old_u32_formula() {
        let env = make_env();
        let child = env.child(3);
        assert_eq!(
            child.index_bytes(),
            legacy_u32_for_path(&[3]).to_be_bytes().to_vec()
        );

        let grandchild = child.child(15);
        assert_eq!(
            grandchild.index_bytes(),
            legacy_u32_for_path(&[3, 15]).to_be_bytes().to_vec()
        );

        let deep = env.child(1).child(2).child(3).child(4);
        assert_eq!(
            deep.index_bytes(),
            legacy_u32_for_path(&[1, 2, 3, 4]).to_be_bytes().to_vec()
        );
    }

    #[test]
    fn path_encoding_used_for_indices_above_15() {
        let env = make_env();
        let child = env.child(3).child(16);
        let bytes = child.index_bytes();
        assert_eq!(bytes[0], 0xFF);
        assert_eq!(bytes[1], 2);
        assert_eq!(bytes[2], 3);
        assert_eq!(bytes[3], 16);
    }

    #[test]
    fn no_collision_between_var_and_mapping() {
        let env = make_env();

        let var_key = env.child(3).child(16).current_key();
        let mut map_env = env.child(3);
        map_env.add_to_mapping_data(&[16]);
        let map_key = map_env.current_key();
        assert_ne!(var_key, map_key);

        let var_key2 = env.child(3).child(1).current_key();
        let mut map_env2 = env.child(3);
        map_env2.add_to_mapping_data(&[0xFF, 2, 3, 1]);
        let map_key2 = map_env2.current_key();
        assert_ne!(var_key2, map_key2);
    }

    #[test]
    fn no_collision_between_small_and_path_encoding() {
        let env = make_env();
        let small_key = env.child(1).child(2).current_key();
        let path_key = env.child(1).child(20).current_key();
        assert_ne!(small_key, path_key);
    }
}
