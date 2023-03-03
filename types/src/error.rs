use crate::arithmetic::ArithmeticsError;

const MAX_USER_ERROR: u16 = 32767;
const USER_ERROR_TOO_HIGH: u16 = 32768;
const UNWRAP_ERROR: u16 = u16::MAX;

// const CODE_EARLY_END_OF_STREAM: u16 = 1;
// const CODE_FORMATTING: u16 = 2;
// const CODE_LEFT_OVER_BYTES: u16 = 3;
// const CODE_OUT_OF_MEMORY: u16 = 4;
// const CODE_EXCEEDED_RECURSION_DEPTH: u16 = 5;
// const CODE_TYPE_MISMATCH: u16 = 6;
const CODE_ADDITION_OVERFLOW: u16 = 7;
const CODE_SUBTRACTION_OVERFLOW: u16 = 8;
const CODE_NON_PAYABLE: u16 = 9;
const CODE_TRANSFER_TO_CONTRACT: u16 = 10;

/// General error type in Odra framework
#[derive(Clone, Debug, PartialEq)]
pub enum OdraError {
    /// An error that can occur during smart contract execution
    ExecutionError(ExecutionError),
    /// An internal virtual machine error
    VmError(VmError)
}

impl From<ArithmeticsError> for ExecutionError {
    fn from(error: ArithmeticsError) -> Self {
        match error {
            ArithmeticsError::AdditionOverflow => {
                Self::internal(CODE_ADDITION_OVERFLOW, "Addition overflow")
            }
            ArithmeticsError::SubtractingOverflow => {
                Self::internal(CODE_SUBTRACTION_OVERFLOW, "Subtracting overflow")
            }
        }
    }
}

impl From<ArithmeticsError> for OdraError {
    fn from(error: ArithmeticsError) -> Self {
        Into::<ExecutionError>::into(error).into()
    }
}

impl From<Box<dyn std::any::Any + Send>> for OdraError {
    fn from(_: Box<dyn std::any::Any + Send>) -> Self {
        OdraError::VmError(VmError::Panic)
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

    /// Creates a specific type of error, meaning that value unwrapping failed.
    pub fn unwrap_error() -> Self {
        Self::new(UNWRAP_ERROR, "Unwrap error")
    }

    /// Return the underlying error code
    pub fn code(&self) -> u16 {
        self.0
    }

    pub fn non_payable() -> Self {
        Self::internal(CODE_NON_PAYABLE, "Method does not accept deposit")
    }

    pub fn can_not_transfer_to_contract() -> Self {
        Self::internal(
            CODE_TRANSFER_TO_CONTRACT,
            "Can't transfer tokens to contract."
        )
    }

    fn internal(code: u16, msg: &str) -> Self {
        ExecutionError(code + USER_ERROR_TOO_HIGH, String::from(msg))
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
            CollectionError::IndexOutOfBounds => Self::internal(9, "Index out of bounds")
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
            AddressError::ZeroAddress => Self::internal(10, "Zero address"),
            AddressError::AddressCreationError => Self::internal(11, "Address creation error")
        }
    }
}

impl From<AddressError> for OdraError {
    fn from(error: AddressError) -> Self {
        Into::<ExecutionError>::into(error).into()
    }
}
