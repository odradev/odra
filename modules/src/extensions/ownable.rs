use crate::extensions::ownable::errors::Error;
use crate::extensions::ownable::events::OwnershipTransferred;
use odra::contract_env::{caller, revert};
use odra::types::event::OdraEvent;
use odra::types::Address;
use odra::Variable;

/// Trait for the `Ownable` extension.
pub trait Ownable {
    fn owner(&self) -> Option<Address>;
    fn transfer_ownership(&mut self, new_owner: Option<Address>);
    fn renounce_ownership(&mut self);
}

pub mod events {
    use odra::types::Address;

    #[derive(odra::Event)]
    pub struct OwnershipTransferred {
        pub previous_owner: Option<Address>,
        pub new_owner: Option<Address>
    }
}

pub mod errors {
    use odra::execution_error;

    execution_error! {
        pub enum Error {
            DoesNotHaveAnOwner => 20_000,
            NotAnOwner => 20_001,
        }
    }
}

#[odra::module]
pub struct OwnableExtension {
    owner: Variable<Option<Address>>
}

impl Ownable for OwnableExtension {
    /// Returns the address of the current owner.
    fn owner(&self) -> Option<Address> {
        self.owner.get().unwrap_or_default()
    }

    /// Transfers ownership of the contract to a new account (`new_owner`).
    /// Can only be called by the current owner.
    /// Emits an `OwnershipTransferred` event.
    /// Throws if the caller is not the current owner.
    fn transfer_ownership(&mut self, new_owner: Option<Address>) {
        let caller = caller();

        self.assert_owner(caller);
        self.owner.set(new_owner);

        OwnershipTransferred {
            previous_owner: Some(caller),
            new_owner
        }
        .emit();
    }

    /// Leaves the contract without owner.
    /// Can only be called by the current owner.
    /// Emits an `OwnershipTransferred` event.
    /// Throws if the caller is not the current owner.
    fn renounce_ownership(&mut self) {
        self.transfer_ownership(None);
    }
}

impl OwnableExtension {
    pub fn init(&mut self, owner: Option<Address>) {
        self.owner.set(owner);
    }

    pub fn assert_owner(&self, owner: Address) {
        if self.owner() != Some(owner) {
            revert(Error::NotAnOwner)
        }
    }
}
