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
pub use utils::{log, ContractLoaderExt, DeployerExt};

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

/// Converts an amount of CSPR to motes (`u64`), e.g. `cspr!(2.5)` is `2_500_000_000`.
///
/// With a literal argument the macro expands to a constant expression, so it can
/// initialise a `const` or `static` (`const DEPLOY_GAS: u64 = cspr!(450);`).
/// For a `U512` constant, combine it with `odra::prelude::u512`:
/// `const STAKE: U512 = u512(cspr!(500) as u128);`.
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

#[cfg(test)]
mod cspr_tests {
    use odra::casper_types::U512;
    use odra::prelude::u512;

    /// `const` items fail to build if the literal arm ever stops being a
    /// constant expression.
    #[test]
    fn cspr_literal_is_usable_in_a_const_context() {
        const WHOLE: u64 = cspr!(450);
        const FRACTION: u64 = cspr!(2.5);
        const STAKE: U512 = u512(cspr!(500) as u128);

        assert_eq!(WHOLE, 450_000_000_000);
        assert_eq!(FRACTION, 2_500_000_000);
        assert_eq!(STAKE, U512::from(500_000_000_000u64));
    }
}
