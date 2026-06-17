use crate::cmd::args::{read_arg, Arg};
use clap::{ArgMatches, Command};
use std::path::PathBuf;

/// MainCmd is a struct that represents the main command of the Odra CLI.
#[derive(Default)]
pub(crate) struct MainCmd {
    sub_cmds: Vec<Command>,
    about: Option<&'static str>
}

impl From<&MainCmd> for Command {
    fn from(value: &MainCmd) -> Self {
        value.to_command(&[])
    }
}

impl MainCmd {
    /// Sets the description of the CLI
    pub fn about(mut self, about: &'static str) -> Self {
        self.about = Some(about);
        self
    }

    pub fn subcommand<T: Into<Command>>(mut self, command: T) -> Self {
        self.sub_cmds.push(command.into());
        self
    }

    /// Builds the clap command, omitting any subcommand whose name appears in `exclude`.
    ///
    /// The REPL uses this to hide the `repl` subcommand from itself — both from help output and
    /// from parsing — so it can't be invoked recursively from inside an interactive session.
    pub fn to_command(&self, exclude: &[&str]) -> Command {
        let mut cmd = Command::new("Odra CLI")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .arg(Arg::Contracts)
            .subcommands(
                self.sub_cmds
                    .iter()
                    .filter(|c| !exclude.contains(&c.get_name()))
                    .cloned()
            );
        if let Some(about) = self.about {
            cmd = cmd.about(about);
        }
        cmd
    }

    /// Runs the CLI and parses the input from `std::env::args()`.
    ///
    /// Exits the process on any parse error or help/version request, mirroring clap's
    /// default behavior. Use this for the one-shot `run` path.
    pub fn get_matches(&self) -> (String, ArgMatches, Option<PathBuf>) {
        match self.try_get_matches_from(std::env::args().collect()) {
            Ok(result) => result,
            Err(err) => err.exit()
        }
    }

    /// Parses an explicit argv vector without exiting the process.
    ///
    /// Returns the `clap::Error` instead of calling `err.exit()`, so the caller (e.g. the REPL
    /// loop) can render errors and help without killing the session. The error covers both real
    /// parse failures and help/version requests (which arrive as `DisplayHelp*` kinds).
    pub fn try_get_matches_from(
        &self,
        argv: Vec<String>
    ) -> Result<(String, ArgMatches, Option<PathBuf>), clap::Error> {
        self.try_get_matches_from_excluding(argv, &[])
    }

    /// Like [`try_get_matches_from`](Self::try_get_matches_from), but omits the named subcommands.
    ///
    /// Used by the REPL to parse input against a command tree that excludes `repl` itself.
    pub fn try_get_matches_from_excluding(
        &self,
        argv: Vec<String>,
        exclude: &[&str]
    ) -> Result<(String, ArgMatches, Option<PathBuf>), clap::Error> {
        let clap_cmd = self.to_command(exclude);
        let matches = clap_cmd.try_get_matches_from(argv)?;

        // Check if the user provided a custom contracts path.
        let contracts_path = read_arg(&matches, Arg::Contracts);

        // `subcommand_required(true)` guarantees a subcommand is present on success.
        let (subcommand, args) = matches
            .subcommand()
            .expect("subcommand is required and validated by clap");

        Ok((subcommand.to_string(), args.clone(), contracts_path))
    }
}

#[cfg(test)]
mod tests {
    use clap::{command, error::ErrorKind};

    use super::*;

    #[test]
    fn test_main_cmd_default() {
        let main = MainCmd::default();
        let main_cmd: Command = (&main).into();
        assert_eq!(main_cmd.get_name(), "Odra CLI");
    }

    #[test]
    fn test_main_cmd_requires_subcommand() {
        let main = MainCmd::default();
        let main_cmd: Command = (&main).into();

        let result = main_cmd.try_get_matches_from(vec!["odra-cli"]);
        assert_eq!(
            result.unwrap_err().kind(),
            ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
        );
    }

    #[test]
    fn test_contracts_arg() {
        let main = MainCmd::default().subcommand(command!("test"));
        let main_cmd: Command = (&main).into();

        let matches = main_cmd.get_matches_from(vec![
            "odra-cli",
            "--contracts-toml",
            "path/to/contracts",
            "test",
        ]);
        let contracts_path = read_arg(&matches, Arg::Contracts);
        assert_eq!(contracts_path, Some(PathBuf::from("path/to/contracts")));
    }

    #[test]
    fn try_get_matches_from_returns_ok_for_valid_subcommand() {
        let main = MainCmd::default().subcommand(command!("whoami"));

        let (cmd, _args, path) = main
            .try_get_matches_from(vec!["odra-cli".to_string(), "whoami".to_string()])
            .expect("valid subcommand should parse");
        assert_eq!(cmd, "whoami");
        assert_eq!(path, None);
    }

    #[test]
    fn try_get_matches_from_returns_err_for_unknown_subcommand() {
        let main = MainCmd::default().subcommand(command!("whoami"));

        let err = main
            .try_get_matches_from(vec!["odra-cli".to_string(), "bogus".to_string()])
            .expect_err("unknown subcommand should return an error, not exit");
        assert_eq!(err.kind(), ErrorKind::InvalidSubcommand);
    }

    #[test]
    fn try_get_matches_from_returns_err_for_help() {
        let main = MainCmd::default().subcommand(command!("whoami"));

        let err = main
            .try_get_matches_from(vec!["odra-cli".to_string(), "--help".to_string()])
            .expect_err("--help should return an error, not exit");
        assert_eq!(err.kind(), ErrorKind::DisplayHelp);
    }
}
