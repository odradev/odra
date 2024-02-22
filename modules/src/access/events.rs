//! Events emitted by the AccessControl module.
use super::access_control::Role;
use odra::prelude::*;
use odra::{Address, Event};

/// Emitted when ownership of the contract is transferred.
#[derive(Event, PartialEq, Eq, Debug)]
pub struct OwnershipTransferred {
    /// The previous owner.
    pub previous_owner: Option<Address>,
    /// The new owner.
    pub new_owner: Option<Address>
}

/// Emitted when the ownership transfer is started.
#[derive(Event, PartialEq, Eq, Debug)]
pub struct OwnershipTransferStarted {
    /// The previous owner.
    pub previous_owner: Option<Address>,
    /// The new owner.
    pub new_owner: Option<Address>
}

/// Informs `new_admin_role` is set as `role`'s admin role, replacing `previous_admin_role`.
///
/// [`DEFAULT_ADMIN_ROLE`](super::access_control::DEFAULT_ADMIN_ROLE) is the starting admin for all roles,
/// but `RoleAdminChanged` not being emitted signaling this.
#[derive(Event, PartialEq, Eq, Debug)]
pub struct RoleAdminChanged {
    /// The role whose admin role is changed.
    pub role: Role,
    /// The previous admin role.
    pub previous_admin_role: Role,
    /// The new admin role.
    pub new_admin_role: Role
}

/// Informs `address` is granted `role`.
#[derive(Event, PartialEq, Eq, Debug)]
pub struct RoleGranted {
    /// The role granted.
    pub role: Role,
    /// The address granted the role.
    pub address: Address,
    /// The address that granted the role.
    pub sender: Address
}

/// Informs `address` is revoked `role`.
#[derive(Event, PartialEq, Eq, Debug)]
pub struct RoleRevoked {
    /// The role revoked.
    pub role: Role,
    /// The address revoked the role.
    pub address: Address,
    /// The address that revoked the role.
    pub sender: Address
}
