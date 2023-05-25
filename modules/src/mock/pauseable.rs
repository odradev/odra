use odra::Variable;

use crate::security::Pauseable;

#[odra::module]
pub struct PauseableCounter {
    value: Variable<u32>,
    pauseable: Pauseable
}

#[odra::module]
impl PauseableCounter {
    pub fn increment(&mut self) {
        self.pauseable.require_not_paused();
        self.raw_increment();
    }

    pub fn pause(&mut self) {
        self.pauseable.pause();
    }

    pub fn unpause(&mut self) {
        self.pauseable.unpause();
    }

    pub fn is_paused(&self) -> bool {
        self.pauseable.is_paused()
    }

    pub fn get_value(&self) -> u32 {
        self.value.get_or_default()
    }
}

impl PauseableCounter {
    fn raw_increment(&mut self) {
        self.value.set(self.value.get_or_default() + 1);
    }
}

#[cfg(test)]
mod test {
    use super::PauseableCounterDeployer;
    use crate::security::{
        errors::Error,
        events::{Paused, Unpaused}
    };
    use odra::{assert_events, test_env};

    #[test]
    fn pause_works() {
        let mut contract = PauseableCounterDeployer::default();
        let caller = test_env::get_account(0);

        assert!(!contract.is_paused());

        contract.pause();
        assert!(contract.is_paused());

        contract.unpause();
        assert!(!contract.is_paused());

        assert_events!(
            contract,
            Paused { account: caller },
            Unpaused { account: caller }
        );
    }

    #[test]
    fn increment_only_if_unpaused() {
        let mut contract = PauseableCounterDeployer::default();
        contract.increment();
        contract.pause();

        test_env::assert_exception(Error::UnpausedRequired, || contract.increment());

        assert_eq!(contract.get_value(), 1);
    }

    #[test]
    fn cannot_unpause_unpaused() {
        let mut contract = PauseableCounterDeployer::default();

        test_env::assert_exception(Error::PausedRequired, || contract.unpause());
    }

    #[test]
    fn cannot_pause_paused() {
        let mut contract = PauseableCounterDeployer::default();
        contract.pause();

        test_env::assert_exception(Error::UnpausedRequired, || contract.pause());
    }
}
