use crate::access::errors::Error::{CallerNotTheNewOwner, CallerNotTheOwner, OwnerNotSet};
use crate::access::events::{OwnershipTransferStarted, OwnershipTransferred};
use odra::prelude::*;
use odra::{
    module::{Module, SubModule},
    Address, UnwrapOrRevert, Var
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
#[odra::module(events = [OwnershipTransferred])]
pub struct Ownable {
    owner: Var<Option<Address>>
}

#[odra::module]
impl Ownable {
    /// Initializes the module setting the caller as the initial owner.
    #[odra(init)]
    pub fn init(&mut self) {
        let caller = self.env().caller();
        let initial_owner = Some(caller);
        self.unchecked_transfer_ownership(initial_owner);
    }

    /// Transfers ownership of the module to `new_owner`. This function can only
    /// be accessed by the current owner of the module.
    pub fn transfer_ownership(&mut self, new_owner: &Address) {
        self.assert_owner(&self.env().caller());
        self.unchecked_transfer_ownership(Some(*new_owner));
    }

    /// If the contract's owner chooses to renounce their ownership, the contract
    /// will no longer have an owner. This means that any functions that can only
    /// be accessed by the owner will no longer be available.
    ///
    /// The function can only be called by the current owner, and it will permanently
    /// remove the owner's privileges.
    pub fn renounce_ownership(&mut self) {
        self.assert_owner(&self.env().caller());
        self.unchecked_transfer_ownership(None);
    }

    /// Returns the address of the current owner.
    pub fn get_owner(&self) -> Address {
        self.get_optional_owner()
            .unwrap_or_revert_with(&self.env(), OwnerNotSet)
    }

    /// Reverts with [`Error::CallerNotTheOwner`] if the function called by
    /// any account other than the owner.
    pub fn assert_owner(&self, address: &Address) {
        if Some(address) != self.get_optional_owner().as_ref() {
            self.env().revert(CallerNotTheOwner)
        }
    }

    pub fn get_optional_owner(&self) -> Option<Address> {
        self.owner.get().flatten()
    }

    pub fn unchecked_transfer_ownership(&mut self, new_owner: Option<Address>) {
        let previous_owner = self.get_optional_owner();
        self.owner.set(new_owner);

        self.env().emit_event(OwnershipTransferred {
            previous_owner,
            new_owner
        });
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
#[odra::module(events = [OwnershipTransferStarted])]
pub struct Ownable2Step {
    ownable: SubModule<Ownable>,
    pending_owner: Var<Option<Address>>
}

#[odra::module]
impl Ownable2Step {
    /// Initializes the module setting the caller as the initial owner.
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
    pub fn transfer_ownership(&mut self, new_owner: &Address) {
        self.ownable.assert_owner(&self.env().caller());

        let previous_owner = self.ownable.get_optional_owner();
        let new_owner = Some(*new_owner);
        self.pending_owner.set(new_owner);
        self.env().emit_event(OwnershipTransferred {
            previous_owner,
            new_owner
        });
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
        let caller = self.env().caller();
        let caller = Some(caller);
        let pending_owner = self.pending_owner.get().flatten();
        if pending_owner != caller {
            self.env().revert(CallerNotTheNewOwner)
        }
        self.pending_owner.set(None);
        self.ownable.unchecked_transfer_ownership(caller);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::access::errors::Error;
    use odra::{
        external_contract,
        host::{Deployer, HostEnv, HostRef, NoArgs}
    };

    #[test]
    fn init() {
        // given new contacts
        let (env, ownable, ownable_2step, deployer) = setup_owned();

        // then the deployer is the owner
        assert_eq!(deployer, ownable.get_owner());
        assert_eq!(deployer, ownable_2step.get_owner());
        // then a OwnershipTransferred event was emitted

        let event = OwnershipTransferred {
            previous_owner: None,
            new_owner: Some(deployer)
        };

        env.emitted_event(ownable.address(), &event);
        env.emitted_event(ownable_2step.address(), &event);
    }

    #[test]
    fn plain_ownership_transfer() {
        // given a new contract
        let (mut contract, initial_owner) = setup_ownable();

        // when the current owner transfers ownership
        let new_owner = contract.env().get_account(1);
        contract.transfer_ownership(new_owner);

        // then the new owner is set
        assert_eq!(new_owner, contract.get_owner());
        // then a OwnershipTransferred event was emitted
        contract.env().emitted_event(
            contract.address(),
            &OwnershipTransferred {
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
        let new_owner = contract.env().get_account(1);
        contract.transfer_ownership(new_owner);

        // when the pending owner accepts the transfer
        contract.env().set_caller(new_owner);
        contract.accept_ownership();

        // then the new owner is set
        assert_eq!(new_owner, contract.get_owner());
        // then the pending owner is unset
        assert_eq!(None, contract.get_pending_owner());
        // then OwnershipTransferStarted and OwnershipTransferred events were emitted
        contract.env().emitted_event(
            contract.address(),
            &OwnershipTransferStarted {
                previous_owner: Some(initial_owner),
                new_owner: Some(new_owner)
            }
        );
        contract.env().emitted_event(
            contract.address(),
            &OwnershipTransferred {
                previous_owner: Some(initial_owner),
                new_owner: Some(new_owner)
            }
        );
    }

    #[test]
    fn failing_plain_ownership_transfer() {
        // given a new contract
        let (mut contract, _) = setup_ownable();

        // when a non-owner account is the caller
        let (caller, new_owner) = (contract.env().get_account(1), contract.env().get_account(2));
        contract.env().set_caller(caller);

        // then ownership transfer fails
        let err = contract.try_transfer_ownership(new_owner).unwrap_err();
        assert_eq!(err, CallerNotTheOwner.into());
    }

    #[test]
    fn failing_two_step_transfer() {
        // given a new contract
        let (mut contract, initial_owner) = setup_ownable_2_step();

        // when a non-owner account is the caller
        let (caller, new_owner) = (contract.env().get_account(1), contract.env().get_account(2));
        contract.env().set_caller(caller);

        // then ownership transfer fails
        let err = contract.try_transfer_ownership(new_owner).unwrap_err();
        assert_eq!(err, CallerNotTheOwner.into());

        // when the owner is the caller
        contract.env().set_caller(initial_owner);
        contract.transfer_ownership(new_owner);

        // then the pending owner is set
        assert_eq!(contract.get_pending_owner(), Some(new_owner));

        // when someone else than the pending owner accepts the ownership
        // transfer, it should fail
        let err = contract.try_accept_ownership().unwrap_err();
        assert_eq!(err, Error::CallerNotTheNewOwner.into());

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
            .for_each(|contract: &mut RenounceableHostRef| {
                // when the current owner renounce ownership
                contract.renounce_ownership();

                // then an event is emitted
                contract.env().emitted_event(
                    contract.address(),
                    &OwnershipTransferred {
                        previous_owner: Some(initial_owner),
                        new_owner: None
                    }
                );
                // then the owner is not set
                let err = contract.try_get_owner().unwrap_err();
                assert_eq!(err, Error::OwnerNotSet.into());
                // then cannot renounce ownership again
                let err = contract.try_renounce_ownership().unwrap_err();
                assert_eq!(err, Error::CallerNotTheOwner.into());
            });
    }

    #[test]
    fn renounce_ownership_fail() {
        // given new contracts
        let (mut contracts, _) = setup_renounceable();

        contracts.iter_mut().for_each(|contract| {
            // when a non-owner account is the caller
            let caller = contract.env().get_account(1);
            contract.env().set_caller(caller);

            // then renounce ownership fails
            let err = contract.try_renounce_ownership().unwrap_err();
            assert_eq!(err, Error::CallerNotTheOwner.into());
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

    fn setup_ownable() -> (OwnableHostRef, Address) {
        let env = odra_test::env();
        (OwnableHostRef::deploy(&env, NoArgs), env.get_account(0))
    }

    fn setup_ownable_2_step() -> (Ownable2StepHostRef, Address) {
        let env = odra_test::env();
        (
            Ownable2StepHostRef::deploy(&env, NoArgs),
            env.get_account(0)
        )
    }

    fn setup_renounceable() -> (Vec<RenounceableHostRef>, Address) {
        let env = odra_test::env();
        let ownable = OwnableHostRef::deploy(&env, NoArgs);
        let ownable_2_step = Ownable2StepHostRef::deploy(&env, NoArgs);
        let renouncable_ref = RenounceableHostRef::new(*ownable.address(), env.clone());
        let renouncable_2_step_ref =
            RenounceableHostRef::new(*ownable_2_step.address(), env.clone());
        (
            vec![renouncable_ref, renouncable_2_step_ref],
            env.get_account(0)
        )
    }

    fn setup_owned() -> (HostEnv, OwnableHostRef, Ownable2StepHostRef, Address) {
        let env = odra_test::env();
        let ownable = OwnableHostRef::deploy(&env, NoArgs);
        let ownable_2_step = Ownable2StepHostRef::deploy(&env, NoArgs);
        (env.clone(), ownable, ownable_2_step, env.get_account(0))
    }
}
