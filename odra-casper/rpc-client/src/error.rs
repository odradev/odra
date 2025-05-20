use thiserror::Error;

#[derive(Debug, Error)]
pub enum LivenetError {
    #[error("Livenet communication error")]
    RpcCommunicationFailure,
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
            _ => todo!()
        }
    }
}
