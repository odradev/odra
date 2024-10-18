use crate::prelude::*;

/// A trait that allows safe unwrapping in the context of a smart contract.
/// On failure the contract does not panic, but reverts calling [`ContractEnv::revert`](crate::ContractEnv::revert()).
/// Works with `Result` and `Option`.
pub trait UnwrapOrRevert<T> {
    /// On success, unwraps the value into its inner type,
    /// on failure, calls [`ContractEnv::revert`](crate::ContractEnv::revert()) with the passed error.
    fn unwrap_or_revert_with<E: Into<OdraError>, M: Revertible>(self, rev: &M, err: E) -> T;
    /// On success, unwraps the value into its inner type,
    /// on failure, calls [`ContractEnv::revert`](crate::ContractEnv::revert()) with the default error.
    fn unwrap_or_revert<M: Revertible>(self, rev: &M) -> T;
}

impl<T, E: Into<OdraError>> UnwrapOrRevert<T> for Result<T, E> {
    fn unwrap_or_revert_with<F: Into<OdraError>, M: Revertible>(self, rev: &M, err: F) -> T {
        self.unwrap_or_else(|_| rev.revert(err))
    }

    fn unwrap_or_revert<M: Revertible>(self, rev: &M) -> T {
        self.unwrap_or_else(|err| rev.revert(err))
    }
}

impl<T> UnwrapOrRevert<T> for Option<T> {
    fn unwrap_or_revert_with<E: Into<OdraError>, M: Revertible>(self, rev: &M, err: E) -> T {
        self.unwrap_or_else(|| rev.revert(err))
    }

    fn unwrap_or_revert<M: Revertible>(self, rev: &M) -> T {
        self.unwrap_or_else(|| rev.revert(ExecutionError::UnwrapError))
    }
}
