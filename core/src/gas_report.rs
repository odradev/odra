//! Module with structs for handling gas reports.
use crate::prelude::*;
use crate::{Address, CallDef};
use casper_types::U512;
use core::fmt::{Debug, Display, Formatter, Result};

/// A Vector of deploy reports makes a full gas report.
pub type GasReport = Vec<DeployReport>;

/// Represents a deploy report, which includes the gas used and the deploy details.
#[cfg_attr(
    not(target_arch = "wasm32"),
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Clone, Debug)]
pub enum DeployReport {
    /// Represents a Wasm deploy.
    WasmDeploy { gas: U512, file_name: String },
    /// Represents a contract call.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Address::Account;
    use casper_types::account::AccountHash;
    use casper_types::RuntimeArgs;

    #[test]
    fn test_deploy_report_display() {
        let wasm_deploy = DeployReport::WasmDeploy {
            gas: U512::from(1000),
            file_name: String::from("test.wasm")
        };
        assert_eq!(format!("{}", wasm_deploy), "Wasm deploy: test.wasm");

        let contract_call = DeployReport::ContractCall {
            gas: U512::from(1000),
            contract_address: Account(AccountHash([0; 32])),
            call_def: CallDef::new("test", false, RuntimeArgs::new())
        };
        assert_eq!(format!("{}", contract_call), "Contract call: test");
    }
}
