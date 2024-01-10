use odra::prelude::*;
use odra::{Address, Module, ModuleWrapper};
use sha3::{Digest, Keccak256};

use odra_modules::access::{AccessControl, Role, DEFAULT_ADMIN_ROLE};

pub const ROLE_MODERATOR: &str = "moderator";
pub const ROLE_MODERATOR_ADMIN: &str = "moderator_admin";

#[odra::module]
pub struct MockModerated {
    access_control: ModuleWrapper<AccessControl>
}

#[odra::module]
impl MockModerated {
    pub fn init(&mut self) {
        let admin = self.env().caller();
        self.access_control
            .unchecked_grant_role(&DEFAULT_ADMIN_ROLE, &admin);
        self.access_control
            .unchecked_grant_role(&Self::role(ROLE_MODERATOR), &admin);
        self.access_control
            .unchecked_grant_role(&Self::role(ROLE_MODERATOR_ADMIN), &admin);
        self.set_moderator_admin_role(&Self::role(ROLE_MODERATOR_ADMIN));
    }

    pub fn add_moderator(&mut self, moderator: &Address) {
        self.access_control
            .grant_role(&Self::role(ROLE_MODERATOR), moderator);
    }

    pub fn add_admin(&mut self, admin: &Address) {
        self.access_control
            .grant_role(&Self::role(ROLE_MODERATOR_ADMIN), admin);
    }

    pub fn remove_moderator(&mut self, moderator: &Address) {
        self.access_control
            .revoke_role(&Self::role(ROLE_MODERATOR), moderator);
    }

    pub fn renounce_moderator_role(&mut self, address: &Address) {
        let role = Self::role(ROLE_MODERATOR);
        self.access_control.renounce_role(&role, address);
    }

    pub fn is_moderator(&self, address: &Address) -> bool {
        self.access_control
            .has_role(&Self::role(ROLE_MODERATOR), address)
    }

    pub fn is_admin(&self, address: &Address) -> bool {
        self.access_control
            .has_role(&Self::role(ROLE_MODERATOR_ADMIN), address)
    }
}

impl MockModerated {
    fn role(name: &str) -> Role {
        keccak_256(name)
    }

    fn set_moderator_admin_role(&mut self, role: &Role) {
        self.access_control
            .set_admin_role(&Self::role(ROLE_MODERATOR), role);
    }
}

pub fn keccak_256(input: &str) -> Role {
    let mut hasher = Keccak256::default();
    hasher.update(input.as_bytes());
    hasher.finalize().into()
}

#[cfg(test)]
pub mod test {
    use super::{
        keccak_256, MockModeratedDeployer, MockModeratedHostRef, ROLE_MODERATOR,
        ROLE_MODERATOR_ADMIN
    };
    use odra::Address;
    use odra_modules::access::{
        errors::Error,
        events::{RoleGranted, RoleRevoked}
    };

    #[test]
    fn deploy_works() {
        let env = odra::test_env();
        let contract = MockModeratedDeployer::init(&env);
        let admin = env.get_account(0);

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
        contract.env().set_caller(admin);
        contract.add_moderator(user1);
        // then the role is granted.
        assert!(contract.is_moderator(user1));

        // when a non-admin adds a moderator.
        contract.env().set_caller(user1);
        let err = contract.try_add_moderator(user2).unwrap_err();
        assert_eq!(err, Error::MissingRole.into());

        // then the User2 is not a moderator.
        assert!(!contract.is_moderator(user2));

        // then two RoleGranted events were emitted.
        contract.env().emitted_event(
            &contract.address(),
            &RoleGranted {
                role: keccak_256(ROLE_MODERATOR),
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
        contract.env().set_caller(user);
        assert_eq!(
            contract.try_remove_moderator(moderator).unwrap_err(),
            Error::MissingRole.into()
        );
        // then Moderator still is a moderator.
        assert!(contract.is_moderator(moderator));

        // when Admin revokes the Moderator's role.
        contract.env().set_caller(admin);
        contract.remove_moderator(moderator);
        // then Moderator no longer is a moderator.
        assert!(!contract.is_moderator(moderator));

        // Re-grant the role.
        contract.add_moderator(moderator);
        // Moderator revoke his role - fails because is not an admin.
        contract.env().set_caller(moderator);
        assert_eq!(
            contract.try_remove_moderator(moderator).unwrap_err(),
            Error::MissingRole.into()
        );
        // then Moderator still is a moderator.
        assert!(contract.is_moderator(moderator));

        contract.env().emitted_event(
            &contract.address(),
            &RoleGranted {
                role: keccak_256(ROLE_MODERATOR),
                address: moderator,
                sender: admin
            }
        );
        contract.env().emitted_event(
            &contract.address(),
            &RoleRevoked {
                role: keccak_256(ROLE_MODERATOR),
                address: moderator,
                sender: admin
            }
        );
        contract.env().emitted_event(
            &contract.address(),
            &RoleGranted {
                role: keccak_256(ROLE_MODERATOR),
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
        assert_eq!(
            contract.try_renounce_moderator_role(moderator).unwrap_err(),
            Error::RoleRenounceForAnotherAddress.into()
        );

        // when Moderator renounces the role.
        contract.env().set_caller(moderator);
        contract.renounce_moderator_role(moderator);
        // then is no longer a moderator.
        assert!(!contract.is_moderator(moderator));
        // RoleRevoked event was emitted.
        contract.env().emitted_event(
            &contract.address(),
            &RoleRevoked {
                role: keccak_256(ROLE_MODERATOR),
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
        contract.env().set_caller(admin);
        contract.add_admin(moderator);
        // then Moderator is an admin.
        assert!(contract.is_admin(moderator));
        // when Moderator grants User the moderator role.
        contract.env().set_caller(moderator);
        contract.add_moderator(user);
        // then User is a moderator.
        assert!(contract.is_moderator(user));

        contract.env().emitted_event(
            &contract.address(),
            &RoleGranted {
                role: keccak_256(ROLE_MODERATOR_ADMIN),
                address: moderator,
                sender: admin
            }
        );
        contract.env().emitted_event(
            &contract.address(),
            &RoleGranted {
                role: keccak_256(ROLE_MODERATOR),
                address: user,
                sender: moderator
            }
        );
    }

    fn setup(add_moderator: bool) -> (MockModeratedHostRef, Address, Address, Address) {
        let env = odra::test_env();
        let mut contract = MockModeratedDeployer::init(&env);
        // given admin who is a moderator and two users that are not moderators.
        let (admin, user1, user2) = (env.get_account(0), env.get_account(1), env.get_account(2));
        if add_moderator {
            contract.add_moderator(user1);
            assert!(contract.is_moderator(user1));
        }
        (contract, admin, user1, user2)
    }
}
