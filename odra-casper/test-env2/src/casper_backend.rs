use casper_contract::contract_api::runtime;
use casper_contract::contract_api::system::{create_purse, transfer_from_purse_to_account};
use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use odra::contract_env::revert;
use odra_core::{CallDef, ContractContext};
use odra_types::casper_types::BlockTime;
use odra_types::{Address, EventData, ExecutionError, U512};

use odra_casper_shared::consts::*;

pub struct CasperBackend;

impl ContractContext for CasperBackend {
    fn get(&self, key: Vec<u8>) -> Option<Vec<u8>> {
        todo!()
    }

    fn set(&mut self, key: Vec<u8>, value: Vec<u8>) {
        todo!()
    }

    fn get_caller(&self) -> Address {
        todo!()
    }

    fn call_contract(&mut self, address: Address, call_def: CallDef) -> Vec<u8> {
        todo!()
    }

    fn get_block_time(&self) -> BlockTime {
        todo!()
    }

    fn callee(&self) -> Address {
        todo!()
    }

    fn attached_value(&self) -> Option<U512> {
        todo!()
    }

    fn emit_event(&mut self, event: EventData) {
        casper_event_standard::emit(event)
    }

    fn transfer_tokens(&mut self, from: &Address, to: &Address, amount: U512) {
        let main_purse = match runtime::get_key(CONTRACT_MAIN_PURSE)
            .map(|key| *key.as_uref().unwrap_or_revert())
        {
            Some(purse) => purse,
            None => {
                let purse = create_purse();
                runtime::put_key(CONTRACT_MAIN_PURSE, purse.into());
                purse
            }
        };

        match to {
            Address::Account(account) => {
                transfer_from_purse_to_account(main_purse, *account, amount.into(), None)
                    .unwrap_or_revert();
            }
            Address::Contract(_) => revert(ExecutionError::can_not_transfer_to_contract())
        };
    }

    fn balance_of(&self, address: &Address) -> U512 {
        todo!()
    }
}
