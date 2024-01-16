use odra::prelude::*;
use odra::{Address, Module, Variable};

#[odra::module]
pub struct HostContract {
    name: Variable<String>,
    created_at: Variable<u64>,
    created_by: Variable<Address>
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
    use super::HostContractDeployer;
    use odra::prelude::string::ToString;

    #[test]
    fn host_test() {
        let test_env = odra_test::test_env();
        let host_contract = HostContractDeployer::init(&test_env, "HostContract".to_string());
        assert_eq!(host_contract.name(), "HostContract".to_string());
    }
}
