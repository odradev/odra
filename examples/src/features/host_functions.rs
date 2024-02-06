use odra::prelude::*;
use odra::{Address, Module, Var};

#[odra::module]
pub struct HostContract {
    name: Var<String>,
    created_at: Var<u64>,
    created_by: Var<Address>
}

#[odra::module]
impl HostContract {
    pub fn init(&mut self, name: String) {
        self.name.set(name);
        self.created_at.set(self.env().get_block_time());
        self.created_by.set(self.env().caller())
    }

    pub fn name(&self) -> String {
        self.name.get_or_default()
    }
}

#[cfg(test)]
mod tests {
    use odra::{host::Deployer, prelude::string::ToString};

    use crate::features::host_functions::{HostContractHostRef, HostContractInitArgs};

    #[test]
    fn host_test() {
        let test_env = odra_test::env();
        let host_contract = HostContractHostRef::deploy(
            &test_env,
            HostContractInitArgs {
                name: "HostContract".to_string()
            }
        );
        assert_eq!(host_contract.name(), "HostContract".to_string());
    }
}
