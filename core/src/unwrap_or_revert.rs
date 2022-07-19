use crate::ContractEnv;
use odra_types::OdraError;

pub trait UnwrapOrRevert<T> {
    fn unwrap_or_revert_with<E: Into<OdraError>>(self, err: E) -> T;

    fn unwrap_or_revert(self) -> T;
}

impl<T, E: Into<OdraError>> UnwrapOrRevert<T> for Result<T, E> {
    fn unwrap_or_revert_with<F: Into<OdraError>>(self, err: F) -> T {
        self.unwrap_or_else(|_| ContractEnv::revert(err.into()))
    }

    fn unwrap_or_revert(self) -> T {
        self.unwrap_or_else(|err| ContractEnv::revert(err.into()))
    }
}

impl<T> UnwrapOrRevert<T> for Option<T> {
    fn unwrap_or_revert_with<E: Into<OdraError>>(self, err: E) -> T {
        self.unwrap_or_else(|| ContractEnv::revert(err.into()))
    }

    fn unwrap_or_revert(self) -> T {
        self.unwrap_or_else(|| ContractEnv::revert(OdraError::Unknown))
    }
}
