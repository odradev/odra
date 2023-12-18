use casper_event_standard::Event;
use odra::prelude::*;
use odra::{Address, Module};

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
    use super::{PartyContractDeployer, PartyStarted};

    #[test]
    fn test_party() {
        let test_env = odra::test_env();
        let party_contract = PartyContractDeployer::init(&test_env);
        test_env.emitted_event(
            &party_contract.address,
            &PartyStarted {
                caller: test_env.get_account(0),
                block_time: 0
            }
        );
    }
}
