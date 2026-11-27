//! Unsigned transactions, built with casper-types alone (it has no public builder): native CSPR
//! transfers as a `TransactionV1` or as a legacy `Deploy`. Sign them with [`crate::sign`].

use std::collections::{BTreeMap, BTreeSet};

use casper_types::{
    account::AccountHash,
    bytesrepr::{Bytes, ToBytes},
    system::mint,
    Deploy, DeployHash, DeployHeader, Digest, ExecutableDeployItem, InitiatorAddr, PricingMode,
    PublicKey, RuntimeArgs, TimeDiff, Timestamp, Transaction, TransactionArgs,
    TransactionEntryPoint, TransactionScheduling, TransactionTarget, TransactionV1,
    TransactionV1Payload, U512
};

use crate::Error;

/// Field indices of a TransactionV1 payload, as casper-client 5.0.0's builder writes them.
pub(crate) const ARGS_MAP_KEY: u16 = 0;
pub(crate) const TARGET_MAP_KEY: u16 = 1;
const ENTRY_POINT_MAP_KEY: u16 = 2;
const SCHEDULING_MAP_KEY: u16 = 3;

/// What every transaction needs besides what it does.
#[derive(Debug, Clone)]
pub struct Header {
    /// The chain, e.g. `casper-test`.
    pub chain_name: String,
    /// When it is built; the node rejects a timestamp in the future.
    pub timestamp: Timestamp,
    /// How long the node accepts it; at most 2 hours on Casper 2.x.
    pub ttl: TimeDiff,
    /// The payment limit, in motes.
    pub payment: u64
}

fn serialization(error: casper_types::bytesrepr::Error) -> Error {
    Error::Invalid(format!("cannot serialize the transaction: {error}"))
}

fn transfer_args(target: AccountHash, amount: U512, id: Option<u64>) -> Result<RuntimeArgs, Error> {
    let mut args = RuntimeArgs::new();
    let cl = |error| Error::Invalid(format!("cannot build the transfer arguments: {error:?}"));
    args.insert(mint::ARG_TARGET, target).map_err(cl)?;
    args.insert(mint::ARG_AMOUNT, amount).map_err(cl)?;
    args.insert(mint::ARG_ID, id).map_err(cl)?;
    Ok(args)
}

/// A TransactionV1 with the given fields, unsigned.
pub fn unsigned_v1(
    header: Header,
    initiator: InitiatorAddr,
    args: TransactionArgs,
    target: TransactionTarget,
    entry_point: TransactionEntryPoint
) -> Result<Transaction, Error> {
    let mut fields = BTreeMap::new();
    fields.insert(ARGS_MAP_KEY, args.to_bytes().map_err(serialization)?.into());
    fields.insert(
        TARGET_MAP_KEY,
        target.to_bytes().map_err(serialization)?.into()
    );
    fields.insert(
        ENTRY_POINT_MAP_KEY,
        entry_point.to_bytes().map_err(serialization)?.into()
    );
    fields.insert(
        SCHEDULING_MAP_KEY,
        TransactionScheduling::Standard
            .to_bytes()
            .map_err(serialization)?
            .into()
    );
    let payload = TransactionV1Payload::new(
        header.chain_name,
        header.timestamp,
        header.ttl,
        PricingMode::PaymentLimited {
            payment_amount: header.payment,
            gas_price_tolerance: 1,
            standard_payment: true
        },
        initiator,
        fields
    );
    let hash = Digest::hash(payload.to_bytes().map_err(serialization)?);
    Ok(Transaction::V1(TransactionV1::new(
        hash.into(),
        payload,
        BTreeSet::new()
    )))
}

/// A native transfer of `amount` motes to `target`, as an unsigned TransactionV1. The
/// initiator may be an account hash, so the account's own key does not have to sign.
pub fn unsigned_transfer_v1(
    header: Header,
    initiator: InitiatorAddr,
    target: AccountHash,
    amount: U512,
    id: Option<u64>
) -> Result<Transaction, Error> {
    unsigned_v1(
        header,
        initiator,
        TransactionArgs::Named(transfer_args(target, amount, id)?),
        TransactionTarget::Native,
        TransactionEntryPoint::Transfer
    )
}

/// A native transfer of `amount` motes to `target`, as an unsigned legacy Deploy (what the
/// CSPR.cloud awaiting deploy API documents). A Deploy's account is always a public key: the
/// account's own one.
pub fn unsigned_transfer_deploy(
    header: Header,
    account: PublicKey,
    target: AccountHash,
    amount: U512,
    id: Option<u64>
) -> Result<Transaction, Error> {
    let mut payment_args = RuntimeArgs::new();
    payment_args
        .insert(mint::ARG_AMOUNT, U512::from(header.payment))
        .map_err(|error| Error::Invalid(format!("cannot build the payment: {error:?}")))?;
    let payment = ExecutableDeployItem::ModuleBytes {
        module_bytes: Bytes::new(),
        args: payment_args
    };
    let session = ExecutableDeployItem::Transfer {
        args: transfer_args(target, amount, id)?
    };
    // As Deploy::build does: the body hash covers payment and session, the deploy hash the header.
    let mut body = payment.to_bytes().map_err(serialization)?;
    body.extend(session.to_bytes().map_err(serialization)?);
    let deploy_header = DeployHeader::new(
        account,
        header.timestamp,
        header.ttl,
        1,
        Digest::hash(body),
        vec![],
        header.chain_name
    );
    let hash = DeployHash::new(Digest::hash(
        deploy_header.to_bytes().map_err(serialization)?
    ));
    Ok(Transaction::Deploy(Deploy::new(
        hash,
        deploy_header,
        payment,
        session
    )))
}
