use crate::security::errors::Error::{PausedRequired, UnpausedRequired};
use crate::security::events::{Paused, Unpaused};
use odra::prelude::*;
use odra::Var;

/// A module allowing to implement an emergency stop mechanism that can be triggered by any account.
///
/// You can use this module in a custom module by adding it as a field.
///
/// It will make available `require_not_paused()` and `require_paused()` functions,
/// which can be used in the functions of your contract to ensure the contract is
/// in the correct state.
#[odra::module]
pub struct Pauseable {
    is_paused: Var<bool>
}

impl Pauseable {
    /// Returns true if the contract is paused, and false otherwise.
    pub fn is_paused(&self) -> bool {
        self.is_paused.get_or_default()
    }

    /// Reverts with `[Error::UnpausedRequired]` if the contract is paused.
    pub fn require_not_paused(&self) {
        if self.is_paused() {
            self.env().revert(UnpausedRequired);
        }
    }

    /// Reverts with `[Error::PausedRequired]` if the contract is paused.
    pub fn require_paused(&self) {
        if !self.is_paused() {
            self.env().revert(PausedRequired);
        }
    }

    /// Changes the state to `stopped`.
    ///
    /// The contract must not be paused.
    ///
    /// Emits Paused event.
    pub fn pause(&mut self) {
        self.require_not_paused();
        self.is_paused.set(true);

        self.env().emit_event(Paused {
            account: self.env().caller()
        });
    }

    /// Returns the contract to normal state.
    ///
    /// The contract must be paused.
    ///
    /// Emits Unpaused event.
    pub fn unpause(&mut self) {
        self.require_paused();
        self.is_paused.set(false);

        self.env().emit_event(Unpaused {
            account: self.env().caller()
        });
    }
}
