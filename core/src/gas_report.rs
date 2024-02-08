use crate::prelude::*;
use crate::{Address, CallDef};
use casper_types::U512;
use core::fmt::Debug;
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
