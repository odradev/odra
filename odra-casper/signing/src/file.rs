use std::{fs, path::Path};

use casper_types::Transaction;

use crate::Error;

/// Reads a transaction file and checks its hash and every signature in it. A transaction
/// without approvals yet is accepted.
pub fn read_transaction(path: &Path) -> Result<Transaction, Error> {
    let json = fs::read_to_string(path).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source
    })?;
    let transaction: Transaction =
        serde_json::from_str(&json).map_err(|error| Error::NotATransaction {
            path: path.to_path_buf(),
            reason: error.to_string()
        })?;
    crate::signature::verify_transaction(&transaction)?;
    Ok(transaction)
}

/// Writes the transaction as pretty-printed JSON. The file is written next to its final place
/// and renamed, so a reader never sees half of it.
pub fn write_transaction(path: &Path, transaction: &Transaction) -> Result<(), Error> {
    let json =
        serde_json::to_string_pretty(transaction).map_err(|error| Error::NotATransaction {
            path: path.to_path_buf(),
            reason: error.to_string()
        })?;
    let io = |source| Error::Io {
        path: path.to_path_buf(),
        source
    };
    let temp = path.with_extension("tmp");
    fs::write(&temp, json).map_err(io)?;
    fs::rename(&temp, path).map_err(io)
}
