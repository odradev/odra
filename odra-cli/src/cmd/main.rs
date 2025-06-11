use crate::{
    args::ARG_CONTRACTS,
    cmd::args::{read_arg, Arg}
};
use clap::{ArgMatches, Command};
use std::{path::PathBuf, str::FromStr};

/// MainCmd is a struct that represents the main command of the Odra CLI.
pub(crate) struct MainCmd {
    main_cmd: Command
}

impl Default for MainCmd {
    fn default() -> Self {
        let main_cmd = Command::new("Odra CLI")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .arg(Arg::Contracts);
        MainCmd { main_cmd }
    }
}

impl MainCmd {
    /// Sets the description of the CLI
    pub fn about(mut self, about: &'static str) -> Self {
        self.main_cmd = self.main_cmd.about(about);
        self
    }

    pub fn subcommand<T: Into<Command>>(mut self, command: T) -> Self {
        self.main_cmd = self.main_cmd.subcommand(command);
        self
    }

    /// Runs the CLI and parses the input.
    pub fn get_matches(&self) -> (String, ArgMatches, Option<PathBuf>) {
        let matches = match self.main_cmd.clone().try_get_matches() {
            Ok(matches) => matches,
            Err(err) => {
                println!("{}", err);
                std::process::exit(0);
            }
        };

        // Check if the user provided a custom contracts path.
        let contracts_path = read_arg(&matches, ARG_CONTRACTS, PathBuf::from_str).ok();

        let result = matches.subcommand();

        let (cmd, args) = match result {
            Some((cmd, args)) => (cmd, args),
            None => {
                prettycli::error("No subcommand provided. Use --help to see available commands.");
                std::process::exit(1);
            }
        };

        (cmd.to_string(), args.clone(), contracts_path)
    }
}
