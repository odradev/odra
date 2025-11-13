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

#[odra::module(factory=on)]
pub struct BetterCounter {
    /// The initial value for the counter.
    value: Var<u32>
}

#[odra::module(factory=on)]
impl BetterCounter {
    pub fn init(&mut self, value: u32) {
        self.value.set(value);
    }

    pub fn increment(&mut self) {
        self.value.set(self.value.get_or_default() + 1);
    }

    pub fn value(&self) -> u32 {
        self.value.get_or_default()
    }

    pub fn upgrade(&mut self, new_value: u32) {
        self.value.set(new_value);
    }
}

#[cfg(test)]
mod tests {
    use odra::{
        casper_types::{
            bytesrepr::{Bytes, ToBytes},
            runtime_args
        },
        host::{Deployer, HostRef, InstallConfig, NoArgs},
        prelude::*
    };

    use crate::factory::counter::BetterCounterFactory;

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
    // #[ignore = "This test does not work on odra vm"]
    fn test_factory() {
        let env = odra_test::env();
        // Deploy the factory contract
        let mut factory_ref = CounterFactory::deploy(&env, NoArgs);
        // Use the factory to deploy a new Counter contract with initial value 10
        let (address, _access_uref) = factory_ref.new_contract(String::from("Counter"), 10);
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
        let mut factory = CounterFactory::deploy_with_cfg(
            &env,
            NoArgs,
            InstallConfig::upgradable::<CounterFactory>()
        );
        let (ten_address, _) = factory.new_contract(String::from("FromTen"), 10);
        let (two_address, _) = factory.new_contract(String::from("FromTwo"), 2);
        let (three_address, _) = factory.new_contract(String::from("FromThree"), 3);
        let (hundred_address, _) = factory.new_contract(String::from("FromHundred"), 100);

        // Upgrade the factory contract
        let result = BetterCounterFactory::try_upgrade(&env, factory.address(), NoArgs);
        assert!(result.is_ok());

        let mut factory = result.unwrap();
        factory.upgrade_child_contract(String::from("FromTen"), 122);
        env.set_caller(env.get_account(11));
        factory.upgrade_child_contract(String::from("FromTwo"), 11);

        factory.batch_upgrade_child_contract(
            Bytes::from(
                runtime_args! {
                    "new_value" => 42u32
                }
                .to_bytes()
                .expect("Failed to serialize runtime args for default_args")
            ),
            Bytes::from(
                vec![
                    "FromTwo".to_string(),
                    "FromThree".to_string(),
                    "FromHundred".to_string(),
                ]
                .to_bytes()
                .expect("Failed to serialize runtime args for names_to_upgrade")
            ),
            Bytes::from(
                BTreeMap::from([(
                    "FromHundred".to_string(),
                    runtime_args! {
                        "new_value" => 1000u32
                    }
                )])
                .to_bytes()
                .expect("Failed to serialize runtime args for specific_args")
            )
        );

        assert_eq!(CounterHostRef::new(ten_address, env.clone()).value(), 122);
        assert_eq!(CounterHostRef::new(two_address, env.clone()).value(), 42);
        assert_eq!(CounterHostRef::new(three_address, env.clone()).value(), 42);
        assert_eq!(
            CounterHostRef::new(hundred_address, env.clone()).value(),
            1000
        );
    }
}
