use casper_types::{account::AccountHash, PublicKey, Timestamp, Transaction, TransactionHash};
use odra_casper_account_manager::{AccountState, ThresholdKind};

use crate::decode_account_change;

/// One signer of a transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signer {
    /// The key that signed.
    pub public_key: PublicKey,
    /// Its account hash.
    pub account_hash: AccountHash,
    /// Its weight in the account the transaction runs as; `None` if it is not one of the
    /// account's associated keys, which makes the node reject the transaction.
    pub weight: Option<u8>
}

/// Whether the transaction can be sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    /// Its time to live is over; it can only be built again.
    Expired,
    /// A signer is not an associated key: the node would reject it whatever else signs.
    ForeignSigner,
    /// The signers' weight is below the threshold it needs.
    Pending {
        /// The weight still missing.
        missing: u8
    },
    /// Signed with enough weight.
    Ready
}

/// What a transaction does with the account it runs as, and how far its signing is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inspection {
    /// The transaction.
    pub hash: TransactionHash,
    /// The account it runs as (its initiator).
    pub account: AccountHash,
    /// The chain it is for.
    pub chain_name: String,
    /// When it was built.
    pub timestamp: Timestamp,
    /// After this, the node no longer accepts it.
    pub expires: Timestamp,
    /// Who signed it so far.
    pub signers: Vec<Signer>,
    /// The signers' total weight (saturating at 255, like the engine).
    pub weight: u8,
    /// The threshold it needs: `key_management` for an account manager transaction, otherwise
    /// `deployment`.
    pub required: ThresholdKind,
    /// That threshold's value.
    pub threshold: u8,
    /// Whether it can be sent now.
    pub status: Status
}

/// Inspects `transaction` against `account`, the current configuration of the account it runs
/// as, at `now`.
pub fn inspect(transaction: &Transaction, account: &AccountState, now: Timestamp) -> Inspection {
    let signers: Vec<Signer> = transaction
        .approvals()
        .iter()
        .map(|approval| {
            let account_hash = approval.signer().to_account_hash();
            Signer {
                public_key: approval.signer().clone(),
                account_hash,
                weight: account.associated_keys.get(&account_hash).copied()
            }
        })
        .collect();
    let weight = account.weight_of(signers.iter().map(|signer| &signer.account_hash));
    let (required, threshold) = if decode_account_change(transaction).is_some() {
        (ThresholdKind::KeyManagement, account.key_management)
    } else {
        (ThresholdKind::Deployment, account.deployment)
    };
    let status = if transaction.expired(now) {
        Status::Expired
    } else if signers.iter().any(|signer| signer.weight.is_none()) {
        Status::ForeignSigner
    } else if weight < threshold {
        Status::Pending {
            missing: threshold - weight
        }
    } else {
        Status::Ready
    };
    Inspection {
        hash: transaction.hash(),
        account: transaction.initiator_addr().account_hash(),
        chain_name: transaction.chain_name(),
        timestamp: transaction.timestamp(),
        expires: transaction.expires(),
        signers,
        weight,
        required,
        threshold,
        status
    }
}
