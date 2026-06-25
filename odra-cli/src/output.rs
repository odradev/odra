//! Output-format selection for commands that can render either human-readable text or JSON.
//!
//! Commands that opt in build a `serde::Serialize` model of their result and then render it through
//! one of two paths depending on [`OutputFormat`]: the existing `prettycli` text for humans, or
//! pretty-printed JSON (via [`print_json`]) for machine consumers — scripts, CI, or an MCP server.

use clap::ArgMatches;
use serde::Serialize;

use crate::cmd::args::ARG_JSON;

/// Whether a command should render human-readable text or machine-readable JSON.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// `prettycli` text, the default.
    #[default]
    Human,
    /// Pretty-printed JSON, selected by the global `--json` flag.
    Json
}

impl OutputFormat {
    /// Reads the global `--json` flag from the parsed arguments.
    ///
    /// `--json` is declared `global`, so it is queryable from any subcommand's matches. We query it
    /// defensively (via `try_get_one`) so a hand-built `ArgMatches` without the flag — as used in
    /// unit tests — falls back to [`OutputFormat::Human`] instead of panicking.
    pub fn from_args(args: &ArgMatches) -> Self {
        let json = args
            .try_get_one::<bool>(ARG_JSON)
            .ok()
            .flatten()
            .copied()
            .unwrap_or(false);
        if json {
            Self::Json
        } else {
            Self::Human
        }
    }
}

/// Serializes `value` as pretty JSON to stdout.
pub fn print_json<T: Serialize>(value: &T) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Arg, ArgAction, Command};

    fn matches(argv: &[&str]) -> ArgMatches {
        Command::new("test")
            .arg(Arg::new(ARG_JSON).long("json").action(ArgAction::SetTrue))
            .get_matches_from(argv)
    }

    #[test]
    fn defaults_to_human_without_flag() {
        assert_eq!(
            OutputFormat::from_args(&matches(&["test"])),
            OutputFormat::Human
        );
    }

    #[test]
    fn selects_json_with_flag() {
        assert_eq!(
            OutputFormat::from_args(&matches(&["test", "--json"])),
            OutputFormat::Json
        );
    }

    #[test]
    fn defaults_to_human_when_flag_absent_from_matches() {
        // A hand-built `ArgMatches` without the flag (as used by some unit tests) must not panic.
        assert_eq!(
            OutputFormat::from_args(&ArgMatches::default()),
            OutputFormat::Human
        );
    }
}
