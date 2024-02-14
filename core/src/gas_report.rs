use crate::prelude::*;
use crate::{Address, CallDef};
use casper_types::U512;
use core::fmt::{Debug, Display, Formatter, Result};
pub type GasReport = Vec<DeployReport>;

#[cfg_attr(
    not(target_arch = "wasm32"),
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Clone, Debug)]
pub enum DeployReport {
    WasmDeploy {
        gas: U512,
        file_name: String
    },
    ContractCall {
        gas: U512,
        contract_address: Address,
        call_def: CallDef
    }
}

impl Display for DeployReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            DeployReport::WasmDeploy { gas: _, file_name } => {
                write!(f, "Wasm deploy: {}", file_name)
            }
            DeployReport::ContractCall {
                gas: _,
                contract_address: _,
                call_def
            } => {
                write!(f, "Contract call: {}", call_def.entry_point(),)
            }
        }
    }
}
