//! This examples shows how to handle events in a contract.
use odra::prelude::*;
use Address;

/// Contract that emits an event when initialized.
#[odra::module(events = [PartyStarted])]
pub struct PartyContract;

/// Event emitted when the contract is initialized.
#[odra::event]
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
    use super::{PartyContract, PartyStarted};
    use odra::host::{Deployer, NoArgs};

    #[test]
    fn test_party() {
        let test_env = odra_test::env();
        let party_contract = PartyContract::deploy(&test_env, NoArgs);
        test_env.emitted_event(
            &party_contract,
            &PartyStarted {
                caller: test_env.get_account(0),
                block_time: 0
            }
        );
    }
}
