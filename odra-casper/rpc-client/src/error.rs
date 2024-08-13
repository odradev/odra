use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Livenet generic error")]
    LivenetToDo,
    #[error("Livenet communication error")]
    RpcCommunicationFailure,
    #[error("Livenet execution error")]
    Execution { error_message: String }
}

impl Error {
    pub fn error_message(&self) -> String {
        match self {
            Error::LivenetToDo => "Livenet generic error".to_string(),
            Error::RpcCommunicationFailure => "Livenet communication error".to_string(),
            Error::Execution { error_message } => error_message.to_string()
        }
    }
}
