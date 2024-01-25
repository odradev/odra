//! A module containing callstack-related data structures.
//!
//! The callstack is used to keep track of the current call context. It is used to determine
//! the current account and contract address, the attached value, and the current entry point.
//!
//! The module provides building blocks for a callstack, such as `CallstackElement` and `ContractCall`.
use super::{casper_types::U512, Address, CallDef};
use crate::prelude::*;

/// A struct representing a callstack element.
#[derive(Clone)]
pub enum CallstackElement {
    /// An account address.
    Account(Address),
    /// A contract call.
    ContractCall {
        /// The address of the contract.
        address: Address,
        /// The contract call definition.
        call_def: CallDef
    }
}

impl CallstackElement {
    /// Creates a new element representing an account address.
    pub fn new_account(address: Address) -> Self {
        Self::Account(address)
    }

    /// Creates a new element representing a contract call.
    pub fn new_contract_call(address: Address, call_def: CallDef) -> Self {
        Self::ContractCall { address, call_def }
    }
}

impl CallstackElement {
    /// Returns the address of the callstack element.
    pub fn address(&self) -> &Address {
        match self {
            CallstackElement::Account(address) => address,
            CallstackElement::ContractCall { address, .. } => address
        }
    }
}

/// A struct representing a callstack.
#[derive(Clone, Default)]
pub struct Callstack(Vec<CallstackElement>);

impl Callstack {
    /// Returns the first (bottom most) callstack element.
    pub fn first(&self) -> CallstackElement {
        self.0
            .first()
            .expect("Not enough elements on callstack")
            .clone()
    }

    /// Returns the current callstack element and removes it from the callstack.
    pub fn pop(&mut self) -> Option<CallstackElement> {
        self.0.pop()
    }

    /// Pushes a new callstack element onto the callstack.
    pub fn push(&mut self, element: CallstackElement) {
        self.0.push(element);
    }

    /// Returns the attached value.
    ///
    /// If the current element is a contract call, the attached value is the amount of tokens
    /// attached to the contract call. If the current element is an account, the attached
    /// value is zero.
    pub fn attached_value(&self) -> U512 {
        let ce = self.0.last().expect("Not enough elements on callstack");
        match ce {
            CallstackElement::Account(_) => U512::zero(),
            CallstackElement::ContractCall { call_def, .. } => call_def.amount()
        }
    }

    /// Attaches the given amount of tokens to the current contract call.
    ///
    /// If the current element is not a contract call, this method does nothing.
    pub fn attach_value(&mut self, amount: U512) {
        if let Some(CallstackElement::ContractCall { call_def, .. }) = self.0.last_mut() {
            *call_def = call_def.clone().with_amount(amount);
        }
    }

    /// Returns the current callstack element.
    pub fn current(&self) -> &CallstackElement {
        self.0.last().expect("Not enough elements on callstack")
    }

    /// Returns the previous (second) callstack element.
    pub fn previous(&self) -> &CallstackElement {
        self.0
            .get(self.0.len() - 2)
            .expect("Not enough elements on callstack")
    }

    /// Returns the size of the callstack.
    pub fn size(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the callstack is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
