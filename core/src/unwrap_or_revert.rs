use crate::{ContractEnv, ExecutionError, OdraError};

/// A trait that allows safe unwrapping in the context of a smart contract.
/// On failure the contract does not panic, but reverts calling [`ContractEnv::revert`](crate::ContractEnv::revert()).
/// Works with `Result` and `Option`.
pub trait UnwrapOrRevert<T> {
    /// On success, unwraps the value into its inner type,
    /// on failure, calls [`ContractEnv::revert`](crate::ContractEnv::revert()) with the passed error.
    fn unwrap_or_revert_with<E: Into<OdraError>>(self, env: &ContractEnv, err: E) -> T;
    /// On success, unwraps the value into its inner type,
    /// on failure, calls [`ContractEnv::revert`](crate::ContractEnv::revert()) with the default error.
    fn unwrap_or_revert(self, env: &ContractEnv) -> T;
}

impl<T, E: Into<OdraError>> UnwrapOrRevert<T> for Result<T, E> {
    fn unwrap_or_revert_with<F: Into<OdraError>>(self, env: &ContractEnv, err: F) -> T {
        self.unwrap_or_else(|_| env.revert(err))
    }

    fn unwrap_or_revert(self, env: &ContractEnv) -> T {
        self.unwrap_or_else(|err| env.revert(err))
    }
}

impl<T> UnwrapOrRevert<T> for Option<T> {
    fn unwrap_or_revert_with<E: Into<OdraError>>(self, env: &ContractEnv, err: E) -> T {
        self.unwrap_or_else(|| env.revert(err))
    }

    fn unwrap_or_revert(self, env: &ContractEnv) -> T {
        self.unwrap_or_else(|| env.revert(ExecutionError::UnwrapError))
    }
}
