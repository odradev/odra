use crate::types;
use clap::{ArgAction, ArgMatches};
use odra::schema::casper_contract_schema::NamedCLType;

pub const ARG_ATTACHED_VALUE: &str = "attached_value";
pub const ARG_GAS: &str = "gas";
pub const ARG_CONTRACTS: &str = "contracts-toml";
pub const ARG_PRINT_EVENTS: &str = "print-events";
pub const ARG_NUMBER: &str = "number";

pub enum Arg {
    AttachedValue,
    Gas,
    Contracts,
    Number(String),
    PrintEvents
}

impl From<Arg> for clap::Arg {
    fn from(arg: Arg) -> Self {
        match arg {
            Arg::AttachedValue => arg_attached_value(),
            Arg::Gas => arg_gas(),
            Arg::Contracts => arg_contracts(),
            Arg::Number(description) => arg_number(description),
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
        .action(ArgAction::Set)
}

fn arg_gas() -> clap::Arg {
    clap::Arg::new(ARG_GAS)
        .help("The amount of gas to attach to the call")
        .long(ARG_GAS)
        .required(true)
        .value_name(format!("{:?}", NamedCLType::U64))
        .action(ArgAction::Set)
}

fn arg_contracts() -> clap::Arg {
    clap::Arg::new(ARG_CONTRACTS)
        .help("The path to the file with the deployed contracts. Relative to the project root.")
        .long(ARG_CONTRACTS)
        .short('c')
        .required(false)
        .value_name(format!("{:?}", NamedCLType::String))
        .action(ArgAction::Set)
}

fn arg_number(description: String) -> clap::Arg {
    clap::Arg::new(ARG_NUMBER)
        .short('n')
        .long(ARG_NUMBER)
        .value_name("N")
        .default_value("10")
        .help(description)
}

fn arg_print_events() -> clap::Arg {
    clap::Arg::new(ARG_PRINT_EVENTS)
        .long(ARG_PRINT_EVENTS)
        .short('p')
        .help("Print events emitted by the contract")
        .action(ArgAction::SetTrue)
}

pub fn read_arg<T: Default, E, F: FnOnce(&str) -> Result<T, E>>(
    args: &ArgMatches,
    name: &str,
    f: F
) -> Result<T, types::Error> {
    args.try_get_one::<String>(name)
        .ok()
        .flatten()
        .map(|s| f(s).map_err(|_| types::Error::Serialization))
        .unwrap_or(Ok(T::default()))
}
