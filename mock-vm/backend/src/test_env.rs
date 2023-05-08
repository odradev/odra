//! Describes test environment API. Delegates methods to the underlying env implementation.
//!
//! Depending on the selected feature, the actual test env is dynamically loaded in the runtime or the Odra local MockVM is used.
use std::collections::HashMap;

use odra_mock_vm_types::{Address, Balance, BlockTime, BorshDeserialize, CallArgs, MockVMType};
use odra_types::{
    event::{EventError, OdraEvent},
    OdraError
};

use crate::{native_token::NativeTokenMetadata, EntrypointArgs, EntrypointCall};

macro_rules! delegate_to_env {
    (
        $(
            $(#[$outer:meta])*
            fn $func_name:ident($( $param_ident:ident : $param_ty:ty ),*) $( -> $ret:ty)*
        )+
    ) => {
        $(
            $(#[$outer])*
            pub fn $func_name( $($param_ident : $param_ty),* ) $(-> $ret)* {
                crate::borrow_env().$func_name($($param_ident),*)
            }
        )+
    }
}

delegate_to_env! {
    /// Registers the contract in the test environment.
    fn register_contract(
        constructor: Option<(String, &CallArgs, EntrypointCall)>,
        constructors: HashMap<String, (EntrypointArgs, EntrypointCall)>,
        entrypoints: HashMap<String, (EntrypointArgs, EntrypointCall)>
    ) -> Address
    /// Increases the current value of block_time.
    fn advance_block_time_by(seconds: BlockTime)
    /// Returns the backend name.
    fn get_backend_name() -> String
    /// Replaces the current caller.
    fn set_caller(address: Address)
    /// Returns the balance of the account associated with the given address.
    fn token_balance(address: Address) -> Balance
}

/// Expects the `block` execution will fail with the specific error.
pub fn assert_exception<F, E>(err: E, block: F)
where
    F: Fn() + std::panic::RefUnwindSafe,
    E: Into<OdraError>
{
    let _ = std::panic::catch_unwind(|| {
        block();
    });
    let exec_err = crate::borrow_env()
        .error()
        .expect("An error expected, but did not occur");
    assert_eq!(exec_err, err.into());
}

/// Returns nth test user account.
pub fn get_account(n: usize) -> Address {
    crate::borrow_env().get_address(n)
}

/// Returns the value that represents one native token.
pub fn one_token() -> Balance {
    Balance::one()
}

/// Calls contract at `address` invoking the `entrypoint` with `args`.
///
/// Returns optional raw bytes to further processing.
pub fn call_contract<T: MockVMType>(
    address: Address,
    entrypoint: &str,
    args: &CallArgs,
    amount: Option<Balance>
) -> T {
    crate::borrow_env().call_contract(address, entrypoint, args, amount)
}

/// Gets nth event emitted by the contract at `address`.
pub fn get_event<T: MockVMType + OdraEvent>(address: Address, index: i32) -> Result<T, EventError> {
    let bytes = crate::borrow_env().get_event(address, index);

    bytes.and_then(|bytes| {
        let event_name = extract_event_name(&bytes)?;
        if event_name == T::name() {
            T::deser(bytes).map_err(|_| EventError::Parsing)
        } else {
            Err(EventError::UnexpectedType(event_name))
        }
    })
}

/// Returns the platform native token metadata
pub fn native_token_metadata() -> NativeTokenMetadata {
    NativeTokenMetadata::new()
}

/// Returns last call gas cost.
pub fn last_call_contract_gas_cost() -> Balance {
    Balance::zero()
}

/// Returns the amount of gas paid for last call.
pub fn last_call_contract_gas_used() -> Balance {
    Balance::zero()
}

fn extract_event_name(mut bytes: &[u8]) -> Result<String, EventError> {
    let name = BorshDeserialize::deserialize(&mut bytes).map_err(|_| EventError::Formatting)?;
    Ok(name)
}
