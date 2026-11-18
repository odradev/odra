pub mod bdd_env;
pub mod odra_world;
pub mod types;
pub mod utils;
mod virtual_balances;

use cucumber::{cli, codegen::WorldInventory, World};
use std::fmt::Debug;

/// Runs the feature files found at `path` (a file or a directory) and exits the process:
/// with a failure if a step failed or has no definition.
///
/// It is the `main` of a test target with `harness = false`, so the arguments are those of
/// `cargo test`: the first free argument selects the scenarios whose name, or whose feature's
/// name, contains it; flags such as `--nocapture` are ignored. Scenarios run one at a time.
pub fn run<W: World + Debug + WorldInventory>(path: &str) {
    let filter = name_filter(std::env::args().skip(1));
    let run = W::cucumber()
        .with_cli(cli::Opts::<_, _, _, cli::Empty>::default())
        .max_concurrent_scenarios(1)
        .fail_on_skipped()
        .filter_run_and_exit(path.to_string(), move |feature, _, scenario| {
            filter.as_ref().is_none_or(|text| {
                scenario.name.contains(text.as_str()) || feature.name.contains(text.as_str())
            })
        });
    futures::executor::block_on(run);
}

/// The name filter among the arguments of `cargo test -- <filter> --nocapture ...`.
fn name_filter(mut args: impl Iterator<Item = String>) -> Option<String> {
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--test-threads" | "--skip" | "--color" | "--format" | "--logfile" | "-Z" => {
                args.next();
            }
            flag if flag.starts_with('-') => {}
            _ => return Some(arg)
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::name_filter;
    use crate::types::{
        account::Account,
        cspr::CSPRAmount,
        token_amount::{USDCAmount, WrappedCSPRAmount}
    };
    use cucumber::{codegen::Regex, Parameter};

    /// A capturing group inside a parameter shifts every parameter that follows it in a step.
    #[test]
    fn parameters_have_no_capturing_groups() {
        for regex in [
            Account::REGEX,
            CSPRAmount::REGEX,
            USDCAmount::REGEX,
            WrappedCSPRAmount::REGEX
        ] {
            assert_eq!(Regex::new(regex).unwrap().captures_len(), 1, "{regex}");
        }
    }

    fn filter(args: &[&str]) -> Option<String> {
        name_filter(args.iter().map(|arg| arg.to_string()))
    }

    #[test]
    fn name_filter_is_the_first_free_argument() {
        assert_eq!(filter(&[]), None);
        assert_eq!(filter(&["--nocapture"]), None);
        assert_eq!(filter(&["--test-threads", "1"]), None);
        assert_eq!(
            filter(&["--test-threads=1", "bid"]),
            Some("bid".to_string())
        );
        assert_eq!(
            filter(&["--skip", "slow", "highest bidder", "--nocapture"]),
            Some("highest bidder".to_string())
        );
    }
}
