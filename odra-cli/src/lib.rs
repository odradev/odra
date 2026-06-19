//! A rust library for building command line interfaces for Odra smart contracts.
//!
//! The Odra CLI is a command line interface built on top of the [clap] crate
//! that allows users to interact with smart contracts.

#![feature(box_patterns, error_generic_member_access)]
mod cli;
mod cmd;
mod container;
mod custom_types;
mod entry_point;
mod output;
mod parser;
#[cfg(test)]
mod test_utils;
mod types;
mod utils;

pub use cli::OdraCli;
pub use cmd::args::CommandArg;
pub use container::{ContractProvider, DeployedContractsContainer};
pub use utils::{log, DeployerExt};

pub mod scenario {
    //! Traits and structs for defining custom scenarios.
    //!
    //! A scenario is a user-defined set of actions that can be run in the Odra CLI.
    //! If you want to run a custom scenario that calls multiple entry points,
    //! you need to implement the [Scenario] and [ScenarioMetadata] traits.
    pub use crate::cmd::{
        Scenario, ScenarioArgs as Args, ScenarioError as Error, ScenarioMetadata
    };
}

pub mod deploy {
    //! Traits and structs for defining deploy scripts.
    //!
    //! In a deploy script, you can define the contracts that you want to deploy to the blockchain
    //! and write metadata to the container.
    pub use crate::cmd::{DeployError as Error, DeployScript};
}

#[macro_export]
macro_rules! cspr {
    ($val:literal) => {{
        const MOTES_PER_CSPR: u64 = 1_000_000_000;
        ($val as f64 * MOTES_PER_CSPR as f64) as u64
    }};
    ($val:expr) => {{
        const MOTES_PER_CSPR: u64 = 1_000_000_000;
        let parsed: f64 = {
            if let Some(s) = $val.as_ref().downcast_ref::<&str>() {
                s.parse().expect("Invalid number string")
            } else if let Some(s) = $val.as_ref().downcast_ref::<String>() {
                s.parse().expect("Invalid number string")
            } else {
                $val as f64
            }
        };
        (parsed * MOTES_PER_CSPR as f64) as u64
    }};
}
