use anyhow::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};
use odra::{
    casper_types::U512, host::HostEnv, prelude::Address,
    schema::casper_contract_schema::NamedCLType
};
use serde::Serialize;

use crate::{
    cmd::{CmdOutput, TRANSFER_SUBCOMMAND},
    custom_types::CustomTypeSet,
    log,
    parser::{self, CLTypedParser, CsprTokenAmountParser},
    scenario::Args,
    DeployedContractsContainer
};

use super::OdraCommand;

const ARG_TO: &str = "to";
const ARG_AMOUNT: &str = "amount";

#[derive(Serialize)]
pub(crate) struct TransferReport {
    from: Address,
    to: Address,
    amount_motes: String,
    amount_cspr: String
}

impl CmdOutput for TransferReport {
    fn pretty_print(&self) {
        log(format!(
            "Transfer completed successfully.\n {} motes ({} cspr) from\n{} -> {}",
            self.amount_motes,
            self.amount_cspr,
            self.from.to_string(),
            self.to.to_string()
        ));
    }
}

/// Transfers native CSPR from the configured caller to an account or contract address.
pub(crate) struct TransferCmd;

impl OdraCommand for TransferCmd {
    type Output = TransferReport;

    fn exec(
        &self,
        env: &HostEnv,
        args: &ArgMatches,
        _types: &CustomTypeSet,
        _container: &DeployedContractsContainer
    ) -> Result<Self::Output> {
        let amount = args
            .get_one::<U512>(ARG_AMOUNT)
            .map(ToOwned::to_owned)
            .unwrap_or_default();

        let args = Args::new(args);
        let to = args.get_single::<Address>(ARG_TO)?;
        env.transfer(to, amount)
            .map_err(|e| anyhow::anyhow!("Transfer failed: {e:?}"))?;

        Ok(TransferReport {
            from: env.caller(),
            to,
            amount_cspr: parser::motes_to_cspr(amount),
            amount_motes: amount.to_string()
        })
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
