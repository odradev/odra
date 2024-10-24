use odra::casper_types::U256;
use odra::prelude::*;

use crate::cep18::utils::SecurityBadge;

/// An event emitted when a mint operation is performed.
#[odra::event]
pub struct Mint {
    /// The recipient of the minted tokens.
    pub recipient: Address,
    /// The amount of tokens minted.
    pub amount: U256
}

/// An event emitted when a burn operation is performed.
#[odra::event]
pub struct Burn {
    /// The owner of the tokens that are burned.
    pub owner: Address,
    /// The amount of tokens burned.
    pub amount: U256
}

/// An event emitted when an allowance is set.
#[odra::event]
pub struct SetAllowance {
    /// The owner of the tokens.
    pub owner: Address,
    /// The spender that is allowed to spend the tokens.
    pub spender: Address,
    /// The allowance amount.
    pub allowance: U256
}

/// An event emitted when an allowance is increased.
#[odra::event]
pub struct IncreaseAllowance {
    /// The owner of the tokens.
    pub owner: Address,
    /// The spender that is allowed to spend the tokens.
    pub spender: Address,
    /// The final allowance amount.
    pub allowance: U256,
    /// The amount by which the allowance was increased.
    pub inc_by: U256
}

/// An event emitted when an allowance is decreased.
#[odra::event]
pub struct DecreaseAllowance {
    /// The owner of the tokens.
    pub owner: Address,
    /// The spender that is allowed to spend the tokens.
    pub spender: Address,
    /// The final allowance amount.
    pub allowance: U256,
    /// The amount by which the allowance was decreased.
    pub decr_by: U256
}

/// An event emitted when a transfer is performed.
#[odra::event]
pub struct Transfer {
    /// The sender of the tokens.
    pub sender: Address,
    /// The recipient of the tokens.
    pub recipient: Address,
    /// The amount of tokens transferred.
    pub amount: U256
}

/// An event emitted when a transfer_from is performed.
#[odra::event]
pub struct TransferFrom {
    /// The spender that is allowed to spend the tokens.
    pub spender: Address,
    /// The sender of the tokens.
    pub owner: Address,
    /// The recipient of the tokens.
    pub recipient: Address,
    /// The amount of tokens transferred.
    pub amount: U256
}

/// An event emitted when a security rules change is performed.
#[odra::event]
pub struct ChangeSecurity {
    /// The address of the administrator which perfomed the change.
    pub admin: Address,
    /// The map of changes made to the security rules.
    pub sec_change_map: BTreeMap<Address, SecurityBadge>
}
