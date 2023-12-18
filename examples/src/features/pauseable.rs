use odra::prelude::*;
use odra::{ModuleWrapper, Variable};
use odra_modules::security::Pauseable;

#[odra::module]
pub struct PauseableCounter {
    value: Variable<u32>,
    pauseable: ModuleWrapper<Pauseable>
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
        let new_value = self.value.get_or_default() + 1;
        self.value.set(new_value);
    }
}

#[cfg(test)]
mod test {
    use super::PauseableCounterDeployer;
    use alloc::string::ToString;
    use odra::ToBytes;
    use odra_modules::security::errors::Error::UnpausedRequired;
    use odra_modules::security::events::{Paused, Unpaused};

    #[test]
    fn pause_works() {
        let test_env = odra::test_env();
        let mut contract = PauseableCounterDeployer::init(&test_env);
        let caller = test_env.get_account(0);

        assert!(!contract.is_paused());

        contract.pause();
        assert!(contract.is_paused());
        let last_call = contract.last_call();
        let env_last_call = test_env.last_call();
        assert_eq!(
            ["d".to_string()].to_vec(),
            contract.last_call().event_names()
        );
        assert!(contract
            .last_call()
            .emitted_event(&Paused { account: caller }));

        contract.unpause();
        assert!(!contract.is_paused());
        assert!(contract
            .last_call()
            .emitted_event(&Unpaused { account: caller }));
    }

    #[test]
    fn increment_only_if_unpaused() {
        let test_env = odra::test_env();
        let mut contract = PauseableCounterDeployer::init(&test_env);
        contract.increment();
        contract.pause();

        assert_eq!(
            contract.try_increment().unwrap_err(),
            UnpausedRequired.into()
        );
        assert_eq!(contract.get_value(), 1);
    }

    #[test]
    fn cannot_unpause_unpaused() {
        let test_env = odra::test_env();
        let mut contract = PauseableCounterDeployer::init(&test_env);

        assert_eq!(contract.try_unpause().unwrap_err(), UnpausedRequired.into());
    }

    #[test]
    fn cannot_pause_paused() {
        let test_env = odra::test_env();
        let mut contract = PauseableCounterDeployer::init(&test_env);
        contract.pause();
        assert_eq!(contract.try_pause().unwrap_err(), UnpausedRequired.into());
    }
}
