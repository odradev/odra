use std::any::Any;

use crate::parser::{CLTypedParser, CsprTokenAmountParser, GasParser};
use clap::{builder::PathBufValueParser, ArgAction, ArgMatches};
use odra::schema::casper_contract_schema::NamedCLType;

pub const ARG_ATTACHED_VALUE: &str = "attached_value";
pub const ARG_GAS: &str = "gas";
pub const ARG_CONTRACTS: &str = "contracts-toml";
pub const ARG_PRINT_EVENTS: &str = "print-events";
pub const ARG_NUMBER: &str = "number";
pub const ARG_DEPLOY_MODE: &str = "deploy-mode";

pub const DEPLOY_MODE_OVERRIDE: &str = "override";
pub const DEPLOY_MODE_ARCHIVE: &str = "archive";

#[derive(Debug, thiserror::Error)]
pub enum ArgsError {
    #[error("Invalid arg value: {0}")]
    TypesError(#[from] crate::types::Error),
    #[error("Decoding error: {0}")]
    DecodingError(String),
    #[error("Arg not found: {0}")]
    ArgNotFound(String),
    #[error("Arg type not found: {0}")]
    ArgTypeNotFound(String)
}

/// A typed command argument.
#[derive(Debug, PartialEq)]
pub struct CommandArg {
    pub name: String,
    pub required: bool,
    pub description: String,
    pub ty: NamedCLType,
    pub is_list_element: bool
}

impl CommandArg {
    pub fn new(name: &str, description: &str, ty: NamedCLType) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            ty,
            required: false,
            is_list_element: false
        }
    }

    pub fn required(self) -> Self {
        Self {
            required: true,
            ..self
        }
    }

    pub fn list(self) -> Self {
        Self {
            is_list_element: true,
            ..self
        }
    }

    pub(crate) fn split_name(&self) -> Vec<String> {
        self.name
            .split('.')
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
    }
}

impl From<CommandArg> for clap::Arg {
    fn from(arg: CommandArg) -> Self {
        let result = clap::Arg::new(&arg.name)
            .long(arg.name)
            .value_name(format!("{:?}", arg.ty))
            .required(arg.required)
            .value_parser(CLTypedParser::new(arg.ty))
            .help(arg.description);

        match arg.is_list_element {
            true => result.action(ArgAction::Append),
            false => result.action(ArgAction::Set)
        }
    }
}

pub enum Arg {
    AttachedValue,
    Gas,
    Contracts,
    EventsNumber,
    PrintEvents,
    DeployMode
}

impl Arg {
    pub fn name(&self) -> &str {
        match self {
            Arg::AttachedValue => ARG_ATTACHED_VALUE,
            Arg::Gas => ARG_GAS,
            Arg::Contracts => ARG_CONTRACTS,
            Arg::EventsNumber => ARG_NUMBER,
            Arg::PrintEvents => ARG_PRINT_EVENTS,
            Arg::DeployMode => ARG_DEPLOY_MODE
        }
    }
}

impl From<Arg> for clap::Arg {
    fn from(arg: Arg) -> Self {
        match arg {
            Arg::AttachedValue => arg_attached_value(),
            Arg::Gas => arg_gas(),
            Arg::Contracts => arg_contracts(),
            Arg::EventsNumber => arg_number("Number of events to print"),
            Arg::PrintEvents => arg_print_events(),
            Arg::DeployMode => arg_deploy_mode()
        }
    }
}

fn arg_attached_value() -> clap::Arg {
    clap::Arg::new(ARG_ATTACHED_VALUE)
        .help("The amount of CSPRs attached to the call")
        .long(ARG_ATTACHED_VALUE)
        .required(false)
        .value_name("CSPR")
        .value_parser(CsprTokenAmountParser)
        .action(ArgAction::Set)
}

fn arg_gas() -> clap::Arg {
    clap::Arg::new(ARG_GAS)
        .help("The amount of gas to attach to the call")
        .long(ARG_GAS)
        .required(true)
        .value_name("CSPR")
        .value_parser(GasParser)
        .action(ArgAction::Set)
}

fn arg_contracts() -> clap::Arg {
    clap::Arg::new(ARG_CONTRACTS)
        .help("The path to the file with the deployed contracts. Relative to the project root.")
        .long(ARG_CONTRACTS)
        .short('c')
        .required(false)
        .value_name("PathBuf")
        .value_parser(PathBufValueParser::new())
        .action(ArgAction::Set)
}

fn arg_number(description: &'static str) -> clap::Arg {
    clap::Arg::new(ARG_NUMBER)
        .short('n')
        .long(ARG_NUMBER)
        .value_name("N")
        .default_value("10")
        .value_parser(clap::value_parser!(u32).range(1..50))
        .help(description)
}

fn arg_print_events() -> clap::Arg {
    clap::Arg::new(ARG_PRINT_EVENTS)
        .long(ARG_PRINT_EVENTS)
        .short('p')
        .help("Print events emitted by the contract")
        .action(ArgAction::SetTrue)
}

fn arg_deploy_mode() -> clap::Arg {
    clap::Arg::new(ARG_DEPLOY_MODE)
        .long(ARG_DEPLOY_MODE)
        .help("Deployment mode strategy.")
        .long_help(
            "Deployment mode strategy:\n\
             - default: Use existing contract if available, otherwise deploy a new one\n\
             - override: Force redeploy, overwrite the existing contract configuration\n\
             - archive: Redeploy contracts, archive the existing contract configuration and create a new one."
        )
        .value_name("MODE")
        .required(false)
        .default_value("default")
        .value_parser(["default", DEPLOY_MODE_OVERRIDE, DEPLOY_MODE_ARCHIVE])
        .action(ArgAction::Set)
}

pub fn read_arg<T: ToOwned<Owned = T> + Any + Clone + Send + Sync + 'static>(
    matches: &ArgMatches,
    arg: Arg
) -> Option<T> {
    matches.get_one::<T>(arg.name()).map(ToOwned::to_owned)
}
