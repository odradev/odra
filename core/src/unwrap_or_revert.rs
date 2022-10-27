use crate::contract_env;
use odra_types::ExecutionError;

/// A trait that allows safe unwrapping in the context of a smart contract.
/// On failure the contract does not panic, but reverts calling [`contract_env::revert`](crate::contract_env).
/// Works with `Result` and `Option`.
pub trait UnwrapOrRevert<T> {
    /// On success, unwraps the value into its inner type,
    /// on failure, calls [`contract_env::revert`](crate::contract_env) with the passed error.
    fn unwrap_or_revert_with<E: Into<ExecutionError>>(self, err: E) -> T;
    /// On success, unwraps the value into its inner type,
    /// on failure, calls [`contract_env::revert`](crate::contract_env) with the default error.
    fn unwrap_or_revert(self) -> T;
}

impl<T, E: Into<ExecutionError>> UnwrapOrRevert<T> for Result<T, E> {
    fn unwrap_or_revert_with<F: Into<ExecutionError>>(self, err: F) -> T {
        self.unwrap_or_else(|_| contract_env::revert(err.into()))
    }

    fn unwrap_or_revert(self) -> T {
        self.unwrap_or_else(|err| contract_env::revert(err.into()))
    }
}

impl<T> UnwrapOrRevert<T> for Option<T> {
    fn unwrap_or_revert_with<E: Into<ExecutionError>>(self, err: E) -> T {
        self.unwrap_or_else(|| contract_env::revert(err.into()))
    }

    fn unwrap_or_revert(self) -> T {
        self.unwrap_or_else(|| contract_env::revert(ExecutionError::unwrap_error()))
    }
}
