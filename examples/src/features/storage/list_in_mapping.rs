//! Module containing ScheduleContract. It is used in docs to explain how to store a `List`
//! as a value of a `Mapping`.
use odra::prelude::*;

/// A simple contract that keeps a schedule of task ids per address.
#[odra::module]
pub struct ScheduleContract {
    schedules: Mapping<Address, List<u32>>
}

#[odra::module]
impl ScheduleContract {
    /// Adds a task id to the caller's schedule.
    pub fn add_task(&mut self, owner: &Address, id: u32) {
        self.schedules.list(owner).push(id);
    }

    /// Returns the number of tasks scheduled for the given address.
    pub fn task_count(&self, owner: &Address) -> u32 {
        self.schedules.list(owner).len()
    }

    /// Returns the n-th task id scheduled for the given address.
    pub fn task_at(&self, owner: &Address, index: u32) -> Option<u32> {
        self.schedules.list(owner).get(index)
    }
}

#[cfg(test)]
mod tests {
    use super::ScheduleContract;
    use odra::host::{Deployer, NoArgs};

    #[test]
    fn unseen_key_has_empty_list() {
        let test_env = odra_test::env();
        let contract = ScheduleContract::deploy(&test_env, NoArgs);
        let owner = test_env.get_account(0);

        assert_eq!(contract.task_count(&owner), 0);
        assert_eq!(contract.task_at(&owner, 0), None);
    }

    #[test]
    fn push_then_read_back() {
        let test_env = odra_test::env();
        let mut contract = ScheduleContract::deploy(&test_env, NoArgs);
        let owner = test_env.get_account(0);

        contract.add_task(&owner, 42);
        contract.add_task(&owner, 43);

        assert_eq!(contract.task_count(&owner), 2);
        assert_eq!(contract.task_at(&owner, 0), Some(42));
        assert_eq!(contract.task_at(&owner, 1), Some(43));
    }

    #[test]
    fn different_keys_get_independent_lists() {
        let test_env = odra_test::env();
        let mut contract = ScheduleContract::deploy(&test_env, NoArgs);
        let alice = test_env.get_account(0);
        let bob = test_env.get_account(1);

        // Only alice gets tasks - bob's list must stay empty.
        contract.add_task(&alice, 1);
        contract.add_task(&alice, 2);
        contract.add_task(&alice, 3);

        assert_eq!(contract.task_count(&alice), 3);
        assert_eq!(contract.task_count(&bob), 0);
        assert_eq!(contract.task_at(&bob, 0), None);

        // Now give bob different values and make sure alice's list is untouched
        // and the two lists never overlap in storage.
        contract.add_task(&bob, 100);

        assert_eq!(contract.task_count(&alice), 3);
        assert_eq!(contract.task_count(&bob), 1);
        assert_eq!(contract.task_at(&alice, 0), Some(1));
        assert_eq!(contract.task_at(&alice, 1), Some(2));
        assert_eq!(contract.task_at(&alice, 2), Some(3));
        assert_eq!(contract.task_at(&bob, 0), Some(100));
    }
}
