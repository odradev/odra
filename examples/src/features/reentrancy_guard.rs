use odra::prelude::*;
use odra::Var;

/// Contract used to test reentrancy guard.
#[odra::module]
pub struct ReentrancyMock {
    counter: Var<u32>
}

#[odra::module]
impl ReentrancyMock {
    /// Simple recursive function that counts to `n`.
    #[odra(non_reentrant)]
    pub fn count_local_recursive(&mut self, n: u32) {
        if n > 0 {
            self.count();
            self.count_local_recursive(n - 1);
        }
    }

    /// Recursive function that counts to `n` using a reference to the contract.
    #[odra(non_reentrant)]
    pub fn count_ref_recursive(&mut self, n: u32) {
        if n > 0 {
            self.count();
            ReentrancyMockContractRef::new(self.env(), self.env().self_address())
                .count_ref_recursive(n - 1);
        }
    }

    /// Recursive function that counts to `n` and is protected.
    #[odra(non_reentrant)]
    pub fn non_reentrant_count(&mut self) {
        self.count();
    }

    /// Returns the value of the counter.
    pub fn get_value(&self) -> u32 {
        self.counter.get_or_default()
    }
}

impl ReentrancyMock {
    fn count(&mut self) {
        let c = self.counter.get_or_default();
        self.counter.set(c + 1);
    }
}

#[cfg(test)]
mod test {
    use super::ReentrancyMockHostRef;
    use odra::host::{Deployer, NoArgs};

    #[test]
    fn non_reentrant_function_can_be_called() {
        let mut contract = ReentrancyMockHostRef::deploy(&odra_test::env(), NoArgs);
        assert_eq!(contract.get_value(), 0);
        contract.non_reentrant_count();
        assert_eq!(contract.get_value(), 1);
    }

    #[test]
    fn ref_recursion_not_allowed() {
        let mut contract = ReentrancyMockHostRef::deploy(&odra_test::env(), NoArgs);
        assert_eq!(
            contract.try_count_ref_recursive(11).unwrap_err(),
            odra::ExecutionError::ReentrantCall.into()
        );
    }

    #[test]
    fn local_recursion_allowed() {
        let mut contract = ReentrancyMockHostRef::deploy(&odra_test::env(), NoArgs);
        contract.count_local_recursive(11);
        assert_eq!(contract.get_value(), 11);
    }
}
