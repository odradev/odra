use std::any::Any;

use crate::parser::{CLTypedParser, GenericCLValueParser};
use clap::{builder::PathBufValueParser, ArgAction, ArgMatches};
use odra::{
    casper_types::{bytesrepr::FromBytes, CLTyped, CLValue, U512},
    schema::casper_contract_schema::NamedCLType
};

pub const ARG_ATTACHED_VALUE: &str = "attached_value";
pub const ARG_GAS: &str = "gas";
pub const ARG_CONTRACTS: &str = "contracts-toml";
pub const ARG_PRINT_EVENTS: &str = "print-events";
pub const ARG_NUMBER: &str = "number";

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
    PrintEvents
}

impl Arg {
    pub fn name(&self) -> &str {
        match self {
            Arg::AttachedValue => ARG_ATTACHED_VALUE,
            Arg::Gas => ARG_GAS,
            Arg::Contracts => ARG_CONTRACTS,
            Arg::EventsNumber => ARG_NUMBER,
            Arg::PrintEvents => ARG_PRINT_EVENTS
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
            Arg::PrintEvents => arg_print_events()
        }
    }
}

fn arg_attached_value() -> clap::Arg {
    clap::Arg::new(ARG_ATTACHED_VALUE)
        .help("The amount of CSPRs attached to the call")
        .long(ARG_ATTACHED_VALUE)
        .required(false)
        .value_name(format!("{:?}", NamedCLType::U512))
        .value_parser(GenericCLValueParser::<U512>::new())
        .action(ArgAction::Set)
}

fn arg_gas() -> clap::Arg {
    clap::Arg::new(ARG_GAS)
        .help("The amount of gas to attach to the call")
        .long(ARG_GAS)
        .required(true)
        .value_name(format!("{:?}", NamedCLType::U64))
        .value_parser(clap::value_parser!(u64).range(2_500_000_000..))
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

pub fn read_arg<T: ToOwned<Owned = T> + Any + Clone + Send + Sync + 'static>(
    matches: &ArgMatches,
    arg: Arg
) -> Option<T> {
    matches.get_one::<T>(arg.name()).map(ToOwned::to_owned)
}

pub fn read_cl_value_arg<
    T: CLTyped + FromBytes + ToOwned<Owned = T> + Any + Clone + Send + Sync + 'static
>(
    matches: &ArgMatches,
    arg: Arg
) -> Option<T> {
    matches
        .get_one::<CLValue>(arg.name())
        .map(ToOwned::to_owned)
        .map(|cl_value| cl_value.into_t::<T>().ok())
        .flatten()
}
