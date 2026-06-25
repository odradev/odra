use anyhow::Result;
use clap::{ArgMatches, Command};
use odra::{contract_def::HasIdent, host::HostEnv, OdraContract};

use crate::{
    cmd::STATUS_SUBCOMMAND, container::ContractProvider, custom_types::CustomTypeSet,
    utils::get_default_contracts_file, DeployedContractsContainer
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

impl OdraCommand for StatusCmd {
    fn run(
        &self,
        _env: &HostEnv,
        _args: &ArgMatches,
        _types: &CustomTypeSet,
        container: &DeployedContractsContainer
    ) -> Result<()> {
        let deployed = container.all_contracts();

        prettycli::info(&format!("Contracts file: {}", get_default_contracts_file()));
        prettycli::info(&format!("Last updated:   {}", container.last_updated()));

        if self.registered.is_empty() && deployed.is_empty() {
            prettycli::info("No contracts registered or deployed yet — run `deploy`.");
            return Ok(());
        }

        prettycli::info("Registered contracts:");
        for reg in &self.registered {
            match deployed.iter().find(|d| d.key_name() == reg.key_name) {
                Some(d) => prettycli::info(&format!(
                    "  [deployed]     {} ({}) -> {}",
                    reg.key_name,
                    reg.ident,
                    d.address().to_string()
                )),
                None => prettycli::info(&format!(
                    "  [not deployed] {} ({})",
                    reg.key_name, reg.ident
                ))
            }
        }

        // Recorded in the file but never added to the builder. Such contracts cannot be called and
        // normally abort startup; surfacing them here helps diagnose a stale contracts file.
        let unregistered: Vec<_> = deployed
            .iter()
            .filter(|d| !self.registered.iter().any(|r| r.key_name == d.key_name()))
            .collect();
        if !unregistered.is_empty() {
            prettycli::warn("Deployed but not registered in the builder:");
            for d in unregistered {
                prettycli::warn(&format!(
                    "  {} ({}) -> {}",
                    d.key_name(),
                    d.name(),
                    d.address().to_string()
                ));
            }
        }

        Ok(())
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
