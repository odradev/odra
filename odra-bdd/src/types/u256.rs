use std::ops::Deref;
use std::str::FromStr;

use cucumber::Parameter;
use odra::casper_types::U256;
// use derive_more::FromStr;

// #[derive(Deref, FromStr, Parameter)]
#[derive(Parameter, PartialEq)]
#[param(regex = r"\d+", name = "token_amount")]
pub struct U256Param<const T: usize>(U256);

impl<const T: usize> Deref for U256Param<T> {
    type Target = U256;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const T: usize> U256Param<T> {
    pub fn as_u256(&self) -> U256 {
        self.0
    }
}

impl<const T: usize> From<U256> for U256Param<T> {
    fn from(value: U256) -> Self {
        U256Param(value)
    }
}

impl<const T: usize> std::fmt::Debug for U256Param<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: use T as the number of decimal places
        write!(f, "{}", self.0)
    }
}

impl<const T: usize> FromStr for U256Param<T> {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: use T as the number of decimal places
        let value = U256::from_dec_str(s).map_err(|e| e.to_string())?;
        Ok(U256Param(value))
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_u256_param_from_str() {
        // TODO: implement this test
    }
}
