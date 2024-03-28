//! Security module events implementation.
use odra::prelude::*;
use odra::Address;

/// Informs the contract has been stopped by `account`.

#[odra::event]
pub struct Paused {
    /// The account that stopped the contract.
    pub account: Address
}

/// Informs the contract has been unstopped by `account`.
#[odra::event]
pub struct Unpaused {
    /// The account that unstopped the contract.
    pub account: Address
}
