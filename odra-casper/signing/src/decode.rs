use casper_types::{bytesrepr::FromBytes, Transaction, TransactionArgs, TransactionTarget};
use odra_casper_account_manager::{AccountChange, ACCOUNT_MANAGER_WASM};

use crate::build::{ARGS_MAP_KEY, TARGET_MAP_KEY};

/// The change an account manager transaction makes, read from the transaction: its session
/// code must be exactly `ACCOUNT_MANAGER_WASM` and its arguments must decode. `None` for any
/// other transaction.
pub fn decode_account_change(transaction: &Transaction) -> Option<AccountChange> {
    let Transaction::V1(v1) = transaction else {
        return None;
    };
    let fields = v1.payload().fields();
    let (target, _) = TransactionTarget::from_bytes(fields.get(&TARGET_MAP_KEY)?).ok()?;
    match target {
        TransactionTarget::Session { module_bytes, .. }
            if module_bytes.as_slice() == ACCOUNT_MANAGER_WASM => {}
        _ => return None
    }
    let (args, _) = TransactionArgs::from_bytes(fields.get(&ARGS_MAP_KEY)?).ok()?;
    match args {
        TransactionArgs::Named(args) => AccountChange::from_runtime_args(&args),
        TransactionArgs::Bytesrepr(_) => None
    }
}
