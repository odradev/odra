use std::path::PathBuf;

use casper_types::TransactionHash;

/// What can go wrong while collecting signatures.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A transaction file could not be read or written.
    #[error("{path}: {source}")]
    Io {
        /// The file.
        path: PathBuf,
        /// The cause.
        source: std::io::Error
    },
    /// A transaction file is not a transaction.
    #[error("{path} is not a Casper transaction: {reason}")]
    NotATransaction {
        /// The file.
        path: PathBuf,
        /// What is wrong with it.
        reason: String
    },
    /// The transaction's hash or one of its signatures does not verify.
    #[error("the transaction does not verify: {0}")]
    Invalid(String),
    /// An approval belongs to another transaction.
    #[error("the signature is for transaction {found}, not {expected}")]
    OtherTransaction {
        /// The transaction being signed.
        expected: TransactionHash,
        /// The one the approval is for.
        found: TransactionHash
    },
    /// An approval's signature does not match its public key and the transaction hash.
    #[error("the signature of {0} does not verify")]
    BadSignature(String),
    /// More approvals than a transaction can carry.
    #[error("{0} approvals, more than the {max} a transaction can carry", max = crate::MAX_APPROVALS)]
    TooManyApprovals(usize),
    /// A signature string could not be read.
    #[error("not a signature string ({0}); expected odra-sig:<v1|deploy>:<hash>:<public key>:<signature>")]
    BadSignatureString(String),
    /// The transport has no transaction with this hash.
    #[error("no transaction {0} here")]
    UnknownTransaction(TransactionHash)
}
