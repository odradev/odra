use odra::{types::{Address, event::OdraEvent}, Mapping, contract_env};
use super::{errors::Error, events::{RoleAdminChanged, RoleGranted, RoleRevoked}};

pub type Role = [u8; 32];

pub const DEFAULT_ADMIN_ROLE: Role = [0u8; 32];

#[odra::module]
pub struct AccessControl {
    roles: Mapping<(Role, Address), bool>,
    role_admin: Mapping<Role, Role>,
}

#[odra::module]
impl AccessControl {

    pub fn has_role(&self, role: Role, address: Address) -> bool {
        self.roles.get(&(role, address)).unwrap_or_default()
    }

    pub fn get_role_admin(&self, role: Role) -> Role {
        let admin_role = self.role_admin.get(&role);
        if let Some(admin) = admin_role {
            admin
        } else {
            DEFAULT_ADMIN_ROLE
        }
    }

    pub fn grant_role(&mut self, role: Role, address: Address) {
        self.check_role(role, contract_env::caller());
        self.raw_grant_role(role, address);
    }

    pub fn revoke_role(&mut self, role: Role, address: Address) {
        self.check_role(role, contract_env::caller());
        self.raw_revoke_role(role, address);
    }

    pub fn renounce_role(&mut self, role: Role, address: Address) {
        if address != contract_env::caller() {
            contract_env::revert(Error::RoleRenounceForAnotherAddress);
        }
        self.raw_revoke_role(role, address);
    }
}

impl AccessControl {

    pub fn check_role(&self, role: Role, address: Address) {
        if !self.has_role(role, address) {
            contract_env::revert(Error::MissingRole);
        }
    }

    pub fn set_admin_role(&mut self, role: Role, admin_role: Role) {
        let previous_admin_role = self.get_role_admin(role);
        self.role_admin.set(&role, admin_role);
        RoleAdminChanged { role, previous_admin_role, new_admin_role: admin_role }.emit();
    }

    pub fn raw_grant_role(&mut self, role: Role, address: Address) {
        if !self.has_role(role, address) {
            self.roles.set(&(role, address), true);
            RoleGranted { role, address, sender: contract_env::caller() }.emit();
        }
    }

    pub fn raw_revoke_role(&mut self, role: Role, address: Address) {
        if self.has_role(role, address) {
            self.roles.set(&(role, address), false);
            RoleRevoked { role, address, sender: contract_env::caller() }.emit();
        }
    }
}