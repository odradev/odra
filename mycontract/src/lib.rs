use odra::types::Address;
use odra::Variable;

#[odra::contract]
pub struct Counter {
    value: Variable<u32>,
}

#[odra::contract]
impl Counter {
    pub fn init(&self) {
        self.value.set(0);
    }

    pub fn increment(&self) {
        let current = self.get();
        self.value.set(current + 1);
    }

    pub fn get(&self) -> u32 {
        self.value.get_or_default()
    }
}

#[odra::contract]
pub struct CounterDoubleCaller {
    first: Variable<Address>,
    second: Variable<Address>,
}

#[odra::contract]
impl CounterDoubleCaller {
    pub fn register(&self, first: Address, second: Address) {
        self.first.set(first);
        self.second.set(second);
    }

    pub fn increment(&self) {
        CounterRef::at(self.first.get().unwrap()).increment();
        CounterRef::at(self.second.get().unwrap()).increment();
    }
}

#[cfg(test)]
mod tests {
    use super::{Counter, CounterDoubleCaller};
    use odra::types::RuntimeArgs;

    #[test]
    fn test_incrementing() {
        let contract = Counter::deploy("counter", RuntimeArgs::new());
        assert_eq!(contract.get(), 0);
        contract.increment();
        assert_eq!(contract.get(), 1);
    }

    #[test]
    fn test_two_counters() {
        let contract1 = Counter::deploy("counter1", RuntimeArgs::new());
        let contract2 = Counter::deploy("counter2", RuntimeArgs::new());
        assert_eq!(contract1.get(), 0);
        assert_eq!(contract2.get(), 0);
        contract1.increment();
        assert_eq!(contract1.get(), 1);
        assert_eq!(contract2.get(), 0);
    }

    #[test]
    fn test_double_caller() {
        let counter = Counter::deploy("counter", RuntimeArgs::new());
        let caller1 = CounterDoubleCaller::deploy("caller1", RuntimeArgs::new());
        let caller2 = CounterDoubleCaller::deploy("caller2", RuntimeArgs::new());
        let caller3 = CounterDoubleCaller::deploy("caller3", RuntimeArgs::new());

        // caller1  ->  caller2 ->  counter
        //                      ->  counter
        //          ->  caller3 ->  counter
        //                      ->  counter
        caller1.register(caller2.address(), caller3.address());
        caller2.register(counter.address(), counter.address());
        caller3.register(counter.address(), counter.address());

        caller1.increment();
        assert_eq!(counter.get(), 4);
    }

    #[test]
    fn test_mock_vm_backend_name() {
        assert_eq!(odra::TestEnv::backend_name(), "MockVM");
    }
}
