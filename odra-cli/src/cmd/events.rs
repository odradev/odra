use crate::{
    cmd::{
        args::{read_arg, Arg, ArgsError},
        OdraCommand, PRINT_EVENTS_SUBCOMMAND
    },
    container::{self, ContractProvider},
    custom_types::CustomTypeSet,
    types, DeployedContractsContainer
};
use anyhow::Result;
use clap::{ArgMatches, Command};
use odra::{contract_def::HasIdent, host::HostEnv, OdraContract};

#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error(transparent)]
    ArgsError(#[from] ArgsError),
    #[error(transparent)]
    TypesError(#[from] types::Error),
    #[error("Contract not found")]
    ContractNotFound,
    #[error("Event at {index} not found for contract '{contract_name}'")]
    EventNotFound { index: u32, contract_name: String },
    #[error(transparent)]
    ContractError(#[from] container::ContractError)
}

#[derive(Default)]
pub(crate) struct PrintEventsCmd {
    subcommands: Vec<PrintContractEventsCmd>
}

impl PrintEventsCmd {
    pub fn add_contract<T: OdraContract>(&mut self) {
        self.subcommands
            .push(PrintContractEventsCmd::new::<T>(None));
    }

    pub fn add_contract_named<T: OdraContract>(&mut self, package_name: String) {
        self.subcommands
            .push(PrintContractEventsCmd::new::<T>(Some(package_name)));
    }
}

impl OdraCommand for PrintEventsCmd {
    fn run(
        &self,
        env: &HostEnv,
        args: &ArgMatches,
        types: &CustomTypeSet,
        container: &DeployedContractsContainer
    ) -> Result<()> {
        let (subcmd, args) = args.subcommand().ok_or(EventError::ContractNotFound)?;
        if let Some(cmd) = self.subcommands.iter().find(|c| c.contract_name == subcmd) {
            cmd.run(env, args, types, container)
        } else {
            Err(EventError::ContractNotFound.into())
        }
    }
}

impl From<&PrintEventsCmd> for Command {
    fn from(value: &PrintEventsCmd) -> Self {
        Command::new(PRINT_EVENTS_SUBCOMMAND)
            .about("Prints the most recent events emitted by a contract")
            .arg_required_else_help(true)
            .subcommand_required(true)
            .subcommands(&value.subcommands)
    }
}

struct PrintContractEventsCmd {
    contract_name: String
}

impl PrintContractEventsCmd {
    fn new<T: OdraContract>(package_name: Option<String>) -> Self {
        let contract_name = package_name.unwrap_or_else(T::HostRef::ident);
        Self { contract_name }
    }
}

impl OdraCommand for PrintContractEventsCmd {
    fn run(
        &self,
        env: &HostEnv,
        args: &ArgMatches,
        types: &CustomTypeSet,
        container: &DeployedContractsContainer
    ) -> Result<()> {
        // Ensure the host environment is set up to capture events.
        env.set_captures_events(true);
        let contract_address = container
            .address_by_name(&self.contract_name)
            .ok_or(EventError::ContractNotFound)?;
        // Get the number of events emitted by the contract.
        let events_count = env.events_count(&contract_address);
        // Max number of events to print is read from the arguments, defaulting to 10.
        // If the number exceeds the total number of events, it is capped.
        // If no number is provided, it defaults to the total number of events.
        let max_events = read_arg(args, Arg::EventsNumber)
            .unwrap_or(events_count)
            .min(events_count);

        prettycli::info(&format!(
            "Printing {:?} the most recent events for contract '{}'",
            max_events, self.contract_name
        ));
        for i in 0..max_events {
            // Read events in reverse order, starting from the most recent.
            let idx = events_count - i - 1;
            let ev = env.get_event_bytes(&contract_address, idx).map_err(|_| {
                EventError::EventNotFound {
                    index: i,
                    contract_name: self.contract_name.clone()
                }
            })?;
            // Decode and print the event.
            prettycli::info(&format!(
                "Event {}: {}",
                i + 1,
                types::decode_event(&ev, types)?
            ));
        }

        Ok(())
    }
}

impl From<&PrintContractEventsCmd> for Command {
    fn from(value: &PrintContractEventsCmd) -> Self {
        Command::new(&value.contract_name)
            .about(format!(
                "Print events of the {} contract",
                &value.contract_name
            ))
            .arg(Arg::EventsNumber)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cmd::args::ARG_NUMBER, test_utils::TestContract};

    #[test]
    fn test_print_events_cmd() {
        let mut cmd = PrintEventsCmd::default();
        let command: Command = (&cmd).into();
        assert_eq!(command.get_name(), PRINT_EVENTS_SUBCOMMAND);
        assert_eq!(command.get_subcommands().count(), 0);

        cmd.add_contract::<TestContract>();
        let command: Command = (&cmd).into();
        assert_eq!(command.get_subcommands().count(), 1);
    }

    #[test]
    fn test_match_print_events_cmd() {
        let mut cmd = PrintEventsCmd::default();
        cmd.add_contract::<TestContract>();
        let command: Command = (&cmd).into();
        let matches = command
            .try_get_matches_from(vec![PRINT_EVENTS_SUBCOMMAND, &TestContract::ident()])
            .unwrap();
        let (subcommand, _) = matches.subcommand().expect("Subcommand should be present");
        assert_eq!(subcommand, TestContract::ident());
    }

    #[test]
    fn parsing_print_events_cmd_invalid_contract() {
        // This test checks that an invalid contract name results in an error.
        let mut cmd = PrintEventsCmd::default();
        cmd.add_contract::<TestContract>();

        let command: Command = (&cmd).into();
        let matches = command.try_get_matches_from(vec![PRINT_EVENTS_SUBCOMMAND, "TestContract2"]);

        assert_eq!(
            matches.unwrap_err().kind(),
            clap::error::ErrorKind::InvalidSubcommand
        );
    }

    #[test]
    fn parsing_number_of_events() {
        let cmd = PrintContractEventsCmd::new::<TestContract>(None);
        let command: Command = (&cmd).into();
        let matches = command.get_matches_from(vec!["TestContract", "--number", "5"]);
        assert_eq!(*matches.get_one::<u32>(ARG_NUMBER).unwrap(), 5);
    }

    #[test]
    fn parsing_default_number_of_events() {
        let cmd = PrintContractEventsCmd::new::<TestContract>(None);
        let command: Command = (&cmd).into();
        let matches = command.try_get_matches_from(vec!["TestContract"]);
        assert!(matches.is_ok());
        let matches = matches.unwrap();
        assert!(matches.contains_id(ARG_NUMBER));
        assert_eq!(*matches.get_one::<u32>(ARG_NUMBER).unwrap(), 10);
    }

    #[test]
    fn parsing_default_number_of_events_with_invalid_value() {
        let cmd = PrintContractEventsCmd::new::<TestContract>(None);
        let command: Command = (&cmd).into();
        let matches = command.try_get_matches_from(vec!["TestContract", "--number", "invalid"]);
        assert!(matches.is_err());
        assert_eq!(
            matches.unwrap_err().kind(),
            clap::error::ErrorKind::ValueValidation
        );
    }
}
