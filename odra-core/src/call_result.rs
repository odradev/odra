use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use odra_types::{Address, Bytes, OdraError};

#[derive(Debug, Clone)]
pub struct CallResult {
    pub contract_address: Address,
    pub caller: Address,
    pub gas_used: u64,
    pub result: Result<Bytes, OdraError>,
    pub events: BTreeMap<Address, Vec<Bytes>>
}

impl CallResult {
    pub fn get_events(&self, address: &Address) -> Vec<Bytes> {
        self.events.get(address).cloned().unwrap_or_default()
    }

    pub fn get_result(&self) -> Bytes {
        match &self.result {
            Ok(result) => result.clone(),
            Err(error) => {
                panic!("Last call result is an error: {:?}", error);
            }
        }
    }

    pub fn get_error(&self) -> OdraError {
        match &self.result {
            Ok(_) => {
                panic!("Last call result is not an error");
            }
            Err(error) => error.clone()
        }
    }

    pub fn get_caller(&self) -> Address {
        self.caller
    }

    pub fn get_gas_used(&self) -> u64 {
        self.gas_used
    }

    pub fn get_contract_address(&self) -> Address {
        self.contract_address
    }
}
