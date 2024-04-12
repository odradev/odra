use odra::{casper_types::{bytesrepr::FromBytes, ContractHash}, Address, ContractEnv, OdraError, UnwrapOrRevert, Var};

pub trait GetAs<T> {
    fn get_as(&self, env: &ContractEnv) -> T;
}

impl<R, T> GetAs<T> for Var<R>
where
    R: TryInto<T> + Default + FromBytes,
    R::Error: Into<OdraError>,
{
    fn get_as(&self, env: &ContractEnv) -> T {
        self.get_or_default().try_into().unwrap_or_revert(env)
    }
}

pub trait IntoOrRevert<T> {
    type Error;
    fn into_or_revert(self, env: &ContractEnv) -> T;
}

impl<R, T> IntoOrRevert<T> for R
where
    R: TryInto<T>,
    R::Error: Into<OdraError>,
{
    type Error = R::Error;
    fn into_or_revert(self, env: &ContractEnv) -> T {
        self.try_into().unwrap_or_revert(env)
    }
}

pub fn get_transfer_filter_contract() -> Option<Address> {
    None
}