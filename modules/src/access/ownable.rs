use crate::access::errors::Error::{CallerNotTheOwner, OwnerNotSet};
use crate::access::events::OwnershipTransferred;
use odra::prelude::*;
use odra::{Address, Module, UnwrapOrRevert, Variable};

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
    owner: Variable<Option<Address>>
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
