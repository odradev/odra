use odra::Variable;
use odra::types::{BlockTime, Address};

#[odra::module]
pub struct MyContract {
    name: Variable<String>,
    created_at: Variable<BlockTime>,
    created_by: Variable<Address>,
}

#[odra::module]
impl MyContract {
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
    use super::MyContractDeployer;

    #[test]
    fn host_test() {
        let my_contract = MyContractDeployer::init("MyContract".to_string());
        assert_eq!(my_contract.name(), "MyContract".to_string());
    }
}
