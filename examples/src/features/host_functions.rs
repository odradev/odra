//! This example demonstrates how to use host functions in a contract.
use odra::{
    casper_types::{bytesrepr::Bytes, U512},
    prelude::*
};

/// Host contract. It shows the Odra's capabilities regarding host functions.
#[odra::module]
pub struct HostContract {
    name: Var<String>,
    created_at: Var<u64>,
    created_by: Var<Address>
}

#[odra::module]
impl HostContract {
    /// Initializes the contract with the given parameters.
    pub fn init(&mut self, name: String) {
        self.name.set(name);
        self.created_at.set(self.env().get_block_time());
        self.created_by.set(self.env().caller())
    }

    /// Returns the contract's name.
    pub fn name(&self) -> String {
        self.name.get_or_default()
    }

    /// Returns pseudorandom bytes of the given size.
    pub fn pseudorandom_bytes(&self, size: u32) -> Bytes {
        self.env().pseudorandom_bytes(size as usize).into()
    }

    /// Returns the pseudorandom number
    pub fn pseudorandom_number(&self, max: U512) -> U512 {
        self.env().pseudorandom_number(max)
    }
}

#[cfg(test)]
mod tests {
    use crate::features::host_functions::{HostContract, HostContractInitArgs};
    use odra::casper_types::U512;
    use odra::{host::Deployer, prelude::string::ToString};

    #[test]
    fn host_test() {
        let test_env = odra_test::env();
        let host_contract = HostContract::deploy(
            &test_env,
            HostContractInitArgs {
                name: "HostContract".to_string()
            }
        );
        assert_eq!(host_contract.name(), "HostContract".to_string());
    }

    #[test]
    fn pseudorandom_test() {
        let test_env = odra_test::env();
        let host_contract = HostContract::deploy(
            &test_env,
            HostContractInitArgs {
                name: "HostContract".to_string()
            }
        );

        let bytes = host_contract.pseudorandom_bytes(129);
        assert_eq!(bytes.len(), 129);

        let number = host_contract.pseudorandom_number(U512::from(255));
        assert!(number < U512::from(255));
    }
}
