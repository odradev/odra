use casper_event_standard::Event;
use odra::casper_event_standard;
use odra::prelude::*;
use odra::{module::Module, Address};

#[odra::module]
pub struct PartyContract {}

#[derive(Event, PartialEq, Eq, Debug)]
pub struct PartyStarted {
    pub caller: Address,
    pub block_time: u64
}

#[odra::module]
impl PartyContract {
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
    use odra::host::{Deployer, EmptyInitArgs, HostRef};

    #[test]
    fn test_party() {
        let test_env = odra_test::env();
        let party_contract = PartyContractHostRef::deploy(&test_env, EmptyInitArgs);
        test_env.emitted_event(
            party_contract.address(),
            &PartyStarted {
                caller: test_env.get_account(0),
                block_time: 0
            }
        );
    }
}
