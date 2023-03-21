use odra::{
    contract_env,
    types::{event::OdraEvent, Address},
    UnwrapOrRevert, Variable
};

use super::{
    errors::Error,
    events::{OwnershipTransferStarted, OwnershipTransferred}
};

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

    /// Transfers ownership of the module to `new_owner`. This function can only
    /// be accessed by the current owner of the module.
    pub fn transfer_ownership(&mut self, new_owner: Address) {
        self.assert_owner(contract_env::caller());
        self.unchecked_transfer_ownership(Some(new_owner));
    }

    /// If the contract's owner chooses to renounce their ownership, the contract
    /// will no longer have an owner. This means that any functions that can only
    /// be accessed by the owner will no longer be available.
    ///
    /// The function can only be called by the current owner, and it will permanently
    /// remove the owner's privileges.
    pub fn renounce_ownership(&mut self) {
        self.assert_owner(contract_env::caller());
        self.unchecked_transfer_ownership(None);
    }

    /// Returns the address of the current owner.
    pub fn get_owner(&self) -> Address {
        self.get_optional_owner()
            .unwrap_or_revert_with(Error::OwnerNotSet)
    }
}

impl Ownable {
    /// Reverts with [`Error::CallerNotTheOwner`] if the function called by
    /// any account other than the owner.
    pub fn assert_owner(&self, address: Address) {
        if Some(address) != self.get_optional_owner() {
            contract_env::revert(Error::CallerNotTheOwner)
        }
    }

    fn get_optional_owner(&self) -> Option<Address> {
        self.owner.get().flatten()
    }

    fn unchecked_transfer_ownership(&mut self, new_owner: Option<Address>) {
        let previous_owner = self.get_optional_owner();
        self.owner.set(new_owner);

        OwnershipTransferred {
            previous_owner,
            new_owner
        }
        .emit();
    }
}

/// This module provides a straightforward access control feature that enables
/// exclusive access to particular functions by an account, known as the owner.
/// The account that initiates contract deployment is automatically assigned as
/// the owner. However, ownership can be transferred later by using the
/// `transfer_ownership()` and `accept_ownership()` functions.
///
/// You can use this module as a standalone contract or integrate it into
/// a custom module by adding it as a field.
///
/// When used in a custom module, the `only_owner()` function is available,
/// allowing you to restrict function usage to the owner.
#[odra::module]
pub struct Ownable2Step {
    ownable: Ownable,
    pending_owner: Variable<Option<Address>>
}

#[odra::module]
impl Ownable2Step {
    /// Initializes the module setting the caller as the initial owner.
    #[odra(init)]
    pub fn init(&mut self) {
        self.ownable.init();
    }

    /// Returns the address of the current owner.
    pub fn get_owner(&self) -> Address {
        self.ownable.get_owner()
    }

    /// Returns the address of the pending owner.
    pub fn get_pending_owner(&self) -> Option<Address> {
        self.pending_owner.get().flatten()
    }

    /// Starts the ownership transfer of the module to a `new_owner`.
    /// Replaces the `pending_owner`if there is one.
    ///
    /// This function can only be accessed by the current owner of the module.
    pub fn transfer_ownership(&mut self, new_owner: Address) {
        self.ownable.assert_owner(contract_env::caller());

        let previous_owner = self.ownable.get_optional_owner();
        let new_owner = Some(new_owner);
        self.pending_owner.set(new_owner);

        OwnershipTransferStarted {
            previous_owner,
            new_owner
        }
        .emit();
    }

    /// If the contract's owner chooses to renounce their ownership, the contract
    /// will no longer have an owner. This means that any functions that can only
    /// be accessed by the owner will no longer be available.
    ///
    /// The function can only be called by the current owner, and it will permanently
    /// remove the owner's privileges.
    pub fn renounce_ownership(&mut self) {
        self.ownable.renounce_ownership()
    }

    /// The new owner accepts the ownership transfer. Replaces the current owner and clears
    /// the pending owner.
    pub fn accept_ownership(&mut self) {
        let caller = Some(contract_env::caller());
        let pending_owner = self.pending_owner.get().flatten();
        if pending_owner != caller {
            contract_env::revert(Error::CallerNotTheNewOwner)
        }
        self.pending_owner.set(None);
        self.ownable.unchecked_transfer_ownership(caller);
    }
}

#[cfg(test)]
mod test {
    use odra::{assert_events, external_contract, test_env};

    use super::*;

    #[test]
    fn init() {
        // given new contacts
        let (contracts, deployer) = setup_owned();

        contracts.iter().for_each(|contract: &OwnedRef| {
            // then the deployer is the owner
            assert_eq!(deployer, contract.get_owner());
            // then a OwnershipTransferred event was emitted
            assert_events!(
                contract,
                OwnershipTransferred {
                    previous_owner: None,
                    new_owner: Some(deployer)
                }
            );
        });
    }

    #[test]
    fn plain_ownership_transfer() {
        // given a new contract
        let (mut contract, initial_owner) = setup_ownable();

        // when the current owner transfers ownership
        let new_owner = test_env::get_account(1);
        contract.transfer_ownership(new_owner);

        // then the new owner is set
        assert_eq!(new_owner, contract.get_owner());
        // then a OwnershipTransferred event was emitted
        assert_events!(
            contract,
            OwnershipTransferred {
                previous_owner: Some(initial_owner),
                new_owner: Some(new_owner)
            }
        );
    }

    #[test]
    fn two_step_ownership_transfer() {
        // given a new contract
        let (mut contract, initial_owner) = setup_ownable_2_step();

        // when the current owner transfers ownership
        let new_owner = test_env::get_account(1);
        contract.transfer_ownership(new_owner);

        // when the pending owner accepts the transfer
        test_env::set_caller(new_owner);
        contract.accept_ownership();

        // then the new owner is set
        assert_eq!(new_owner, contract.get_owner());
        // then the pending owner is unset
        assert_eq!(None, contract.get_pending_owner());
        // then OwnershipTransferStarted and OwnershipTransferred events were emitted
        assert_events!(
            contract,
            OwnershipTransferStarted {
                previous_owner: Some(initial_owner),
                new_owner: Some(new_owner)
            },
            OwnershipTransferred {
                previous_owner: Some(initial_owner),
                new_owner: Some(new_owner)
            }
        );
    }

    #[test]
    fn failing_plain_ownership_transfer() {
        // given a new contract
        let (contract, _) = setup_ownable();

        // when a non-owner account is the caller
        let (caller, new_owner) = (test_env::get_account(1), test_env::get_account(2));
        test_env::set_caller(caller);

        // then ownership transfer fails
        test_env::assert_exception(Error::CallerNotTheOwner, || {
            OwnableRef::at(contract.address()).transfer_ownership(new_owner);
        });
    }

    #[test]
    fn failing_two_step_transfer() {
        // given a new contract
        let (mut contract, initial_owner) = setup_ownable_2_step();

        // when a non-owner account is the caller
        let (caller, new_owner) = (test_env::get_account(1), test_env::get_account(2));
        test_env::set_caller(caller);

        // then ownership transfer fails
        test_env::assert_exception(Error::CallerNotTheOwner, || {
            Ownable2StepRef::at(contract.address()).transfer_ownership(new_owner);
        });

        // when the owner is the caller
        test_env::set_caller(initial_owner);
        contract.transfer_ownership(new_owner);

        // then the pending owner is set
        assert_eq!(contract.get_pending_owner(), Some(new_owner));

        // when someone else than the pending owner accepts the ownership
        // transfer, it should fail
        test_env::assert_exception(Error::CallerNotTheNewOwner, || {
            Ownable2StepRef::at(contract.address()).accept_ownership();
        });

        // then the owner remain the same
        assert_eq!(contract.get_owner(), initial_owner);
        // then the pending owner remain the same
        assert_eq!(contract.get_pending_owner(), Some(new_owner));
    }

    #[test]
    fn renounce_ownership() {
        // given new contracts
        let (mut contracts, initial_owner) = setup_renounceable();

        contracts
            .iter_mut()
            .for_each(|contract: &mut RenounceableRef| {
                // when the current owner renounce ownership
                contract.renounce_ownership();

                // then an event is emitted
                assert_events!(
                    contract,
                    OwnershipTransferred {
                        previous_owner: Some(initial_owner),
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
            });
    }

    #[test]
    fn renounce_ownership_fail() {
        // given new contracts
        let (contracts, _) = setup_renounceable();

        contracts.iter().for_each(|contract| {
            // when a non-owner account is the caller
            let caller = test_env::get_account(1);
            test_env::set_caller(caller);

            // then renounce ownership fails
            test_env::assert_exception(Error::CallerNotTheOwner, || {
                Ownable2StepRef::at(contract.address()).renounce_ownership();
            });
        });
    }

    #[external_contract]
    trait Owned {
        fn get_owner(&self) -> Address;
    }

    #[external_contract]
    trait Renounceable {
        fn renounce_ownership(&mut self);
        fn get_owner(&self) -> Address;
    }

    fn setup_ownable() -> (OwnableRef, Address) {
        (OwnableDeployer::init(), test_env::get_account(0))
    }

    fn setup_ownable_2_step() -> (Ownable2StepRef, Address) {
        (Ownable2StepDeployer::init(), test_env::get_account(0))
    }

    fn setup_renounceable() -> (Vec<RenounceableRef>, Address) {
        let ownable = OwnableDeployer::init();
        let ownable_2_step = Ownable2StepDeployer::init();
        (
            vec![
                RenounceableRef::at(ownable.address()),
                RenounceableRef::at(ownable_2_step.address()),
            ],
            test_env::get_account(0)
        )
    }

    fn setup_owned() -> (Vec<OwnedRef>, Address) {
        let ownable = OwnableDeployer::init();
        let ownable_2_step = Ownable2StepDeployer::init();
        (
            vec![
                OwnedRef::at(ownable.address()),
                OwnedRef::at(ownable_2_step.address()),
            ],
            test_env::get_account(0)
        )
    }
}
