use std::path::PathBuf;

use anyhow::Result;
use clap::ArgMatches;
use odra::host::HostEnv;

use crate::CustomTypeSet;

pub mod args;
pub mod contract;
pub mod deploy;
pub mod events;
pub mod main;
pub mod scenario;

/// OdraCommand is a trait that represents a command that can be run in the Odra CLI.
pub(crate) trait OdraCommand {
    fn run(
        &self,
        env: &HostEnv,
        args: &ArgMatches,
        types: &CustomTypeSet,
        contracts_path: Option<PathBuf>
    ) -> Result<()>;
}
