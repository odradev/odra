use core::any::Any;

use alloc::{boxed::Box, string::String};

use crate::arithmetic::ArithmeticsError;

const MAX_USER_ERROR: u16 = 32767;
const USER_ERROR_TOO_HIGH: u16 = 32768;
const UNWRAP_ERROR: u16 = 1;

const CODE_ADDITION_OVERFLOW: u16 = 100;
const CODE_SUBTRACTION_OVERFLOW: u16 = 101;
const CODE_NON_PAYABLE: u16 = 102;
const CODE_TRANSFER_TO_CONTRACT: u16 = 103;
const CODE_REENTRANT_CALL: u16 = 104;
const CODE_CONTRACT_ALREADY_INSTALLED: u16 = 105;
const CODE_UNKNOWN_CONSTRUCTOR: u16 = 106;
const CODE_NATIVE_TRANSFER_ERROR: u16 = 107;
const CODE_INDEX_OUT_OF_BOUNDS: u16 = 108;
const CODE_ZERO_ADDRESS: u16 = 109;
const CODE_ADDRESS_CREATION_FAILED: u16 = 110;
const CODE_SERIALIZATION_FAILED: u16 = 111;
const CODE_KEY_NOT_FOUND: u16 = 112;

/// General error type in Odra framework
#[derive(Clone, Debug, PartialEq)]
pub enum OdraError {
    /// An error that can occur during smart contract execution
    ExecutionError(ExecutionError),
    /// An internal virtual machine error
    VmError(VmError)
}

impl OdraError {
    pub fn code(&self) -> u16 {
        match self {
            OdraError::ExecutionError(e) => e.code(),
            OdraError::VmError(e) => 123
        }
    }
}

impl From<ArithmeticsError> for ExecutionError {
    fn from(error: ArithmeticsError) -> Self {
        match error {
            ArithmeticsError::AdditionOverflow => Self::addition_overflow(),
            ArithmeticsError::SubtractingOverflow => Self::subtraction_overflow()
        }
    }
}

impl From<ArithmeticsError> for OdraError {
    fn from(error: ArithmeticsError) -> Self {
        Into::<ExecutionError>::into(error).into()
    }
}

impl From<Box<dyn Any + Send>> for OdraError {
    fn from(_: Box<dyn Any + Send>) -> Self {
        OdraError::VmError(VmError::Panic)
    }
}

impl From<casper_types::bytesrepr::Error> for ExecutionError {
    fn from(value: casper_types::bytesrepr::Error) -> Self {
        Self::sys(
            CODE_SERIALIZATION_FAILED,
            match value {
                casper_types::bytesrepr::Error::EarlyEndOfStream => "Early end of stream",
                casper_types::bytesrepr::Error::Formatting => "Formatting",
                casper_types::bytesrepr::Error::LeftOverBytes => "Leftover bytes",
                casper_types::bytesrepr::Error::OutOfMemory => "Out of memory",
                casper_types::bytesrepr::Error::NotRepresentable => "Not representable",
                casper_types::bytesrepr::Error::ExceededRecursionDepth => {
                    "Exceeded recursion depth"
                }
                _ => "Serialization failed"
            }
        )
    }
}

/// An error that can occur during smart contract execution
///
/// It is represented by an error code and a human-readable message.
///
/// Errors codes 0..32767 are available for the user to define custom error
/// in smart contracts.
/// 32768 code is a special code representing a violation of the custom error code space.
///
/// The rest of codes 32769..[u16::MAX](u16::MAX), are used internally by the framework.
#[derive(Clone, Debug)]
pub struct ExecutionError(u16, String);

impl ExecutionError {
    /// Creates an instance with specified code and message.
    ///
    /// If the custom error code space is violated, an error with code 32768 is returned.
    pub fn new(code: u16, msg: &str) -> Self {
        if code > MAX_USER_ERROR {
            Self(
                USER_ERROR_TOO_HIGH,
                String::from("User error too high. The code should be in range 0..32767.")
            )
        } else {
            Self(code, String::from(msg))
        }
    }

    /// Creates an instance of a system error.
    fn sys(code: u16, msg: &str) -> Self {
        ExecutionError(code + USER_ERROR_TOO_HIGH, String::from(msg))
    }

    /// Return the underlying error code
    pub fn code(&self) -> u16 {
        self.0
    }

    /// Creates a specific type of error, meaning that value unwrapping failed.
    pub fn unwrap_error() -> Self {
        Self::sys(UNWRAP_ERROR, "Unwrap error")
    }

    pub fn non_payable() -> Self {
        Self::sys(CODE_NON_PAYABLE, "Method does not accept deposit")
    }

    pub fn can_not_transfer_to_contract() -> Self {
        Self::sys(
            CODE_TRANSFER_TO_CONTRACT,
            "Can't transfer tokens to contract."
        )
    }

    pub fn reentrant_call() -> Self {
        Self::sys(CODE_REENTRANT_CALL, "Reentrant call.")
    }

    pub fn contract_already_installed() -> Self {
        Self::sys(
            CODE_CONTRACT_ALREADY_INSTALLED,
            "Contract already installed."
        )
    }

    pub fn unknown_constructor() -> Self {
        Self::sys(CODE_UNKNOWN_CONSTRUCTOR, "Unknown constructor.")
    }

    pub fn native_token_transfer_error() -> Self {
        Self::sys(CODE_NATIVE_TRANSFER_ERROR, "Native token transfer error.")
    }

    pub fn addition_overflow() -> Self {
        Self::sys(CODE_ADDITION_OVERFLOW, "Addition overflow")
    }

    pub fn subtraction_overflow() -> Self {
        Self::sys(CODE_SUBTRACTION_OVERFLOW, "Subtraction overflow")
    }

    pub fn index_out_of_bounds() -> Self {
        Self::sys(CODE_INDEX_OUT_OF_BOUNDS, "Index out of bounds")
    }

    pub fn zero_address() -> Self {
        Self::sys(CODE_ZERO_ADDRESS, "Zero address")
    }

    pub fn address_creation_failed() -> Self {
        Self::sys(CODE_ADDRESS_CREATION_FAILED, "Address creation failed")
    }

    pub fn key_not_found() -> Self {
        Self::sys(CODE_KEY_NOT_FOUND, "Key not found")
    }
}

impl PartialEq for ExecutionError {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl From<ExecutionError> for OdraError {
    fn from(error: ExecutionError) -> Self {
        Self::ExecutionError(error)
    }
}

/// An internal virtual machine error
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VmError {
    /// Failed to serialize a value.
    Serialization,
    /// Failed to deserialize a value.
    Deserialization,
    /// Exceeded the account balance
    BalanceExceeded,
    /// Non existing host entrypoint was called.
    NoSuchMethod(String),
    /// Accessing a contract with an invalid address.
    InvalidContractAddress,
    /// Error calling a host function in a wrong context.
    InvalidContext,
    /// Calling a contract with missing entrypoint arguments.
    MissingArg,
    /// Non-specified error with a custom message.
    Other(String),
    /// Unspecified error.
    Panic
}

/// Error that can occur while operating on a collection.
pub enum CollectionError {
    // The requested index is bigger than the max collection index.
    IndexOutOfBounds
}

impl From<CollectionError> for ExecutionError {
    fn from(error: CollectionError) -> Self {
        match error {
            CollectionError::IndexOutOfBounds => Self::index_out_of_bounds()
        }
    }
}

impl From<CollectionError> for OdraError {
    fn from(error: CollectionError) -> Self {
        Into::<ExecutionError>::into(error).into()
    }
}

/// Error that can occur while operating on an Address.
#[derive(Clone, Debug, PartialEq)]
pub enum AddressError {
    /// Tried to construct a zero address.
    ZeroAddress,
    /// Tried to construct an address and failed.
    AddressCreationError
}

impl From<AddressError> for ExecutionError {
    fn from(error: AddressError) -> Self {
        match error {
            AddressError::ZeroAddress => Self::zero_address(),
            AddressError::AddressCreationError => Self::address_creation_failed()
        }
    }
}

impl From<AddressError> for OdraError {
    fn from(error: AddressError) -> Self {
        Into::<ExecutionError>::into(error).into()
    }
}
