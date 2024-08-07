use odra_core::OdraError;
use odra_core::VmError::Other;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Livenet generic error")]
    LivenetToDoError,
    #[error("Livenet communication error")]
    RpcCommunicationError
}

// impl Into<OdraError> for Error {
//     fn into(self) -> OdraError {
//         OdraError::VmError(Other(self.to_string()))
//     }
// }

impl From<Error> for OdraError {
    fn from(value: Error) -> Self {
        value.into()
    }
}
