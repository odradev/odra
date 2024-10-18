use crate::casper_types::bytesrepr::{Bytes, ToBytes};
use crate::prelude::*;
use crate::utils::extract_event_name;
use casper_event_standard::EventInstance;

/// Represents the result of a contract call. Includes external contracts calls.
///
/// The result contains the address of the called contract, the address of the caller, the amount of gas
/// used, the result of the call, and the events emitted by the contract.
#[derive(Debug, Clone)]
pub(crate) struct CallResult {
    contract_address: Address,
    caller: Address,
    gas_used: u64,
    result: OdraResult<Bytes>,
    events: BTreeMap<Address, Vec<Bytes>>
}

impl CallResult {
    /// Creates a new `CallResult` instance with the specified parameters.
    pub fn new(
        contract_address: Address,
        caller: Address,
        gas_used: u64,
        result: OdraResult<Bytes>,
        events: BTreeMap<Address, Vec<Bytes>>
    ) -> Self {
        Self {
            contract_address,
            caller,
            gas_used,
            result,
            events
        }
    }

    /// Returns the result of the contract call as a [Bytes] object.
    ///
    /// # Panics
    ///
    /// Panics if the result is an error.
    pub fn bytes(&self) -> Bytes {
        match &self.result {
            Ok(result) => result.clone(),
            Err(error) => {
                panic!("Last call result is an error: {:?}", error);
            }
        }
    }

    /// Returns the result of the contract call as a [OdraResult] object.
    pub fn result(&self) -> OdraResult<Bytes> {
        self.result.clone()
    }

    /// Returns the error of the contract call as an `OdraError` object.
    pub fn error(&self) -> OdraError {
        match &self.result {
            Ok(_) => {
                panic!("Last call result is not an error");
            }
            Err(error) => error.clone()
        }
    }

    /// Returns the address of the caller.
    pub fn caller(&self) -> Address {
        self.caller
    }

    /// Returns the amount of gas used in the contract call.
    pub fn gas_used(&self) -> u64 {
        self.gas_used
    }

    /// Returns the address of the contract.
    pub fn contract_address(&self) -> Address {
        self.contract_address
    }

    /// Returns the names of the events emitted by the contract at the given address.
    pub fn event_names(&self, contract_address: &Address) -> Vec<String> {
        self.events
            .get(contract_address)
            .unwrap_or(&vec![])
            .iter()
            .map(|event_bytes| extract_event_name(event_bytes).unwrap())
            .collect()
    }

    /// Returns the events emitted by the contract at the given address.
    pub fn contract_events(&self, contract_address: &Address) -> Vec<Bytes> {
        self.events.get(contract_address).unwrap_or(&vec![]).clone()
    }

    /// Checks if the specified event has been emitted by the contract at the given address.
    pub fn emitted(&self, contract_address: &Address, event_name: &str) -> bool {
        self.event_names(contract_address)
            .contains(&event_name.to_string())
    }

    /// Checks if the specified event instance has been emitted by the contract at the given address.
    pub fn emitted_event<T: ToBytes + EventInstance>(
        &self,
        contract_address: &Address,
        event: &T
    ) -> bool {
        self.contract_events(contract_address)
            .contains(&Bytes::from(event.to_bytes().unwrap()))
    }

    /// Returns a wrapper [ContractCallResult] object containing the current `CallResult` and the given contract address.
    pub fn contract_last_call(self, contract_address: Address) -> ContractCallResult {
        ContractCallResult {
            call_result: self,
            contract_address
        }
    }
}

/// Represents the result of a contract call.
///
/// It may represent not the original call but the an external call.
/// However the result and gas used come from the original call.
#[derive(Debug, Clone)]
pub struct ContractCallResult {
    call_result: CallResult,
    contract_address: Address
}

impl ContractCallResult {
    /// Returns the address of the contract.
    ///
    /// # Returns
    ///
    /// The address of the contract.
    pub fn contract_address(&self) -> Address {
        self.contract_address
    }

    /// Returns the address of the callee contract.
    ///
    /// # Returns
    ///
    /// The address of the callee contract.
    pub fn callee_contract_address(&self) -> Address {
        self.call_result.contract_address()
    }

    /// Returns the result of the original contract call as a [Bytes] object.
    ///
    /// # Panics
    ///
    /// Panics if the result is an error.
    pub fn callee_contract_bytes(&self) -> Bytes {
        self.call_result.bytes()
    }

    /// Returns the result of the original contract call as a [OdraResult] object.
    pub fn callee_contract_result(&self) -> OdraResult<Bytes> {
        self.call_result.result()
    }

    /// Returns the error of the original contract call as an `OdraError` object.
    pub fn callee_contract_error(&self) -> OdraError {
        self.call_result.error()
    }

    /// Returns the address of the original contract caller.
    pub fn callee_contract_caller(&self) -> Address {
        self.call_result.caller()
    }

    /// Returns the amount of gas used in the original contract call.
    pub fn callee_contract_gas_used(&self) -> u64 {
        self.call_result.gas_used()
    }

    /// Returns the names of the events emitted by the contract call.
    ///
    /// # Returns
    ///
    /// A vector containing the names of the events emitted by the contract call.
    pub fn event_names(&self) -> Vec<String> {
        self.call_result.event_names(&self.contract_address)
    }

    /// Returns the events emitted by the contract call.
    ///
    /// # Returns
    ///
    /// A vector containing the events emitted by the contract call.
    pub fn events(&self) -> Vec<Bytes> {
        self.call_result.contract_events(&self.contract_address)
    }

    /// Checks if an event with the specified name was emitted by the contract call.
    ///
    /// # Arguments
    ///
    /// * `event_name` - The name of the event to check.
    ///
    /// # Returns
    ///
    /// `true` if the event was emitted, otherwise `false`.
    pub fn emitted(&self, event_name: &str) -> bool {
        self.call_result.emitted(&self.contract_address, event_name)
    }

    /// Checks if the specified event instance was emitted by the contract call.
    ///
    /// # Arguments
    ///
    /// * `event` - The event instance to check.
    ///
    /// # Returns
    ///
    /// `true` if the event was emitted, otherwise `false`.
    pub fn emitted_event<T: ToBytes + EventInstance>(&self, event: &T) -> bool {
        self.call_result
            .emitted_event(&self.contract_address, event)
    }
}
