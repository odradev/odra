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
