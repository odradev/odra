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
    DictQueryError
}

impl LivenetError {
    pub fn error_message(&self) -> String {
        match self {
            LivenetError::RpcCommunicationFailure => "Livenet communication error".to_string(),
            LivenetError::ExecutionError(error_message) => error_message.to_string(),
            LivenetError::RpcRequestError(_, _) => self.to_string(),
            LivenetError::SerializationError => self.to_string(),
            LivenetError::BlockTimeError => self.to_string(),
            LivenetError::ClientError(_) => self.to_string(),
            LivenetError::DictQueryError => self.to_string()
        }
    }
}
