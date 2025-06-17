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
        let mut cmd = Command::new("Odra CLI")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .arg(Arg::Contracts)
            .subcommands(&value.sub_cmds);
        if let Some(about) = value.about {
            cmd = cmd.about(about);
        }
        cmd
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

    /// Runs the CLI and parses the input.
    pub fn get_matches(&self) -> (String, ArgMatches, Option<PathBuf>) {
        let clap_cmd: Command = self.into();
        let matches = match clap_cmd.try_get_matches() {
            Ok(matches) => matches,
            Err(err) => {
                err.exit();
            }
        };

        // Check if the user provided a custom contracts path.
        let contracts_path = read_arg(&matches, Arg::Contracts);

        let result = matches.subcommand();

        let (subcommand, args) = match result {
            Some((subcommand, args)) => (subcommand, args),
            None => {
                prettycli::error("No subcommand provided. Use --help to see available commands.");
                std::process::exit(1);
            }
        };

        (subcommand.to_string(), args.clone(), contracts_path)
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
}
