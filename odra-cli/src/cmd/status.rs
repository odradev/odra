use anyhow::Result;
use clap::{ArgMatches, Command};
use odra::{contract_def::HasIdent, host::HostEnv, OdraContract};
use serde_derive::Serialize;

use crate::{
    cmd::{CmdOutput, STATUS_SUBCOMMAND},
    container::ContractProvider,
    custom_types::CustomTypeSet,
    log,
    utils::get_default_contracts_file,
    DeployedContractsContainer
};

use super::OdraCommand;

/// Lists the contracts recorded in the contracts file and cross-references them against the
/// contracts registered in the CLI builder, so the user can see what is deployed at a glance.
#[derive(Default)]
pub(crate) struct StatusCmd {
    registered: Vec<RegisteredContract>
}

/// A contract added to the builder via `.contract::<T>()` / `.named_contract::<T>()`.
struct RegisteredContract {
    ident: String,
    key_name: String
}

impl StatusCmd {
    pub fn add_contract<T: OdraContract>(&mut self) {
        let ident = T::HostRef::ident();
        self.registered.push(RegisteredContract {
            key_name: ident.clone(),
            ident
        });
    }

    pub fn add_contract_named<T: OdraContract>(&mut self, key_name: String) {
        self.registered.push(RegisteredContract {
            ident: T::HostRef::ident(),
            key_name
        });
    }
}

/// The deployment state of the contracts known to the CLI.
#[derive(Serialize)]
pub(crate) struct StatusReport {
    contracts_file: String,
    last_updated: String,
    /// Contracts added to the builder, each flagged with whether it's deployed.
    registered: Vec<ContractStatus>,
    /// Contracts in the file that were never added to the builder (a stale-file symptom).
    unregistered: Vec<UnregisteredContract>
}

impl CmdOutput for StatusReport {
    fn pretty_print(&self) {
        log(format!("Contracts file: {}", self.contracts_file));
        log(format!("Last updated:   {}", self.last_updated));

        if self.registered.is_empty() && self.unregistered.is_empty() {
            log("No contracts registered or deployed yet — run `deploy`.");
            return;
        }

        log("Registered contracts:");
        for c in &self.registered {
            match &c.address {
                Some(address) => log(format!(
                    "  [deployed]     {} ({}) -> {}",
                    c.key_name, c.ident, address
                )),
                None => log(format!("  [not deployed] {} ({})", c.key_name, c.ident))
            }
        }

        if !self.unregistered.is_empty() {
            prettycli::warn("Deployed but not registered in the builder:");
            for d in &self.unregistered {
                prettycli::warn(&format!("  {} ({}) -> {}", d.key_name, d.name, d.address));
            }
        }
    }
}

#[derive(Serialize)]
struct ContractStatus {
    key_name: String,
    ident: String,
    deployed: bool,
    address: Option<String>
}

#[derive(Serialize)]
struct UnregisteredContract {
    key_name: String,
    name: String,
    address: String
}

impl StatusCmd {
    /// Builds the deployment-state report by cross-referencing the registered contracts against the
    /// deployed-contracts container.
    fn report(&self, container: &DeployedContractsContainer) -> StatusReport {
        let deployed = container.all_contracts();

        let registered = self
            .registered
            .iter()
            .map(|reg| {
                let address = deployed
                    .iter()
                    .find(|d| d.key_name() == reg.key_name)
                    .map(|d| d.address().to_string());
                ContractStatus {
                    key_name: reg.key_name.clone(),
                    ident: reg.ident.clone(),
                    deployed: address.is_some(),
                    address
                }
            })
            .collect();

        // Recorded in the file but never added to the builder. Such contracts cannot be called and
        // normally abort startup; surfacing them here helps diagnose a stale contracts file.
        let unregistered = deployed
            .iter()
            .filter(|d| !self.registered.iter().any(|r| r.key_name == d.key_name()))
            .map(|d| UnregisteredContract {
                key_name: d.key_name(),
                name: d.name(),
                address: d.address().to_string()
            })
            .collect();

        StatusReport {
            contracts_file: get_default_contracts_file(),
            last_updated: container.last_updated(),
            registered,
            unregistered
        }
    }
}

impl OdraCommand for StatusCmd {
    type Output = StatusReport;

    fn exec(
        &self,
        _env: &HostEnv,
        _args: &ArgMatches,
        _types: &CustomTypeSet,
        container: &DeployedContractsContainer
    ) -> Result<Self::Output> {
        Ok(self.report(container))
    }
}

impl From<&StatusCmd> for Command {
    fn from(_value: &StatusCmd) -> Self {
        Command::new(STATUS_SUBCOMMAND)
            .about("Lists deployed contracts and their registration status")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{self, TestContract};

    #[test]
    fn builds_status_command() {
        let cmd = StatusCmd::default();
        let clap_cmd: Command = (&cmd).into();
        assert_eq!(clap_cmd.get_name(), STATUS_SUBCOMMAND);
    }

    #[test]
    fn registers_contracts() {
        let mut cmd = StatusCmd::default();
        cmd.add_contract::<TestContract>();
        assert_eq!(cmd.registered.len(), 1);
        assert_eq!(cmd.registered[0].key_name, "TestContract");
        assert_eq!(cmd.registered[0].ident, "TestContract");
    }

    #[test]
    fn report_marks_registered_contract_not_deployed() {
        let mut cmd = StatusCmd::default();
        cmd.add_contract::<TestContract>();

        let report = cmd.report(&test_utils::mock_contracts_container());
        assert_eq!(report.registered.len(), 1);
        assert!(!report.registered[0].deployed);
        assert!(report.registered[0].address.is_none());
        assert!(report.unregistered.is_empty());
    }

    #[test]
    fn runs_against_empty_container() {
        let mut cmd = StatusCmd::default();
        cmd.add_contract::<TestContract>();

        let env = test_utils::mock_host_env();
        let container = test_utils::mock_contracts_container();
        let result = cmd.run(
            &env,
            &ArgMatches::default(),
            &CustomTypeSet::new(),
            &container
        );
        assert!(result.is_ok());
    }
}
