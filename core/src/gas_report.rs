//! Module with structs for handling gas reports.
use crate::prelude::*;
use crate::{Address, CallDef};
use casper_types::U512;
use core::fmt::{Debug, Display, Formatter, Result};
use serde::{Deserialize, Serialize};

/// A Vector of deploy reports makes a full gas report.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GasReport(Vec<DeployReport>);

impl GasReport {
    /// Adds a deploy report to the gas report.
    pub fn push(&mut self, report: DeployReport) {
        self.0.push(report)
    }

    /// Returns new, empty gas report.
    pub fn new() -> Self {
        GasReport::default()
    }

    /// Returns an iterator over the gas report.
    pub fn iter(&self) -> Iter<'_, DeployReport> {
        self.0.iter()
    }
}

impl Display for GasReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        for report in &self.0 {
            writeln!(f, "{}", report)?;
        }
        Ok(())
    }
}

impl IntoIterator for GasReport {
    type Item = DeployReport;
    type IntoIter = IntoIter<DeployReport>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Represents a deploy report, which includes the gas used and the deploy details.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DeployReport {
    /// Represents a Wasm deploy.
    WasmDeploy {
        /// The gas used for the deploy.
        gas: U512,
        /// The file name of the deployed WASM.
        file_name: String
    },
    /// Represents a contract call.
    ContractCall {
        /// The gas used for the call.
        gas: U512,
        /// The address of the contract called.
        contract_address: Address,
        /// The call definition.
        call_def: CallDef
    }
}

impl Display for DeployReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            DeployReport::WasmDeploy { gas, file_name } => {
                write!(
                    f,
                    "Wasm deploy: {} - {} CSPR",
                    file_name,
                    cspr_pretty_string(gas)
                )
            }
            DeployReport::ContractCall { gas, call_def, .. } => {
                write!(
                    f,
                    "Contract call: {} - {} CSPR",
                    call_def.entry_point(),
                    cspr_pretty_string(gas)
                )
            }
        }
    }
}

/// Render 1234567890 as 1.23456789
fn cspr_pretty_string(cspr: &U512) -> String {
    let cspr_top = cspr / U512::from(1_000_000_000);
    let cspr_bottom = cspr % U512::from(1_000_000_000); // Use modulo to get the remainder
    let cspr_bottom_str = format!("{:09}", cspr_bottom);
    let mut cspr_bottom_str = cspr_bottom_str.trim_end_matches('0');
    if cspr_bottom_str.is_empty() {
        cspr_bottom_str = "0";
    }
    format!("{}.{}", cspr_top, cspr_bottom_str)
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
        assert_eq!(
            format!("{}", wasm_deploy),
            "Wasm deploy: test.wasm - 0.000001 CSPR"
        );

        let contract_call = DeployReport::ContractCall {
            gas: U512::from(1_000_000_000),
            contract_address: Account(AccountHash([0; 32])),
            call_def: CallDef::new("test", false, RuntimeArgs::new())
        };
        assert_eq!(
            format!("{}", contract_call),
            "Contract call: test - 1.0 CSPR"
        );
    }
}
