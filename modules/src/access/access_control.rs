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
#[odra::module]
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

pub mod mock {
    use odra::{contract_env, types::Address};

    use super::{AccessControl, Role, DEFAULT_ADMIN_ROLE};

    pub const ROLE_MODERATOR: &str = "moderator";
    pub const ROLE_MODERATOR_ADMIN: &str = "moderator_admin";

    #[odra::module]
    pub struct MockModerated {
        access_control: AccessControl
    }

    #[odra::module]
    impl MockModerated {
        #[odra(init)]
        pub fn init(&mut self) {
            let admin: Address = contract_env::caller();
            self.access_control
                .unchecked_grant_role(DEFAULT_ADMIN_ROLE, admin);
            self.access_control
                .unchecked_grant_role(Self::role(ROLE_MODERATOR), admin);
            self.access_control
                .unchecked_grant_role(Self::role(ROLE_MODERATOR_ADMIN), admin);
            self.set_moderator_admin_role(Self::role(ROLE_MODERATOR_ADMIN));
        }

        pub fn add_moderator(&mut self, moderator: Address) {
            self.access_control
                .grant_role(Self::role(ROLE_MODERATOR), moderator);
        }

        pub fn add_admin(&mut self, admin: Address) {
            self.access_control
                .grant_role(Self::role(ROLE_MODERATOR_ADMIN), admin);
        }

        pub fn remove_moderator(&mut self, moderator: Address) {
            self.access_control
                .revoke_role(Self::role(ROLE_MODERATOR), moderator);
        }

        pub fn renounce_moderator_role(&mut self, address: Address) {
            let role = Self::role(ROLE_MODERATOR);
            self.access_control.renounce_role(role, address);
        }

        pub fn is_moderator(&self, address: Address) -> bool {
            self.access_control
                .has_role(Self::role(ROLE_MODERATOR), address)
        }

        pub fn is_admin(&self, address: Address) -> bool {
            self.access_control
                .has_role(Self::role(ROLE_MODERATOR_ADMIN), address)
        }
    }

    impl MockModerated {
        fn role(name: &str) -> Role {
            crate::crypto::keccak256(name)
        }

        fn set_moderator_admin_role(&mut self, role: Role) {
            self.access_control
                .set_admin_role(Self::role(ROLE_MODERATOR), role);
        }
    }
}

#[cfg(test)]
pub mod test {
    use crate::access::{
        access_control::mock::{
            MockModeratedDeployer, MockModeratedRef, ROLE_MODERATOR, ROLE_MODERATOR_ADMIN
        },
        errors::Error,
        events::{RoleGranted, RoleRevoked}
    };
    use odra::{assert_events, test_env, types::Address};

    #[test]
    fn deploy_works() {
        let contract = MockModeratedDeployer::init();
        let admin = test_env::get_account(0);

        assert!(contract.is_moderator(admin));
        assert!(contract.is_admin(admin));
    }

    #[test]
    fn add_moderators() {
        // given Admin is a moderator and an admin.
        // given User1 and User2 that are not moderators.
        let (mut contract, admin, user1, user2) = setup(false);
        assert!(!contract.is_moderator(user1));
        assert!(!contract.is_moderator(user2));

        // when Admin adds a moderator.
        test_env::set_caller(admin);
        contract.add_moderator(user1);
        // then the role is granted.
        assert!(contract.is_moderator(user1));

        // when a non-admin adds a moderator.
        test_env::assert_exception(Error::MissingRole, || {
            test_env::set_caller(user1);
            MockModeratedRef::at(contract.address()).add_moderator(user2);
        });
        // then the User2 is not a moderator.
        assert!(!contract.is_moderator(user2));

        // then two RoleGranted events were emitted.
        assert_events!(
            contract,
            RoleGranted {
                role: crate::crypto::keccak256(ROLE_MODERATOR),
                address: user1,
                sender: admin
            }
        );
    }

    #[test]
    fn remove_moderator() {
        // given
        // Admin who is a moderator and an admin,
        // Moderator who is a moderator,
        // User who is neither an admin nor a moderator.
        let (mut contract, admin, moderator, user) = setup(true);

        // when User removes the role - it fails.
        test_env::assert_exception(Error::MissingRole, || {
            test_env::set_caller(user);
            MockModeratedRef::at(contract.address()).remove_moderator(moderator);
        });
        // then Moderator still is a moderator.
        assert!(contract.is_moderator(moderator));

        // when Admin revokes the Moderator's role.
        test_env::set_caller(admin);
        contract.remove_moderator(moderator);
        // then Moderator no longer is a moderator.
        assert!(!contract.is_moderator(moderator));

        // Re-grant the role.
        contract.add_moderator(moderator);
        // Moderator revoke his role - fails because is not an admin.
        test_env::assert_exception(Error::MissingRole, || {
            test_env::set_caller(moderator);
            MockModeratedRef::at(contract.address()).remove_moderator(moderator);
        });
        // then Moderator still is a moderator.
        assert!(contract.is_moderator(moderator));

        assert_events!(
            contract,
            RoleGranted {
                role: crate::crypto::keccak256(ROLE_MODERATOR),
                address: moderator,
                sender: admin
            },
            RoleRevoked {
                role: crate::crypto::keccak256(ROLE_MODERATOR),
                address: moderator,
                sender: admin
            },
            RoleGranted {
                role: crate::crypto::keccak256(ROLE_MODERATOR),
                address: moderator,
                sender: admin
            }
        );
    }

    #[test]
    fn renounce_moderator_role() {
        // given a user having a moderator role.
        let (mut contract, _, moderator, _) = setup(true);

        // when Admin renounces the role on moderator's behalf - it fails.
        test_env::assert_exception(Error::RoleRenounceForAnotherAddress, || {
            MockModeratedRef::at(contract.address()).renounce_moderator_role(moderator);
        });

        // when Moderator renounces the role.
        test_env::set_caller(moderator);
        contract.renounce_moderator_role(moderator);
        // then is no longer a moderator.
        assert!(!contract.is_moderator(moderator));
        // RoleRevoked event was emitted.
        assert_events!(
            contract,
            RoleRevoked {
                role: crate::crypto::keccak256(ROLE_MODERATOR),
                address: moderator,
                sender: moderator
            }
        );
    }

    #[test]
    fn add_admin() {
        // given
        // Admin who is a moderator and an admin,
        // Moderator who is a moderator,
        // User who is neither an admin nor a moderator.
        let (mut contract, admin, moderator, user) = setup(true);

        // when Admin grants Moderator the admin role.
        test_env::set_caller(admin);
        contract.add_admin(moderator);
        // then Moderator is an admin.
        assert!(contract.is_admin(moderator));
        // when Moderator grants User the moderator role.
        test_env::set_caller(moderator);
        contract.add_moderator(user);
        // then User is a moderator.
        assert!(contract.is_moderator(user));

        assert_events!(
            contract,
            RoleGranted {
                role: crate::crypto::keccak256(ROLE_MODERATOR_ADMIN),
                address: moderator,
                sender: admin
            },
            RoleGranted {
                role: crate::crypto::keccak256(ROLE_MODERATOR),
                address: user,
                sender: moderator
            }
        );
    }

    fn setup(add_moderator: bool) -> (MockModeratedRef, Address, Address, Address) {
        let mut contract = MockModeratedDeployer::init();
        // given admin who is a moderator and two users that are not moderators.
        let (admin, user1, user2) = (
            test_env::get_account(0),
            test_env::get_account(1),
            test_env::get_account(2)
        );
        if add_moderator {
            contract.add_moderator(user1);
            assert!(contract.is_moderator(user1));
        }
        (contract, admin, user1, user2)
    }
}
