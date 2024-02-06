use odra::prelude::*;
use odra::{Address, Event};

/// Informs the contract has been stopped by `account`.

#[derive(Event, PartialEq, Eq, Debug)]
pub struct Paused {
    pub account: Address
}

/// Informs the contract has been unstopped by `account`.
#[derive(Event, PartialEq, Eq, Debug)]
pub struct Unpaused {
    pub account: Address
}
