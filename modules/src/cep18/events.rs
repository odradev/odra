use odra::casper_types::U256;
use odra::prelude::*;
use odra::Address;

use crate::cep18::utils::SecurityBadge;

#[odra::event]
pub struct Mint {
    pub recipient: Address,
    pub amount: U256
}

#[odra::event]
pub struct Burn {
    pub owner: Address,
    pub amount: U256
}

#[odra::event]
pub struct SetAllowance {
    pub owner: Address,
    pub spender: Address,
    pub allowance: U256
}

#[odra::event]
pub struct IncreaseAllowance {
    pub owner: Address,
    pub spender: Address,
    pub allowance: U256,
    pub inc_by: U256
}

#[odra::event]
pub struct DecreaseAllowance {
    pub owner: Address,
    pub spender: Address,
    pub allowance: U256,
    pub decr_by: U256
}

#[odra::event]
pub struct Transfer {
    pub sender: Address,
    pub recipient: Address,
    pub amount: U256
}

#[odra::event]
pub struct TransferFrom {
    pub spender: Address,
    pub owner: Address,
    pub recipient: Address,
    pub amount: U256
}

#[odra::event]
pub struct ChangeSecurity {
    pub admin: Address,
    pub sec_change_map: BTreeMap<Address, SecurityBadge>
}
