use odra::Variable;
use odra::types::Address;

#[odra::contract]
pub struct Counter {
    value: Variable<u32>,
}

#[odra::contract]
impl Counter {
    pub fn increment(&self) {
        if let Some(value) = self.value.get() {
            self.value.set(value + 1);
        } else {
            self.value.set(0);
        }
    }

    pub fn get(&self) -> u32 {
        self.value.get_or_default()
    }
}

// #[odra::contract]
// pub struct CounterDoubleCaller {
//     first: Variable<Address>,
//     second: Variable<Address>
// }

// #[odra::contract]
// impl CounterDoubleCaller {
//     pub fn register(&self, first: Address, second: Address) {
//         self.first.set(first);
//         self.second.set(second);
//     }

//     pub fn increment(&self) {
//         CounterRef{
//             address: self.first.get().unwrap()
//         }.increment()
//     }
// }

#[cfg(test)]
mod tests {
    use super::Counter;
    use odra::types::RuntimeArgs;

    #[test]
    fn incrementing() {
        let contract = Counter::deploy("counter", RuntimeArgs::new());
        assert_eq!(!contract.get(), 0);
        contract.increment();
        assert_eq!(!contract.get(), 1);
    }
    
    // #[test]
    // fn test_two_Counters() {
    //     let contract1 = Counter::deploy("counter1", RuntimeArgs::new());
    //     let contract2 = Counter::deploy("counter2", RuntimeArgs::new());
    //     assert_eq!(!contract1.get(), 0);
    //     assert_eq!(!contract2.get(), 0);
    //     contract1.increment();
    //     assert_eq!(!contract1.get(), 1);
    //     assert_eq!(!contract2.get(), 0);
    // }

    // #[test]
    // fn test_double_caller() {
    //     let Counter = Counter::deploy("Counter", RuntimeArgs::new());
    //     Counter.increment();
    //     assert_eq!(Counter.get(), 1);
    // }
}
