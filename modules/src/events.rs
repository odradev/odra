use odra::{types::Address, Event};

#[derive(Event, PartialEq, Eq, Debug)]
pub struct OwnershipTransferred {
    pub previous_owner: Option<Address>,
    pub new_owner: Option<Address>
}

#[derive(Event, PartialEq, Eq, Debug)]
pub struct OwnershipTransferStarted {
    pub previous_owner: Option<Address>,
    pub new_owner: Option<Address>
}
