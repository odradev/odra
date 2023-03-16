//! Whitelist-based access control system.

use super::{
    errors::Error,
    events::{AddedToAllowList, RemovedFromAllowList}
};
use odra::{
    contract_env,
    types::{event::OdraEvent, Address},
    Mapping
};

/// This module allows limiting access to certain features of other modules.
///
/// Accounts can be added or removed to the list dynamically using the [`add()`] and [`remove()`] functions,
/// respectively.
///
/// To make sure only privileged account can access the feature, call `ensure_allowed()` function.
#[odra::module]
pub struct AllowList {
    allow_list: Mapping<Address, bool>
}

#[odra::module]
impl AllowList {
    /// Adds `address` to the allow list.
    pub fn add(&mut self, address: Address) {
        self.allow_list.set(&address, true);
        AddedToAllowList {
            address,
            sender: contract_env::caller()
        }
        .emit();
    }

    /// Removes `address` from the allow list.
    pub fn remove(&mut self, address: Address) {
        self.allow_list.set(&address, false);
        RemovedFromAllowList {
            address,
            sender: contract_env::caller()
        }
        .emit();
    }

    /// Asserts the caller is on the list. Reverts otherwise.
    pub fn ensure_allowed(&self) {
        if !self.is_allowed(contract_env::caller()) {
            contract_env::revert(Error::NotAllowed);
        }
    }

    /// Returns true if `address` is on the list.
    pub fn is_allowed(&self, address: Address) -> bool {
        self.allow_list.get_or_default(&address)
    }
}

/// This module allows limiting access to certain features of other modules. The module is initialized
/// with a single address added to the list.
///
/// Accounts can be added or removed to the list dynamically using the [`add()`] and [`remove()`] functions,
/// respectively. The caller of these function must be already added to the list.
///
/// To make sure only privileged account can access the feature, call `ensure_allowed()` function.
#[odra::module]
pub struct ProtectedAllowList {
    list: AllowList
}

#[odra::module]
impl ProtectedAllowList {
    /// Inits the list with a given `address`.
    #[odra(init)]
    pub fn init(&mut self, address: Address) {
        self.list.add(address);
    }

    /// Adds new `address` to the allow list.
    pub fn add(&mut self, address: Address) {
        self.ensure_allowed();
        self.list.add(address);
    }

    /// Removes an `address` from the allow list.
    pub fn remove(&mut self, address: Address) {
        self.ensure_allowed();
        self.list.remove(address);
    }

    /// Asserts the caller is on the list. Revert otherwise.
    pub fn ensure_allowed(&self) {
        self.list.ensure_allowed()
    }

    /// Returns true if `address` is on the list.
    pub fn is_allowed(&self, address: Address) -> bool {
        self.list.is_allowed(address)
    }
}

#[cfg(test)]
mod test {
    use odra::{assert_events, test_env};

    use crate::access::{
        errors::Error,
        events::{AddedToAllowList, RemovedFromAllowList},
        ProtectedAllowListRef
    };

    use super::{AllowListDeployer, ProtectedAllowListDeployer};

    #[test]
    fn allow_list_works() {
        let mut contract = AllowListDeployer::default();
        let account = test_env::get_account(0);

        contract.add(account);
        assert!(contract.is_allowed(account));

        contract.remove(account);
        assert!(!contract.is_allowed(account));

        test_env::assert_exception(Error::NotAllowed, || contract.ensure_allowed());

        assert_events!(
            contract,
            AddedToAllowList {
                address: account,
                sender: account
            },
            RemovedFromAllowList {
                address: account,
                sender: account
            }
        );
    }

    #[test]
    fn protected_allow_list_works() {
        let (address1, address2, address3) = (
            test_env::get_account(0),
            test_env::get_account(1),
            test_env::get_account(2)
        );
        let mut contract = ProtectedAllowListDeployer::init(address1);

        assert!(contract.is_allowed(address1));

        test_env::assert_exception(Error::NotAllowed, || {
            test_env::set_caller(address2);
            ProtectedAllowListRef::at(contract.address()).add(address3);
        });

        test_env::set_caller(address1);
        contract.add(address3);

        assert!(contract.is_allowed(address3));

        test_env::assert_exception(Error::NotAllowed, || {
            test_env::set_caller(address2);
            ProtectedAllowListRef::at(contract.address()).remove(address3);
        });

        test_env::set_caller(address1);
        contract.remove(address3);

        assert!(!contract.is_allowed(address3));

        assert_events!(
            contract,
            AddedToAllowList {
                address: address1,
                sender: address1
            },
            AddedToAllowList {
                address: address3,
                sender: address1
            },
            RemovedFromAllowList {
                address: address3,
                sender: address1
            }
        );
    }
}
