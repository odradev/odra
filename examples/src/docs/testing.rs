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

    pub fn created_at(&self) -> BlockTime {
        self.created_at.get().unwrap()
    }

    pub fn created_by(&self) -> Address {
        self.created_by.get().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::MyContractDeployer;

    #[test]
    fn test_env() {
        let my_contract = MyContractDeployer::init("MyContract".to_string());
        let creator = my_contract.created_by();
        odra::test_env::set_caller(odra::test_env::get_account(1));
        let my_contract2 = MyContractDeployer::init("MyContract2".to_string());
        let creator2 = my_contract2.created_by();
        assert!(creator != creator2);
    }
}
