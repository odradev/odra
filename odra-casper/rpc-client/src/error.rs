use odra_core::{ExecutionError, OdraError, VmError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Livenet generic error")]
    LivenetToDoError,
    #[error("Livenet communication error")]
    RpcCommunicationError,
    #[error("Livenet execution error")]
    ExecutionError { error_message: String }
}

// impl Into<OdraError> for Error {
//     fn into(self) -> OdraError {
//         OdraError::VmError(Other(self.to_string()))
//     }
// }

impl Error {
    pub fn error_message(&self) -> String {
        match self {
            Error::LivenetToDoError => "Livenet generic error".to_string(),
            Error::RpcCommunicationError => "Livenet communication error".to_string(),
            Error::ExecutionError { error_message } => error_message.to_string()
        }
    }
}
