use std::path::Path;

use anyhow::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};
use clap_complete::Shell;

use crate::cmd::COMPLETIONS_SUBCOMMAND;

const ARG_SHELL: &str = "shell";

/// Emits a shell completion script for the whole generated command tree.
///
/// Unlike the other commands this one needs the fully assembled clap [`Command`], so generation is
/// driven from the CLI dispatch (which owns the command tree) via [`CompletionsCmd::generate`].
pub(crate) struct CompletionsCmd;

impl CompletionsCmd {
    /// Writes the completion script for the requested shell to stdout.
    ///
    /// `command` is the top-level command tree to generate completions for.
    pub fn generate(&self, args: &ArgMatches, mut command: Command) -> Result<()> {
        let shell = args
            .get_one::<Shell>(ARG_SHELL)
            .copied()
            .ok_or_else(|| anyhow::anyhow!("Missing shell. Use `completions <SHELL>`."))?;
        let bin_name = bin_name();
        clap_complete::generate(shell, &mut command, bin_name, &mut std::io::stdout());
        Ok(())
    }
}

/// Best-effort binary name for the generated script, taken from `argv[0]`.
fn bin_name() -> String {
    std::env::args()
        .next()
        .and_then(|arg0| {
            Path::new(&arg0)
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "odra-cli".to_string())
}

impl From<&CompletionsCmd> for Command {
    fn from(_value: &CompletionsCmd) -> Self {
        Command::new(COMPLETIONS_SUBCOMMAND)
            .about("Generates a shell completion script (bash, zsh, fish, ...)")
            .arg(
                Arg::new(ARG_SHELL)
                    .required(true)
                    .value_name("SHELL")
                    .help("The shell to generate completions for")
                    .value_parser(clap::value_parser!(Shell))
                    .action(ArgAction::Set)
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_completions_command() {
        let clap_cmd: Command = (&CompletionsCmd).into();
        assert_eq!(clap_cmd.get_name(), COMPLETIONS_SUBCOMMAND);
    }

    #[test]
    fn requires_shell() {
        let clap_cmd: Command = (&CompletionsCmd).into();
        let result = clap_cmd.try_get_matches_from(vec!["completions"]);
        assert_eq!(
            result.unwrap_err().kind(),
            clap::error::ErrorKind::MissingRequiredArgument
        );
    }

    #[test]
    fn parses_known_shell() {
        let clap_cmd: Command = (&CompletionsCmd).into();
        let matches = clap_cmd
            .try_get_matches_from(vec!["completions", "bash"])
            .expect("bash is a valid shell");
        assert_eq!(
            matches.get_one::<Shell>(ARG_SHELL).copied(),
            Some(Shell::Bash)
        );
    }
}
