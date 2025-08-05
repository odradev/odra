//! A module containing callstack-related data structures.
//!
//! The callstack is used to keep track of the current call context. It is used to determine
//! the current account and contract address, the attached value, and the current entry point.
//!
//! The module provides building blocks for a callstack, such as `CallstackElement` and `ContractCall`.
use super::{casper_types::U512, CallDef};
use crate::prelude::*;

/// A struct representing a callstack element.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CallstackElement {
    /// An account address.
    Account(Address),
    /// A contract call.
    ContractCall {
        /// The name of the contract.
        contract_name: String,
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
    pub fn new_contract_call(contract_name: String, address: Address, call_def: CallDef) -> Self {
        Self::ContractCall {
            contract_name,
            address,
            call_def
        }
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
pub struct Callstack {
    elements: Vec<CallstackElement>,
    stack_record: Vec<String>
}

impl Callstack {
    /// Returns the first (bottom most) callstack element.
    pub fn first(&self) -> CallstackElement {
        self.elements
            .first()
            .expect("Not enough elements on callstack")
            .clone()
    }

    /// Returns the current callstack element and removes it from the callstack.
    pub fn pop(&mut self) -> Option<CallstackElement> {
        self.elements.pop()
    }

    /// Pushes a new callstack element onto the callstack.
    pub fn push(&mut self, element: CallstackElement) {
        self.elements.push(element);
    }

    /// Returns the attached value.
    ///
    /// If the current element is a contract call, the attached value is the amount of tokens
    /// attached to the contract call. If the current element is an account, the attached
    /// value is zero.
    pub fn attached_value(&self) -> U512 {
        let ce = self
            .elements
            .last()
            .expect("Not enough elements on callstack");
        match ce {
            CallstackElement::Account(_) => U512::zero(),
            CallstackElement::ContractCall { call_def, .. } => call_def.amount()
        }
    }

    /// Attaches the given amount of tokens to the current contract call.
    ///
    /// If the current element is not a contract call, this method does nothing.
    pub fn attach_value(&mut self, amount: U512) {
        if let Some(CallstackElement::ContractCall { call_def, .. }) = self.elements.last_mut() {
            *call_def = call_def.clone().with_amount(amount);
        }
    }

    /// Returns the current callstack element.
    pub fn current(&self) -> &CallstackElement {
        self.elements
            .last()
            .expect("Not enough elements on callstack")
    }

    /// Returns the previous (second) callstack element.
    pub fn previous(&self) -> &CallstackElement {
        self.elements
            .get(self.elements.len() - 2)
            .expect("Not enough elements on callstack")
    }

    /// Returns the size of the callstack.
    pub fn size(&self) -> usize {
        self.elements.len()
    }

    /// Returns `true` if the callstack is empty.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Stringifies the callstack elements and stores them in the stack record.
    pub fn record(&mut self) {
        self.stack_record.clear();
        for element in self.elements.iter().rev() {
            match element {
                CallstackElement::Account(address) => {
                    self.stack_record.push(format!("  ↳ caller: {:?}", address));
                }
                CallstackElement::ContractCall {
                    contract_name,
                    call_def,
                    ..
                } => {
                    self.stack_record
                        .push(format!("  ↳ contract: {:?}", contract_name));
                    self.stack_record
                        .push(format!("    ↳ entry point: {}", call_def.entry_point()));
                    let args: Vec<String> = call_def
                        .args()
                        .named_args()
                        .map(|arg| {
                            let mut arg_json = serde_json::to_value(arg.cl_value())
                                .unwrap_or(serde_json::Value::Null);
                            arg_json.as_object_mut().unwrap().remove("bytes");
                            format!(
                                "      ↳ arg: {:?} - {}",
                                arg.name(),
                                serde_json::to_string(&arg_json).unwrap_or_default()
                            )
                        })
                        .collect();
                    self.stack_record.extend(args);
                }
            }
        }
    }

    /// Returns the stack record as a string.
    pub fn record_to_string(&self) -> String {
        self.stack_record.join("\n")
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
            "mock_contract".to_string(),
            Address::new(PACKAGE_HASH).unwrap(),
            CallDef::new("a", false, RuntimeArgs::default())
        )
    }

    fn mock_contract_element_with_value(amount: U512) -> CallstackElement {
        CallstackElement::new_contract_call(
            "mock_contract".to_string(),
            Address::new(PACKAGE_HASH).unwrap(),
            CallDef::new("a", false, RuntimeArgs::default()).with_amount(amount)
        )
    }
}
