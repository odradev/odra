use super::{
    errors::Error,
    events::{RoleAdminChanged, RoleGranted, RoleRevoked}
};
use odra::{
    contract_env,
    types::{event::OdraEvent, Address},
    Mapping
};

pub type Role = [u8; 32];

pub const DEFAULT_ADMIN_ROLE: Role = [0u8; 32];

/// This contract module enables the implementation of role-based access control mechanisms for children
/// modules.
///
/// Roles are identified by their 32-bytes identifier, which should be unique and exposed in the external API.
///
/// Roles can be used to represent a set of permissions, and the hasRole function is used to restrict
/// access to a function call.
///
/// Roles can be granted and revoked dynamically using the [`grant_role()`] and [`revoke_role()`] functions,
/// respectively. Each role has an associated admin role, and only accounts that have the role's admin role
/// can call grant_role and revoke_role.
///
/// By default, the admin role for all roles is [`DEFAULT_ADMIN_ROLE`], which means that only accounts with
/// this role can grant or revoke other roles.
///
/// More complex role relationships can be established using the `set_role_admin()` function.
#[odra::module(events = [RoleAdminChanged, RoleGranted, RoleRevoked])]
pub struct AccessControl {
    roles: Mapping<(Role, Address), bool>,
    role_admin: Mapping<Role, Role>
}

#[odra::module]
impl AccessControl {
    /// Returns true if account has been granted `role`.
    pub fn has_role(&self, role: Role, address: Address) -> bool {
        self.roles.get(&(role, address)).unwrap_or_default()
    }

    /// Returns the admin role that controls `role`.
    ///
    /// The admin role may be changed using [set_admin_role()](Self::set_admin_role()).
    pub fn get_role_admin(&self, role: Role) -> Role {
        let admin_role = self.role_admin.get(&role);
        if let Some(admin) = admin_role {
            admin
        } else {
            DEFAULT_ADMIN_ROLE
        }
    }

    /// Grants `role` to `address`.
    ///
    /// If the role has been already granted - nothing happens,
    /// otherwise [`RoleGranted`] event is emitted.
    ///
    /// The caller must have `role`'s admin role.
    pub fn grant_role(&mut self, role: Role, address: Address) {
        self.check_role(self.get_role_admin(role), contract_env::caller());
        self.unchecked_grant_role(role, address);
    }

    /// Grants `role` to `address`.
    ///
    /// If the role has been already revoked - nothing happens,
    /// otherwise [`RoleRevoked`] event is emitted.
    ///
    /// The caller must have `role`'s admin role.
    pub fn revoke_role(&mut self, role: Role, address: Address) {
        self.check_role(self.get_role_admin(role), contract_env::caller());
        self.unchecked_revoke_role(role, address);
    }

    /// The function is used to remove a role from the account that initiated the call.
    ///
    /// One common way of managing roles is by using [`grant_role()`](Self::grant_role())
    /// and [`revoke_role()`](Self::revoke_role()).
    /// The purpose of revokeRole is to provide a mechanism for revoking privileges from an account
    /// in case it gets compromised.
    ///
    /// If the account had previously been granted the role, the function will trigger a `RoleRevoked` event.
    ///
    /// Note that only `address` is authorized to call this function.
    pub fn renounce_role(&mut self, role: Role, address: Address) {
        if address != contract_env::caller() {
            contract_env::revert(Error::RoleRenounceForAnotherAddress);
        }
        self.unchecked_revoke_role(role, address);
    }
}

impl AccessControl {
    /// Ensures `address` has `role`. If not, reverts with [Error::MissingRole].
    pub fn check_role(&self, role: Role, address: Address) {
        if !self.has_role(role, address) {
            contract_env::revert(Error::MissingRole);
        }
    }

    /// Sets `admin_role` as `role`'s admin role.
    ///
    /// Emits a `RoleAdminChanged` event.
    pub fn set_admin_role(&mut self, role: Role, admin_role: Role) {
        let previous_admin_role = self.get_role_admin(role);
        self.role_admin.set(&role, admin_role);
        RoleAdminChanged {
            role,
            previous_admin_role,
            new_admin_role: admin_role
        }
        .emit();
    }

    /// Grants `role` to `address`.
    ///
    /// Internal function without access restriction.
    /// This function should be used to setup the initial access control.
    ///
    /// May emit a `RoleGranted` event.
    pub fn unchecked_grant_role(&mut self, role: Role, address: Address) {
        if !self.has_role(role, address) {
            self.roles.set(&(role, address), true);
            RoleGranted {
                role,
                address,
                sender: contract_env::caller()
            }
            .emit();
        }
    }

    /// Revokes `role` from `address`.
    ///
    /// Internal function without access restriction.
    /// This function should be used to setup the initial access control.
    ///
    /// May emit a `RoleRevoked` event.
    pub fn unchecked_revoke_role(&mut self, role: Role, address: Address) {
        if self.has_role(role, address) {
            self.roles.set(&(role, address), false);
            RoleRevoked {
                role,
                address,
                sender: contract_env::caller()
            }
            .emit();
        }
    }
}
