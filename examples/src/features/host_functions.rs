//! This example demonstrates how to use host functions in a contract.
use odra::prelude::*;

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
}

#[cfg(test)]
mod tests {
    use odra::{host::Deployer, prelude::string::ToString};

    use crate::features::host_functions::{HostContract, HostContractInitArgs};

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
}
