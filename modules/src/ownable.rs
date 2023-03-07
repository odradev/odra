use odra::{
    contract_env,
    types::{event::OdraEvent, Address, OdraType},
    UnwrapOrRevert, Variable
};

use crate::{errors::Error, events::OwnershipTransferred};

/// This module provides a straightforward access control feature that enables 
/// exclusive access to particular functions by an account, known as the owner. 
/// The account that initiates contract deployment is automatically assigned as 
/// the owner. However, ownership can be transferred later by using the 
/// `transfer_ownership()` function.
///
/// You can use this module as a standalone contract or integrate it into 
/// a custom module by adding it as a field. 
/// 
/// When used in a custom module, the `only_owner()` function is available, 
/// allowing you to restrict function usage to the owner.
#[odra::module]
pub struct Ownable {
    owner: Variable<Option<Address>>
}

#[odra::module]
impl Ownable {

    /// Initializes the module setting the caller as the initial owner.
    #[odra(init)]
    pub fn init(&mut self) {
        let initial_owner = Some(contract_env::caller());
        self.unchecked_transfer_ownership(initial_owner);
    }

    /// Returns the address of the current owner.
    /// 
    /// # Errors
    /// * [`Error::OwnerNotSet`] - if the current owner is None.
    pub fn get_owner(&self) -> Address {
        self.get_optional_owner()
            .unwrap_or_revert_with(Error::OwnerNotSet)
    }

    /// Transfers ownership of the module to `new_owner`. This function can only 
    /// be accessed by the current owner of the module.
    pub fn transfer_ownership(&mut self, new_owner: Address) {
        let action = |o: &mut Ownable| o.unchecked_transfer_ownership(Some(new_owner));
        self.only_owner(action);
    }

    /// If the contract's owner chooses to renounce their ownership, the contract 
    /// will no longer have an owner. This means that any functions that can only 
    /// be accessed by the owner will no longer be available. 
    /// 
    /// The function can only be called by the current owner, and it will permanently 
    /// remove the owner's privileges.
    pub fn renounce_ownership(&mut self) {
        let action = |o: &mut Ownable| o.unchecked_transfer_ownership(None);
        self.only_owner(action);
    }
}

impl Ownable {

    /// Reverts with [`Error::CallerNotTheOwner`] if the function called by 
    /// any account other than the owner.
    pub fn only_owner<T: OdraType>(&mut self, mut f: impl FnMut(&mut Self) -> T) -> T {
        if Some(contract_env::caller()) != self.get_optional_owner() {
            contract_env::revert(Error::CallerNotTheOwner)
        }

        f(self)
    }

    fn get_optional_owner(&self) -> Option<Address> {
        self.owner.get().flatten()
    }

    fn unchecked_transfer_ownership(&mut self, new_owner: Option<Address>) {
        let old_owner = self.get_optional_owner();
        self.owner.set(new_owner);

        OwnershipTransferred {
            old_owner,
            new_owner
        }
        .emit();
    }
}

struct Ownable2Step {
    ownable: Ownable,
    pending_owner: Option<Address>
}

#[cfg(test)]
mod test {
    use odra::{test_env, assert_events};

    use super::*;

    fn setup() -> (OwnableRef, Address) {
        (OwnableDeployer::init(), test_env::get_account(0))
    }

    #[test]
    fn init() {
        // given a new contact
        let (contract, deployer) = setup();

        // then the deployer is the owner
        assert_eq!(deployer, contract.get_owner());
        // then an event is emitted
        assert_events!(
            contract, 
            OwnershipTransferred { 
                old_owner: None, 
                new_owner: Some(deployer) 
            }
        );
    }

    #[test]
    fn transfer() {
        // given a new contract
        let (mut contract, initial_owner) = setup();

        // when the current owner transfers ownership
        let new_owner = test_env::get_account(1);
        contract.transfer_ownership(new_owner);

        // then the new owner is set
        assert_eq!(new_owner, contract.get_owner());
        // then an event is emitted
        assert_events!(
            contract, 
            OwnershipTransferred { 
                old_owner: Some(initial_owner), 
                new_owner: Some(new_owner) 
            }
        );
    }

    #[test]
    fn transfer_fail() {
        // given a new contract 
        let (contract, _) = setup();
        
        // when a non-owner account is the caller
        let (caller, new_owner) = (test_env::get_account(1), test_env::get_account(2));
        test_env::set_caller(caller);

        // then ownership transfer fails
        test_env::assert_exception(Error::CallerNotTheOwner, || {
            OwnableRef::at(contract.address()).transfer_ownership(new_owner);
        });
    }

    #[test]
    fn renounce_ownership() {
        // given a new contract 
        let (mut contract, initial_owner) = setup();

        // when the current owner renounce ownership
        contract.renounce_ownership();

        // then an event is emitted
        assert_events!(
            contract, 
            OwnershipTransferred { 
                old_owner: Some(initial_owner), 
                new_owner: None 
            }
        );
        // then the owner is not set
        test_env::assert_exception(Error::OwnerNotSet, || {
            contract.get_owner();
        });
        // then cannot renounce ownership again
        test_env::assert_exception(Error::CallerNotTheOwner, || {
            let mut contract = OwnableRef::at(contract.address());
            contract.renounce_ownership();
        });
    }

    #[test]
    fn renounce_ownership_fail() {
        // given a new contract 
        let (contract, _) = setup();
        
        // when a non-owner account is the caller
        let caller = test_env::get_account(1);
        test_env::set_caller(caller);
        
        // then renounce ownership fails
        test_env::assert_exception(Error::CallerNotTheOwner, || {
            OwnableRef::at(contract.address()).renounce_ownership();
        });
    }
}
