//! This module provides types and traits for working with entrypoint arguments.

use crate::{contract_def::Argument, prelude::*, ContractEnv, ExecutionError};
use casper_types::{
    bytesrepr::{FromBytes, ToBytes}, CLType, CLTyped, Parameter, RuntimeArgs
};

/// A type that represents an entrypoint arg that may or may not be present.
#[derive(Debug, Clone)]
pub enum Maybe<T> {
    /// A value is present.
    Some(T),
    /// No value is present.
    None
}

impl<T> Maybe<T> {
    /// Returns `true` if the value is present.
    pub fn is_some(&self) -> bool {
        matches!(self, Maybe::Some(_))
    }

    /// Returns `true` if the value is not present.
    pub fn is_none(&self) -> bool {
        matches!(self, Maybe::None)
    }

    /// Unwraps the value.
    /// If the value is not present, the contract reverts with an `ExecutionError::UnwrapError`.
    pub fn unwrap(self, env: &ContractEnv) -> T {
        match self {
            Maybe::Some(value) => value,
            Maybe::None => env.revert(ExecutionError::UnwrapError)
        }
    }
}

impl<T: Default> Maybe<T> {
    /// Unwraps the value or returns the default value.
    pub fn unwrap_or_default(self) -> T {
        match self {
            Maybe::Some(value) => value,
            Maybe::None => T::default()
        }
    }
}

impl<T: ToBytes> ToBytes for Maybe<T> {
    fn to_bytes(&self) -> Result<Vec<u8>, casper_types::bytesrepr::Error> {
        match self {
            Maybe::Some(value) => value.to_bytes(),
            Maybe::None => Ok(Vec::new())
        }
    }

    fn serialized_length(&self) -> usize {
        match self {
            Maybe::Some(value) => value.serialized_length(),
            Maybe::None => 0
        }
    }
}

impl<T: FromBytes> FromBytes for Maybe<T> {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), casper_types::bytesrepr::Error> {
        let res = T::from_bytes(bytes);
        if let Ok((value, rem)) = res {
            Ok((Maybe::Some(value), rem))
        } else {
            Ok((Maybe::None, bytes))
        }
    }

    fn from_vec(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), casper_types::bytesrepr::Error> {
        Self::from_bytes(bytes.as_slice()).map(|(x, remainder)| (x, Vec::from(remainder)))
    }
}

/// A trait for types that can be used as entrypoint arguments.
pub trait EntrypointArgument: Sized {
    /// Returns `true` if the argument is required.
    fn is_required() -> bool;
    /// Returns the CLType of the argument.
    fn cl_type() -> CLType;
    /// Inserts the argument into the runtime args.
    fn insert_runtime_arg(self, name: &str, args: &mut RuntimeArgs);
    fn unwrap(value: Option<Self>, env: &ContractEnv) -> Self;
}

impl <T: CLTyped + ToBytes> EntrypointArgument for Maybe<T> {
    fn is_required() -> bool {
        false
    }

    fn cl_type() -> CLType {
        T::cl_type()
    }

    fn insert_runtime_arg(self, name: &str, args: &mut RuntimeArgs) {
        if let Maybe::Some(v) = self {
            let _ = args.insert(name, v);
        }
    }
    
    fn unwrap(value: Option<Self>, env: &ContractEnv) -> Self {
        match value {
            Some(v) => v,
            None => Maybe::None
        }
    }
}

impl <T: CLTyped + ToBytes> EntrypointArgument for T {
    fn is_required() -> bool {
        true
    }

    fn cl_type() -> CLType {
        T::cl_type()
    }

    fn insert_runtime_arg(self, name: &str, args: &mut RuntimeArgs) {
        let  _ = args.insert(name, self);
    }
    
    fn unwrap(value: Option<Self>, env: &ContractEnv) -> Self {
        match value {
            Some(v) => v,
            None => env.revert(ExecutionError::UnwrapError)
            
        }
    }
}

/// Converts a type into Casper's entrypoint argument representation.
pub fn into_parameter<T: EntrypointArgument>(name: &str) -> Option<Parameter> {
    match T::is_required() {
        true => Some(Parameter::new(name, T::cl_type())),
        false => None
    }
}

/// Converts a type into Odra's entrypoint argument representation.
pub fn into_argument<T: EntrypointArgument>(name: &str) -> Argument {
    Argument { 
        ident: name.to_string(), 
        ty: T::cl_type(), 
        is_ref: false, 
        is_slice: false,
        is_required: T::is_required()
    }
}

#[cfg(test)]
mod tests {
    use casper_types::U256;

    use crate::{contract_context::MockContractContext, Address};

    use super::*;

    #[test]
    fn test_maybe() {
        let some = Maybe::Some(1);
        let none: Maybe<u32> = Maybe::None;

        let ctx = MockContractContext::new();
        let env = ContractEnv::new(0, Rc::new(RefCell::new(ctx)));

        assert_eq!(some.is_some(), true);
        assert_eq!(some.is_none(), false);
        assert_eq!(some.clone().unwrap(&env), 1);
        assert_eq!(some.unwrap_or_default(), 1);

        assert_eq!(none.is_some(), false);
        assert_eq!(none.is_none(), true);
        assert_eq!(none.unwrap_or_default(), 0);
    }

    #[test]
    #[should_panic(expected = "revert")]
    fn unwrap_on_none() {
        let none: Maybe<u32> = Maybe::None;
        let mut ctx = MockContractContext::new();
        ctx.expect_revert().returning(|_| panic!("revert"));
        let env = ContractEnv::new(0, Rc::new(RefCell::new(ctx)));

        none.unwrap(&env);
    }

    #[test]
    fn test_into_args() {
        let args = vec![
            into_argument::<Maybe<u32>>("arg1"),
            into_argument::<U256>("arg2"),
            into_argument::<Option<String>>("arg3"),
        ];

        assert_eq!(args.len(), 3);
    }


    #[test]
    fn test_into_casper_parameters() {
        let params = vec![
            into_parameter::<Maybe<u32>>("arg1"),
            into_parameter::<Option<u32>>("arg2"),
            into_parameter::<Maybe<Option<u32>>>("arg3"),
            into_parameter::<Address>("arg4"),
        ]
        .into_iter()
        .filter_map(|x| x)
        .collect::<Vec<_>>();

        let params = vec![
            into_parameter::<String>("name"),
            into_parameter::<Maybe<String>>("metadata"),
        ]
        .into_iter()
        .filter_map(|x| x)
        .collect::<Vec<_>>();

        assert_eq!(params.len(), 2);
    }
}
