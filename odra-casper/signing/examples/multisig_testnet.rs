//! The offline steps of a multisig transfer on Casper testnet, for
//! `examples/multisig-testnet.sh`, which does the network part with curl.
//!
//! It reads secret keys only from the files given, and never prints them. It builds for
//! `casper-test` only.
//!
//! ```text
//! multisig_testnet public-key <secret_key.pem>
//! multisig_testnet build-transfer <v1|deploy> <account public key> <target account hash> <motes> <out.json>
//! multisig_testnet sign <tx.json> <secret_key.pem>            # prints one odra-sig:... line
//! multisig_testnet merge <tx.json> <out.json> <sig file>...
//! multisig_testnet inspect <tx.json> <account.json>           # account.json: state_get_account_info .result.account
//! ```

use std::{path::Path, process::exit};

use casper_types::{
    account::{Account, AccountHash},
    AsymmetricType, InitiatorAddr, PublicKey, SecretKey, TimeDiff, Timestamp, U512
};
use odra_casper_account_manager::AccountState;
use odra_casper_signing::{
    build::{unsigned_transfer_deploy, unsigned_transfer_v1, Header},
    inspect, merge, read_transaction, sign, write_transaction, SignatureString, Status
};

/// The only chain this example builds for.
const CHAIN_NAME: &str = "casper-test";
/// Native transfers cost 0.1 CSPR on Casper 2.x.
const TRANSFER_PAYMENT: u64 = 100_000_000;

fn fail(message: impl std::fmt::Display) -> ! {
    eprintln!("error: {message}");
    exit(1)
}

fn secret_key(path: &str) -> SecretKey {
    SecretKey::from_file(path)
        .unwrap_or_else(|_| fail(format!("cannot read the secret key {path}")))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["public-key", key] => println!("{}", PublicKey::from(&secret_key(key)).to_hex()),
        ["build-transfer", kind, account, target, motes, out] => {
            let account =
                PublicKey::from_hex(account).unwrap_or_else(|_| fail("bad account public key"));
            let target = AccountHash::from_formatted_str(target)
                .unwrap_or_else(|_| fail("the target must be account-hash-..."));
            let amount = U512::from_dec_str(motes).unwrap_or_else(|_| fail("bad amount"));
            let header = Header {
                chain_name: CHAIN_NAME.to_string(),
                timestamp: Timestamp::now(),
                ttl: TimeDiff::from_seconds(30 * 60),
                payment: TRANSFER_PAYMENT
            };
            let transaction = match *kind {
                // The account hash as initiator: the account's own key does not have to sign.
                "v1" => unsigned_transfer_v1(
                    header,
                    InitiatorAddr::AccountHash(account.to_account_hash()),
                    target,
                    amount,
                    None
                ),
                "deploy" => unsigned_transfer_deploy(header, account, target, amount, None),
                _ => fail("the kind is v1 or deploy")
            }
            .unwrap_or_else(|error| fail(error));
            write_transaction(Path::new(out), &transaction).unwrap_or_else(|error| fail(error));
            println!("{}", transaction.hash());
        }
        ["sign", tx, key] => {
            let transaction = read_transaction(Path::new(tx)).unwrap_or_else(|error| fail(error));
            let approval = sign(&transaction, &secret_key(key));
            let line = SignatureString::new(transaction.hash(), approval)
                .unwrap_or_else(|error| fail(error));
            println!("{line}");
        }
        ["merge", tx, out, sigs @ ..] if !sigs.is_empty() => {
            let transaction = read_transaction(Path::new(tx)).unwrap_or_else(|error| fail(error));
            let approvals: Vec<_> = sigs
                .iter()
                .map(|file| {
                    let text = std::fs::read_to_string(file)
                        .unwrap_or_else(|_| fail(format!("cannot read {file}")));
                    let line: SignatureString = text.parse().unwrap_or_else(|error| fail(error));
                    line.approval_for(&transaction)
                        .unwrap_or_else(|error| fail(error))
                })
                .collect();
            let merged = merge(transaction, approvals).unwrap_or_else(|error| fail(error));
            write_transaction(Path::new(out), &merged).unwrap_or_else(|error| fail(error));
            println!("{} approvals", merged.approvals().len());
        }
        ["inspect", tx, account] => {
            let transaction = read_transaction(Path::new(tx)).unwrap_or_else(|error| fail(error));
            let json = std::fs::read_to_string(account)
                .unwrap_or_else(|_| fail(format!("cannot read {account}")));
            let account: Account = serde_json::from_str(&json)
                .unwrap_or_else(|error| fail(format!("not an account: {error}")));
            let state = AccountState {
                associated_keys: account
                    .associated_keys()
                    .iter()
                    .map(|(key, weight)| (*key, weight.value()))
                    .collect(),
                deployment: account.action_thresholds().deployment().value(),
                key_management: account.action_thresholds().key_management().value()
            };
            let inspection = inspect(&transaction, &state, Timestamp::now());
            println!("transaction {}", inspection.hash);
            println!("account     {}", inspection.account.to_formatted_string());
            for signer in &inspection.signers {
                let weight = signer
                    .weight
                    .map(|weight| weight.to_string())
                    .unwrap_or_else(|| "NOT AN ASSOCIATED KEY".to_string());
                println!(
                    "signer      {}  weight {weight}",
                    signer.account_hash.to_formatted_string()
                );
            }
            println!(
                "weight      {} of {} ({:?})",
                inspection.weight, inspection.threshold, inspection.required
            );
            println!("expires     {}", inspection.expires);
            match inspection.status {
                Status::Ready => println!("status      ready"),
                Status::Pending { missing } => {
                    println!("status      pending, {missing} more weight needed")
                }
                Status::ForeignSigner => println!("status      a signer is not an associated key"),
                Status::Expired => println!("status      expired")
            }
        }
        _ => fail("usage: see the top of examples/multisig_testnet.rs")
    }
}
