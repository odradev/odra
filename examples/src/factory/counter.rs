use odra::prelude::*;

#[odra::module(factory=on)]
pub struct Counter {
    /// The initial value for the counter.
    value: Var<u32>
}

#[odra::module(factory=on)]
impl Counter {
    pub fn init(&mut self, value: u32) {
        self.value.set(value);
    }

    pub fn increment(&mut self) {
        self.value.set(self.value.get_or_default() + 1);
    }

    pub fn value(&self) -> u32 {
        self.value.get_or_default()
    }
}

#[cfg(test)]
mod tests {
    use odra::{
        casper_types::RuntimeArgs,
        host::{Deployer, FactoryUpgradeArgs, HostRef, InstallConfig, NoArgs},
        prelude::*
    };

    use super::{
        Counter, CounterFactory, CounterFactoryContractDeployed, CounterHostRef, CounterInitArgs
    };

    #[test]
    fn test_standalone_module() {
        let env = odra_test::env();
        let mut counter_ref = Counter::deploy(&env, CounterInitArgs { value: 1 });
        assert_eq!(counter_ref.value(), 1);
        counter_ref.increment();
        assert_eq!(counter_ref.value(), 2);
    }

    #[test]
    #[ignore = "This test does not work on odra vm"]
    fn test_factory() {
        let env = odra_test::env();
        // Deploy the factory contract
        let mut factory_ref = CounterFactory::deploy(&env, NoArgs);
        // Use the factory to deploy a new Counter contract with initial value 10
        let (address, _access_uref) = factory_ref.factory(String::from("Counter"), 10);
        assert!(env.emitted_event(
            &factory_ref,
            CounterFactoryContractDeployed {
                contract_address: address,
                contract_name: String::from("Counter")
            }
        ));
        // Interact with the newly deployed Counter contract
        let mut counter_ref = CounterHostRef::new(address, env);
        // Increment the counter
        counter_ref.increment();
        // The value should now be 11
        assert_eq!(counter_ref.value(), 11);
    }

    #[test]
    fn test_factory_upgrade() {
        let env = odra_test::env();
        // Deploy the factory contract
        let factory_ref = CounterFactory::deploy_with_cfg(
            &env,
            NoArgs,
            InstallConfig::upgradable::<CounterFactory>()
        );
        let args = FactoryUpgradeArgs {
            default_args: RuntimeArgs::new(),
            names_to_upgrade: vec![],
            ..Default::default()
        };
        // Upgrade the factory contract
        let result = CounterFactory::try_upgrade(&env, factory_ref.address(), args);
        assert!(result.is_ok());
    }
}
