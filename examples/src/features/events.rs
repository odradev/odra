//! This examples shows how to handle events in a contract.
use odra::prelude::*;
use odra::{Address, Event};

/// Contract that emits an event when initialized.
#[odra::module(events = [PartyStarted])]
pub struct PartyContract;

/// Event emitted when the contract is initialized.
#[derive(Event, PartialEq, Eq, Debug)]
pub struct PartyStarted {
    /// Address of the caller.
    pub caller: Address,
    /// Block time when the contract was initialized.
    pub block_time: u64
}

#[odra::module]
impl PartyContract {
    /// Initializes the contract.
    pub fn init(&self) {
        self.env().emit_event(PartyStarted {
            caller: self.env().caller(),
            block_time: self.env().get_block_time()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{PartyContractHostRef, PartyStarted};
    use odra::host::{Deployer, HostRef, NoArgs};

    #[test]
    fn test_party() {
        let test_env = odra_test::env();
        let party_contract = PartyContractHostRef::deploy(&test_env, NoArgs);
        test_env.emitted_event(
            party_contract.address(),
            &PartyStarted {
                caller: test_env.get_account(0),
                block_time: 0
            }
        );
    }
}
