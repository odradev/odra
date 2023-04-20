use odra::types::{Address, BlockTime};
use odra::Variable;

#[odra::module]
pub struct HostContract {
    name: Variable<String>,
    created_at: Variable<BlockTime>,
    created_by: Variable<Address>
}

#[odra::module]
impl HostContract {
    #[odra(init)]
    pub fn init(&mut self, name: String) {
        self.name.set(name);
        self.created_at.set(odra::contract_env::get_block_time());
        self.created_by.set(odra::contract_env::caller())
    }

    pub fn name(&self) -> String {
        self.name.get_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::HostContractDeployer;

    #[test]
    fn host_test() {
        let host_contract = HostContractDeployer::init("HostContract".to_string());
        assert_eq!(host_contract.name(), "HostContract".to_string());
    }
}
