use casper_types::CLValue;

use crate::call_def::CallDef;
use crate::casper_types::bytesrepr::Bytes;
use crate::casper_types::U512;
use crate::prelude::*;

/// Trait representing the context of a smart contract.
#[cfg_attr(test, allow(unreachable_code))]
#[cfg_attr(test, mockall::automock)]
pub trait ContractContext {
    /// Retrieves from the storage the value associated with the given key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to retrieve the value for.
    ///
    /// # Returns
    ///
    /// An `Option` containing the value associated with the key, or `None` if the key is not found.
    fn get_value(&self, key: &[u8]) -> Option<Bytes>;

    /// Writes to the storage the value associated with the given key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to set the value for.
    /// * `value` - The value to be set.
    fn set_value(&self, key: &[u8], value: Bytes);

    /// Retrieves the value behind a named key.
    ///
    /// # Arguments
    ///  
    /// * `name` - The name of the key.
    fn get_named_value(&self, name: &str) -> Option<Bytes>;

    /// Sets the value behind a named key.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the key.
    /// * `value` - The value to set.
    fn set_named_value(&self, name: &str, value: CLValue);

    /// Retrieves the key value behind a named dictionary.
    ///
    /// # Arguments
    ///
    /// * `dictionary_name` - The name of the dictionary.
    /// * `key` - The key to retrieve the value for.
    fn get_dictionary_value(&self, dictionary_name: &str, key: &[u8]) -> Option<Bytes>;

    /// Sets the key value behind a named dictionary.
    ///
    /// # Arguments
    ///
    /// * `dictionary_name` - The name of the dictionary.
    /// * `key` - The key to set the value for.
    /// * `value` - The value to set.
    fn set_dictionary_value(&self, dictionary_name: &str, key: &[u8], value: CLValue);

    /// Removes the named key from the storage.
    ///
    /// # Arguments
    /// * `dictionary_name` - The name of the dictionary.
    fn remove_dictionary(&self, dictionary_name: &str);

    /// Retrieves the address of the caller.
    fn caller(&self) -> Address;

    /// Retrieves the address of the current contract.
    fn self_address(&self) -> Address;

    /// Calls another contract at the specified address with the given call definition.
    ///
    /// # Arguments
    ///
    /// * `address` - The address of the contract to call.
    /// * `call_def` - The call definition specifying the method and arguments to call.
    ///
    /// # Returns
    ///
    /// The result of the contract call as a byte array.
    fn call_contract(&self, address: Address, call_def: CallDef) -> Bytes;

    /// Retrieves the current block time.
    ///
    /// # Returns
    ///
    /// The current block time as a `u64` value.
    fn get_block_time(&self) -> u64;

    /// Retrieves the value attached to the call.
    ///
    /// # Returns
    ///
    /// The attached value as a `U512` value.
    fn attached_value(&self) -> U512;

    /// Retrieves the balance of the current contract.
    ///
    /// # Returns
    /// The balance of the current contract in U512
    fn self_balance(&self) -> U512;

    /// Emits an event with the specified event data.
    ///
    /// # Arguments
    ///
    /// * `event` - The event data to emit.
    fn emit_event(&self, event: &Bytes);

    /// Transfers tokens to the specified address.
    ///
    /// # Arguments
    ///
    /// * `to` - The address to transfer the tokens to.
    /// * `amount` - The amount of tokens to transfer.
    fn transfer_tokens(&self, to: &Address, amount: &U512);

    /// Reverts the contract execution with the specified error.
    ///
    /// # Arguments
    ///
    /// * `error` - The error to revert with.
    ///
    /// # Panics
    ///
    /// This function will panic and abort the contract execution.
    fn revert(&self, error: OdraError) -> !;

    /// Retrieves the value of the named argument as a byte array.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the argument.
    ///
    /// # Returns
    ///
    /// The value of the named argument as a byte array.
    fn get_named_arg_bytes(&self, name: &str) -> OdraResult<Bytes>;

    /// Similar to `get_named_arg_bytes`, but returns `None` if the named argument is not present.
    fn get_opt_named_arg_bytes(&self, name: &str) -> Option<Bytes>;

    /// Handles the value attached to the call. Sets the value in the contract context.
    fn handle_attached_value(&self);

    /// Clears the value attached to the call.
    fn clear_attached_value(&self);

    /// Computes the hash of the given byte array.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The byte array to compute the hash for.
    ///
    /// # Returns
    ///
    /// The computed hash as a fixed-size byte array of length 32.
    fn hash(&self, bytes: &[u8]) -> [u8; 32];
}
