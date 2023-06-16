use odra::types::Address;
use odra::{execution_error, UnwrapOrRevert, Variable};
use alloc::{vec::Vec, string::String};

#[odra::module]
pub struct OwnedContract {
    name: Variable<String>,
    owner: Variable<Address>
}

execution_error! {
    pub enum Error {
        OwnerNotSet => 1,
        NotAnOwner => 2,
    }
}

#[odra::module]
impl OwnedContract {
    #[odra(init)]
    pub fn init(&mut self, name: String) {
        self.name.set(name);
        self.owner.set(odra::contract_env::caller())
    }

    pub fn name(&self) -> String {
        self.name.get_or_default()
    }

    pub fn owner(&self) -> Address {
        self.owner.get().unwrap_or_revert_with(Error::OwnerNotSet)
    }

    pub fn change_name(&mut self, name: String) {
        let caller = odra::contract_env::caller();
        if caller != self.owner() {
            odra::contract_env::revert(Error::NotAnOwner)
        }

        self.name.set(name);
    }
}

#[cfg(test)]
mod tests {
    use super::Error;
    use super::OwnedContractDeployer;

    #[test]
    fn test_owner() {
        let owner = odra::test_env::get_account(0);
        let not_an_owner = odra::test_env::get_account(1);

        odra::test_env::set_caller(owner);
        let mut owned_contract = OwnedContractDeployer::init("OwnedContract".to_string());

        odra::test_env::set_caller(not_an_owner);
        odra::test_env::assert_exception(Error::NotAnOwner, || {
            owned_contract.change_name("NewName".to_string());
        });
        assert_ne!(owned_contract.name(), "NewName");
    }

    #[test]
    fn test_owner_error() {
        let owner = odra::test_env::get_account(0);
        let not_an_owner = odra::test_env::get_account(1);

        odra::test_env::set_caller(owner);
        let mut owned_contract = OwnedContractDeployer::init("OwnedContract".to_string());

        odra::test_env::set_caller(not_an_owner);
        odra::test_env::assert_exception(Error::NotAnOwner, || {
            owned_contract.change_name("NewName".to_string());
        })
    }
}
