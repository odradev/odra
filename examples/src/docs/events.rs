use odra::{Event, contract_env};
use odra::types::{Address, BlockTime, event::OdraEvent};

#[odra::module]
pub struct PartyContract {
}

#[derive(Event, PartialEq, Eq, Debug)]
pub struct PartyStarted {
    pub caller: Address,
    pub block_time: BlockTime,
}

#[odra::module]
impl PartyContract {
    #[odra(init)]
    pub fn init(&self) {
        PartyStarted {
            caller: contract_env::caller(),
            block_time: contract_env::get_block_time(),
        }.emit();
    }
}

#[cfg(test)]
mod tests {
    use odra::{assert_events, test_env};
    use crate::docs::events::PartyStarted;
    use super::PartyContractDeployer;

    #[test]
    fn test_party() {
        let party_contract = PartyContractDeployer::init();
        assert_events!(
            party_contract,
            PartyStarted {
                caller: test_env::get_account(0),
                block_time: 0,
            }
        );
    }
}
