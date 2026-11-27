use std::path::{Path, PathBuf};

use casper_types::{Approval, Transaction, TransactionHash};

use crate::{merge, read_transaction, write_transaction, Error};

/// Where a transaction and its approvals are kept while they are collected from several
/// machines.
pub trait SigningTransport {
    /// Makes the transaction, usually signed by the first key, available to the other signers.
    fn publish(&self, transaction: &Transaction) -> Result<(), Error>;
    /// Adds one approval. It must be a valid signature of the transaction.
    fn add_approval(&self, hash: &TransactionHash, approval: Approval) -> Result<(), Error>;
    /// The transaction with every approval collected so far.
    fn fetch(&self, hash: &TransactionHash) -> Result<Transaction, Error>;
}

/// Keeps each transaction as `<hash>.json` in a directory: the offline flow, where the file is
/// passed between the signers by any means and its approvals are merged into it.
pub struct FileTransport {
    dir: PathBuf
}

impl FileTransport {
    /// A transport over `dir`, which must exist.
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        FileTransport { dir: dir.into() }
    }

    /// The file of a transaction.
    pub fn path(&self, hash: &TransactionHash) -> PathBuf {
        let hex = match hash {
            TransactionHash::V1(hash) => format!("{:?}", hash.inner()),
            TransactionHash::Deploy(hash) => format!("{:?}", hash.inner())
        };
        self.dir.join(format!("{hex}.json"))
    }

    /// The directory.
    pub fn dir(&self) -> &Path {
        &self.dir
    }
}

impl SigningTransport for FileTransport {
    fn publish(&self, transaction: &Transaction) -> Result<(), Error> {
        let path = self.path(&transaction.hash());
        // Publishing again keeps the approvals already collected.
        let transaction = if path.exists() {
            merge(read_transaction(&path)?, transaction.approvals())?
        } else {
            crate::signature::verify_transaction(transaction)?;
            transaction.clone()
        };
        write_transaction(&path, &transaction)
    }

    fn add_approval(&self, hash: &TransactionHash, approval: Approval) -> Result<(), Error> {
        let transaction = self.fetch(hash)?;
        let merged = merge(transaction, [approval])?;
        write_transaction(&self.path(hash), &merged)
    }

    fn fetch(&self, hash: &TransactionHash) -> Result<Transaction, Error> {
        let path = self.path(hash);
        if !path.exists() {
            return Err(Error::UnknownTransaction(*hash));
        }
        read_transaction(&path)
    }
}
