use core::any::Any;

use casper_types::bytesrepr::Error as BytesReprError;
use casper_types::{CLType, CLValueError};

use crate::arithmetic::ArithmeticsError;
use crate::prelude::*;
use crate::VmError::Serialization;

/// General error type in Odra framework
#[repr(u16)]
#[derive(Clone, Debug, PartialEq)]
pub enum OdraError {
    /// An error that can occur during smart contract execution
    ExecutionError(ExecutionError),
    /// An internal virtual machine error
    VmError(VmError)
}

impl OdraError {
    /// Returns the error code.
    pub fn code(&self) -> u16 {
        match self {
            OdraError::ExecutionError(e) => e.code(),
            OdraError::VmError(_e) => 0
        }
    }

    /// Creates a new user error with a given code.
    pub fn user(code: u16) -> Self {
        if code >= ExecutionError::UserErrorTooHigh.code() {
            ExecutionError::UserErrorTooHigh.into()
        } else {
            ExecutionError::User(code).into()
        }
    }
}

impl From<ArithmeticsError> for ExecutionError {
    fn from(error: ArithmeticsError) -> Self {
        match error {
            ArithmeticsError::AdditionOverflow => Self::AdditionOverflow,
            ArithmeticsError::SubtractingOverflow => Self::SubtractionOverflow
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
    fn from(error: casper_types::bytesrepr::Error) -> Self {
        match error {
            casper_types::bytesrepr::Error::EarlyEndOfStream => Self::EarlyEndOfStream,
            casper_types::bytesrepr::Error::Formatting => Self::Formatting,
            casper_types::bytesrepr::Error::LeftOverBytes => Self::LeftOverBytes,
            casper_types::bytesrepr::Error::OutOfMemory => Self::OutOfMemory,
            casper_types::bytesrepr::Error::NotRepresentable => Self::NotRepresentable,
            casper_types::bytesrepr::Error::ExceededRecursionDepth => Self::ExceededRecursionDepth,
            _ => Self::Formatting
        }
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
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ExecutionError {
    /// Unwrap error.
    UnwrapError = 1,
    /// Something unexpected happened.
    UnexpectedError = 2,
    /// Addition overflow
    AdditionOverflow = 100,
    /// Subtraction overflow
    SubtractionOverflow = 101,
    /// Method does not accept deposit
    NonPayable = 102,
    /// Can't transfer tokens to contract.
    TransferToContract = 103,
    /// Reentrant call detected
    ReentrantCall = 104,
    /// Contract already installed
    ContractAlreadyInstalled = 105,
    /// Unknown constructor
    UnknownConstructor = 106,
    /// Native transfer error
    NativeTransferError = 107,
    /// Index out of bounds
    IndexOutOfBounds = 108,
    /// Tried to construct a zero address.
    ZeroAddress = 109,
    /// Address creation failed
    AddressCreationFailed = 110,
    /// Early end of stream - deserialization error
    EarlyEndOfStream = 111,
    /// Formatting error - deserialization error
    Formatting = 112,
    /// Left over bytes - deserialization error
    LeftOverBytes = 113,
    /// Out of memory
    OutOfMemory = 114,
    /// Not representable
    NotRepresentable = 115,
    /// Exceeded recursion depth
    ExceededRecursionDepth = 116,
    /// Key not found
    KeyNotFound = 117,
    /// Could not deserialize signature
    CouldNotDeserializeSignature = 118,
    /// Type mismatch
    TypeMismatch = 119,
    /// Could not sign message
    CouldNotSignMessage = 120,
    /// Empty dictionary name
    EmptyDictionaryName = 121,
    /// Calling a contract with missing entrypoint arguments.
    MissingArg = 122,
    /// Reading the address from the storage failed.
    MissingAddress = 123,
    /// Out of gas error
    OutOfGas = 124,
    /// Maximum code for user errors
    MaxUserError = 64535,
    /// User error too high. The code should be in range 0..32767.
    UserErrorTooHigh = 64536,
    /// User error
    User(u16)
}

impl ExecutionError {
    /// Returns the error code.
    pub fn code(&self) -> u16 {
        unsafe {
            match self {
                ExecutionError::User(code) => *code,
                ExecutionError::MaxUserError => 64535,
                ExecutionError::UserErrorTooHigh => 64536,
                _ => ExecutionError::UserErrorTooHigh.code() + *(self as *const Self as *const u16)
            }
        }
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
    /// Calling a contract with a wrong argument type.
    TypeMismatch {
        /// Expected type.
        expected: CLType,
        /// Found type.
        found: CLType
    },
    /// Non-specified error with a custom message.
    Other(String),
    /// Unspecified error.
    Panic
}

/// Error that can occur while operating on a collection.
pub enum CollectionError {
    /// The requested index is bigger than the max collection index.
    IndexOutOfBounds
}

impl From<CollectionError> for ExecutionError {
    fn from(error: CollectionError) -> Self {
        match error {
            CollectionError::IndexOutOfBounds => Self::IndexOutOfBounds
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
            AddressError::ZeroAddress => Self::ZeroAddress,
            AddressError::AddressCreationError => Self::AddressCreationFailed
        }
    }
}

impl From<AddressError> for OdraError {
    fn from(error: AddressError) -> Self {
        Into::<ExecutionError>::into(error).into()
    }
}

/// Event-related errors.
#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub enum EventError {
    /// The type of event is different than expected.
    UnexpectedType(String),
    /// Index of the event is out of bounds.
    IndexOutOfBounds,
    /// Formatting error while deserializing.
    Formatting,
    /// Unexpected error while deserializing.
    Parsing,
    /// Could not extract event name.
    CouldntExtractName,
    /// Could not extract event data.
    CouldntExtractEventData,
    /// Contract doesn't support CES events.
    ContractDoesntSupportEvents
}

/// Represents the result of a contract call.
pub type OdraResult<T> = Result<T, OdraError>;

impl From<CLValueError> for OdraError {
    fn from(error: CLValueError) -> Self {
        match error {
            CLValueError::Serialization(_) => OdraError::VmError(Serialization),
            CLValueError::Type(cl_type_mismatch) => OdraError::VmError(VmError::TypeMismatch {
                expected: cl_type_mismatch.expected.clone(),
                found: cl_type_mismatch.found.clone()
            })
        }
    }
}

impl From<BytesReprError> for OdraError {
    fn from(error: BytesReprError) -> Self {
        match error {
            BytesReprError::EarlyEndOfStream => ExecutionError::EarlyEndOfStream,
            BytesReprError::Formatting => ExecutionError::Formatting,
            BytesReprError::LeftOverBytes => ExecutionError::LeftOverBytes,
            BytesReprError::OutOfMemory => ExecutionError::OutOfMemory,
            BytesReprError::NotRepresentable => ExecutionError::NotRepresentable,
            BytesReprError::ExceededRecursionDepth => ExecutionError::ExceededRecursionDepth,
            _ => ExecutionError::Formatting
        }
        .into()
    }
}
