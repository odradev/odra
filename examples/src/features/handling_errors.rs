use odra::prelude::*;
use odra::{Address, Module, OdraError, Variable};

#[odra::module]
pub struct OwnedContract {
    name: Variable<String>,
    owner: Variable<Address>
}

#[derive(OdraError)]
pub enum Error {
    OwnerNotSet = 1,
    NotAnOwner = 2
}

#[odra::module]
impl OwnedContract {
    pub fn init(&mut self, name: String) {
        self.name.set(name);
        self.owner.set(self.env().caller())
    }

    pub fn name(&self) -> String {
        self.name.get_or_default()
    }

    pub fn owner(&self) -> Address {
        self.owner.get_or_revert_with(Error::OwnerNotSet)
    }

    pub fn change_name(&mut self, name: String) {
        let caller = self.env().caller();
        if caller != self.owner() {
            self.env().revert(Error::NotAnOwner)
        }

        self.name.set(name);
    }
}

#[cfg(test)]
mod tests {
    use super::Error;
    use super::OwnedContractDeployer;
    use odra::prelude::*;

    #[test]
    fn test_owner() {
        let test_env = odra_test::test_env();
        let owner = test_env.get_account(0);
        let not_an_owner = test_env.get_account(1);

        test_env.set_caller(owner);
        let mut owned_contract =
            OwnedContractDeployer::init(&test_env, "OwnedContract".to_string());

        test_env.set_caller(not_an_owner);
        assert_eq!(
            owned_contract
                .try_change_name("NewName".to_string())
                .unwrap_err(),
            Error::NotAnOwner.into()
        );
        assert_ne!(owned_contract.name(), "NewName");
    }

    #[test]
    fn test_owner_error() {
        let test_env = odra_test::test_env();
        let owner = test_env.get_account(0);
        let not_an_owner = test_env.get_account(1);

        test_env.set_caller(owner);
        let mut owned_contract =
            OwnedContractDeployer::init(&test_env, "OwnedContract".to_string());

        test_env.set_caller(not_an_owner);
        assert_eq!(
            owned_contract
                .try_change_name("NewName".to_string())
                .unwrap_err(),
            Error::NotAnOwner.into()
        );
    }
}
