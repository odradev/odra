//! A module containing callstack-related data structures.
//!
//! The callstack is used to keep track of the current call context. It is used to determine
//! the current account and contract address, the attached value, and the current entry point.
//!
//! The module provides building blocks for a callstack, such as `CallstackElement` and `ContractCall`.
use crate::prelude::*;

use super::{casper_types::U512, Address, CallDef};

/// A struct representing a callstack element.
#[derive(Clone, Debug, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use casper_types::{account::AccountHash, RuntimeArgs};

    use super::*;

    #[test]
    fn test_first() {
        let mut callstack = Callstack::default();
        callstack.push(mock_account_element());
        callstack.push(mock_contract_element());

        assert_eq!(callstack.first(), mock_account_element());
    }

    #[test]
    fn test_pop() {
        let mut callstack = Callstack::default();
        callstack.push(mock_account_element());
        callstack.push(mock_contract_element());

        assert_eq!(callstack.pop(), Some(mock_contract_element()));
    }

    #[test]
    fn test_push() {
        let mut callstack = Callstack::default();
        callstack.push(mock_account_element());
        callstack.push(mock_contract_element());

        assert_eq!(callstack.size(), 2);
    }

    #[test]
    fn test_attached_value() {
        let mut callstack = Callstack::default();
        callstack.push(mock_account_element());
        assert_eq!(callstack.attached_value(), U512::zero());

        callstack.push(mock_contract_element_with_value(U512::from(100)));
        assert_eq!(callstack.attached_value(), U512::from(100));
    }

    #[test]
    fn test_attach_value() {
        let mut callstack = Callstack::default();
        callstack.push(mock_account_element());
        callstack.push(mock_contract_element());

        callstack.attach_value(U512::from(200));

        assert_eq!(
            callstack.current(),
            &mock_contract_element_with_value(U512::from(200))
        );
    }

    #[test]
    fn test_previous() {
        let mut callstack = Callstack::default();
        callstack.push(mock_account_element());
        callstack.push(mock_contract_element());

        assert_eq!(callstack.previous(), &mock_account_element());
    }

    #[test]
    fn test_size() {
        let mut callstack = Callstack::default();

        callstack.push(mock_account_element());
        callstack.push(mock_contract_element());

        assert_eq!(callstack.size(), 2);
    }

    #[test]
    fn test_is_empty() {
        let mut callstack = Callstack::default();
        assert!(callstack.is_empty());

        callstack.push(mock_account_element());
        assert!(!callstack.is_empty());
    }

    const PACKAGE_HASH: &str =
        "package-7ba9daac84bebee8111c186588f21ebca35550b6cf1244e71768bd871938be6a";
    const ACCOUNT_HASH: &str =
        "account-hash-3b4ffcfb21411ced5fc1560c3f6ffed86f4885e5ea05cde49d90962a48a14d95";

    fn mock_account_element() -> CallstackElement {
        CallstackElement::Account(Address::Account(
            AccountHash::from_formatted_str(ACCOUNT_HASH).unwrap()
        ))
    }

    fn mock_contract_element() -> CallstackElement {
        CallstackElement::new_contract_call(
            Address::new(PACKAGE_HASH).unwrap(),
            CallDef::new("a", false, RuntimeArgs::default())
        )
    }

    fn mock_contract_element_with_value(amount: U512) -> CallstackElement {
        CallstackElement::new_contract_call(
            Address::new(PACKAGE_HASH).unwrap(),
            CallDef::new("a", false, RuntimeArgs::default()).with_amount(amount)
        )
    }
}
