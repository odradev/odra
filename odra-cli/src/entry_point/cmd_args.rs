use odra::schema::casper_contract_schema::Entrypoint;

use crate::cmd::args::CommandArg;
use crate::custom_types::CustomTypeSet;
use crate::entry_point::utils::flatten_schema_arg;

pub fn entry_point_args(entry_point: &Entrypoint, types: &CustomTypeSet) -> Vec<CommandArg> {
    entry_point
        .arguments
        .iter()
        .flat_map(|arg| flatten_schema_arg(arg, types, false))
        .flatten()
        .collect()
}
