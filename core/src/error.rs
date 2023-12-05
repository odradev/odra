use crate::arithmetic::ArithmeticsError;
use crate::prelude::*;
use core::any::Any;

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
    pub fn code(&self) -> u16 {
        match self {
            OdraError::ExecutionError(e) => e.code(),
            OdraError::VmError(_e) => 123
        }
    }

    pub fn user(code: u16) -> Self {
        OdraError::ExecutionError(ExecutionError::User(code))
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
    /// Unwrap error
    UnwrapError = 1,
    AdditionOverflow = 100,
    SubtractionOverflow = 101,
    /// Method does not accept deposit
    NonPayable = 102,
    /// Can't transfer tokens to contract.
    TransferToContract = 103,
    ReentrantCall = 104,
    ContractAlreadyInstalled = 105,
    UnknownConstructor = 106,
    NativeTransferError = 107,
    IndexOutOfBounds = 108,
    ZeroAddress = 109,
    AddressCreationFailed = 110,
    EarlyEndOfStream = 111,
    Formatting = 112,
    LeftOverBytes = 113,
    OutOfMemory = 114,
    NotRepresentable = 115,
    ExceededRecursionDepth = 116,
    KeyNotFound = 117,
    CouldNotDeserializeSignature = 118,
    MaxUserError = 32767,
    /// User error too high. The code should be in range 0..32767.
    UserErrorTooHigh = 32768,
    User(u16)
}

impl ExecutionError {
    pub fn code(&self) -> u16 {
        unsafe {
            match self {
                ExecutionError::User(code) => *code,
                ExecutionError::UserErrorTooHigh => 32768,
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
