use odra_schema::casper_contract_schema::Entrypoint;
use odra_schema::codec::CustomTypeSet;

use crate::cmd::args::CommandArg;
use crate::entry_point::utils::flatten_schema_arg;

pub fn entry_point_args(entry_point: &Entrypoint, types: &CustomTypeSet) -> Vec<CommandArg> {
    entry_point
        .arguments
        .iter()
        .filter(|arg| arg.name != "__cargo_purse")
        .flat_map(|arg| flatten_schema_arg(arg, types, false))
        .flatten()
        .collect()
}
