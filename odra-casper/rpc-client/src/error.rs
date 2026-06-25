use odra_core::prelude::Address;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LivenetError {
    #[error("Livenet communication error")]
    RpcCommunicationFailure,
    #[error("RPC request error {0}: {1}")]
    RpcRequestError(String, String),
    #[error("Livenet execution error")]
    ExecutionError(String),
    #[error("Serialization error")]
    SerializationError,
    #[error("Couldn't get block time")]
    BlockTimeError,
    #[error("Casper client error: {0}")]
    ClientError(String),
    #[error("Couldn't query dictionary")]
    DictQueryError,
    #[error("Gas not set")]
    GasNotSet,
    #[error("Invalid target address for transfer: {0:?}")]
    InvalidTransferTarget(Address),
    #[error("Invalid transaction")]
    InvalidTransaction,
    #[error("Environment variable {0} must be set. Have you setup your .env file?")]
    EnvVariableNotSet(String),
    #[error(transparent)]
    SecrectKeyLoadError(#[from] odra_core::casper_types::crypto::ErrorExt)
}

impl LivenetError {
    pub fn error_message(&self) -> String {
        match self {
            LivenetError::RpcCommunicationFailure => "Livenet communication error".to_string(),
            LivenetError::ExecutionError(error_message) => error_message.to_string(),
            _ => self.to_string()
        }
    }
}
