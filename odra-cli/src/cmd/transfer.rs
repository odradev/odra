use anyhow::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};
use odra::{
    casper_types::U512, host::HostEnv, prelude::Address,
    schema::casper_contract_schema::NamedCLType
};

use crate::{
    cmd::TRANSFER_SUBCOMMAND,
    custom_types::CustomTypeSet,
    parser::{CLTypedParser, CsprTokenAmountParser},
    scenario::Args,
    DeployedContractsContainer
};

use super::OdraCommand;

const ARG_TO: &str = "to";
const ARG_AMOUNT: &str = "amount";

/// Transfers native CSPR from the configured caller to an account or contract address.
pub(crate) struct TransferCmd;

impl OdraCommand for TransferCmd {
    fn run(
        &self,
        env: &HostEnv,
        args: &ArgMatches,
        _types: &CustomTypeSet,
        _container: &DeployedContractsContainer
    ) -> Result<()> {
        let amount = args
            .get_one::<U512>(ARG_AMOUNT)
            .map(ToOwned::to_owned)
            .unwrap_or_default();

        let args = Args::new(args);
        let to = args.get_single::<Address>(ARG_TO)?;
        // let amount = args.get_single::<U512>(ARG_AMOUNT)?;
        // let amount = read_arg::<U512>(args, Arg::AttachedValue).unwrap_or_default();

        prettycli::info(&format!(
            "Transferring {amount} motes from\n{} -> {}",
            env.caller().to_string(),
            to.to_string()
        ));
        env.transfer(to, amount)
            .map_err(|e| anyhow::anyhow!("Transfer failed: {e:?}"))?;
        prettycli::info("Transfer completed successfully.");
        Ok(())
    }
}

impl From<&TransferCmd> for Command {
    fn from(_value: &TransferCmd) -> Self {
        Command::new(TRANSFER_SUBCOMMAND)
            .about("Transfers native CSPR from the caller to an account or contract")
            .arg(
                Arg::new(ARG_TO)
                    .long(ARG_TO)
                    .required(true)
                    .value_name("ADDRESS")
                    .help("Recipient address (hash-... or account-hash-...)")
                    .value_parser(CLTypedParser::new(NamedCLType::Key))
                    .action(ArgAction::Set)
            )
            .arg(
                Arg::new(ARG_AMOUNT)
                    .long(ARG_AMOUNT)
                    .required(true)
                    .value_name("AMOUNT (motes, or 'X.Y cspr')")
                    .help("Amount of CSPR to transfer")
                    .value_parser(CsprTokenAmountParser)
                    .action(ArgAction::Set)
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_transfer_command() {
        let clap_cmd: Command = (&TransferCmd).into();
        assert_eq!(clap_cmd.get_name(), TRANSFER_SUBCOMMAND);
    }

    #[test]
    fn requires_to_and_amount() {
        let clap_cmd: Command = (&TransferCmd).into();
        let result = clap_cmd.try_get_matches_from(vec!["transfer"]);
        assert_eq!(
            result.unwrap_err().kind(),
            clap::error::ErrorKind::MissingRequiredArgument
        );
    }

    #[test]
    fn parses_to_and_amount() {
        let clap_cmd: Command = (&TransferCmd).into();
        let result = clap_cmd.try_get_matches_from(vec![
            "transfer",
            "--to",
            "account-hash-5e3725bec4389ea63151903f5c9005233d19a569c5e593e5bbd83b05714f7364",
            "--amount",
            "1000",
        ]);
        assert!(result.is_ok());
    }
}
