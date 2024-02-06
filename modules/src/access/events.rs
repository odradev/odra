use super::access_control::Role;
use odra::prelude::*;
use odra::{Address, Event};

#[derive(Event, PartialEq, Eq, Debug)]
pub struct OwnershipTransferred {
    pub previous_owner: Option<Address>,
    pub new_owner: Option<Address>
}

#[derive(Event, PartialEq, Eq, Debug)]
pub struct OwnershipTransferStarted {
    pub previous_owner: Option<Address>,
    pub new_owner: Option<Address>
}

/// Informs `new_admin_role` is set as `role`'s admin role, replacing `previous_admin_role`.
///
/// [`DEFAULT_ADMIN_ROLE`](super::access_control::DEFAULT_ADMIN_ROLE) is the starting admin for all roles,
/// but `RoleAdminChanged` not being emitted signaling this.
#[derive(Event, PartialEq, Eq, Debug)]
pub struct RoleAdminChanged {
    pub role: Role,
    pub previous_admin_role: Role,
    pub new_admin_role: Role
}

/// Informs `address` is granted `role`.
#[derive(Event, PartialEq, Eq, Debug)]
pub struct RoleGranted {
    pub role: Role,
    pub address: Address,
    pub sender: Address
}

/// Informs `address` is revoked `role`.
#[derive(Event, PartialEq, Eq, Debug)]
pub struct RoleRevoked {
    pub role: Role,
    pub address: Address,
    pub sender: Address
}
