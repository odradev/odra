use odra::{contract_env, types::Address};

use crate::access::{AccessControl, Role, DEFAULT_ADMIN_ROLE};

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
        crate::crypto::keccak256(name)
    }

    fn set_moderator_admin_role(&mut self, role: &Role) {
        self.access_control
            .set_admin_role(&Self::role(ROLE_MODERATOR), role);
    }
}

#[cfg(test)]
pub mod test {
    use super::{MockModeratedDeployer, MockModeratedRef, ROLE_MODERATOR, ROLE_MODERATOR_ADMIN};
    use crate::access::{
        errors::Error,
        events::{RoleGranted, RoleRevoked}
    };
    use odra::{assert_events, test_env, types::Address};

    #[test]
    fn deploy_works() {
        let contract = MockModeratedDeployer::init();
        let admin = test_env::get_account(0);

        assert!(contract.is_moderator(&admin));
        assert!(contract.is_admin(&admin));
    }

    #[test]
    fn add_moderators() {
        // given Admin is a moderator and an admin.
        // given User1 and User2 that are not moderators.
        let (mut contract, admin, user1, user2) = setup(false);
        assert!(!contract.is_moderator(&user1));
        assert!(!contract.is_moderator(&user2));

        // when Admin adds a moderator.
        test_env::set_caller(admin);
        contract.add_moderator(&user1);
        // then the role is granted.
        assert!(contract.is_moderator(&user1));

        // when a non-admin adds a moderator.
        test_env::assert_exception(Error::MissingRole, || {
            test_env::set_caller(user1);
            MockModeratedRef::at(contract.address()).add_moderator(&user2);
        });
        // then the User2 is not a moderator.
        assert!(!contract.is_moderator(&user2));

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
            MockModeratedRef::at(contract.address()).remove_moderator(&moderator);
        });
        // then Moderator still is a moderator.
        assert!(contract.is_moderator(&moderator));

        // when Admin revokes the Moderator's role.
        test_env::set_caller(admin);
        contract.remove_moderator(&moderator);
        // then Moderator no longer is a moderator.
        assert!(!contract.is_moderator(&moderator));

        // Re-grant the role.
        contract.add_moderator(&moderator);
        // Moderator revoke his role - fails because is not an admin.
        test_env::assert_exception(Error::MissingRole, || {
            test_env::set_caller(moderator);
            MockModeratedRef::at(contract.address()).remove_moderator(&moderator);
        });
        // then Moderator still is a moderator.
        assert!(contract.is_moderator(&moderator));

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
            MockModeratedRef::at(contract.address()).renounce_moderator_role(&moderator);
        });

        // when Moderator renounces the role.
        test_env::set_caller(moderator);
        contract.renounce_moderator_role(&moderator);
        // then is no longer a moderator.
        assert!(!contract.is_moderator(&moderator));
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
        contract.add_admin(&moderator);
        // then Moderator is an admin.
        assert!(contract.is_admin(&moderator));
        // when Moderator grants User the moderator role.
        test_env::set_caller(moderator);
        contract.add_moderator(&user);
        // then User is a moderator.
        assert!(contract.is_moderator(&user));

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
            contract.add_moderator(&user1);
            assert!(contract.is_moderator(&user1));
        }
        (contract, admin, user1, user2)
    }
}
