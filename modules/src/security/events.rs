use odra::casper_event_standard::{self, Event};
use odra::Address;

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
