use odra::{types::{event::OdraEvent}, Variable, contract_env};

#[derive(odra::Event)]
struct ValueUpdated {
    pub old_value: u32,
    pub new_value: u32,
    pub operator: odra::types::Address
}

#[derive(odra::Event)]
struct Init {
    pub value: u32,
}

#[odra::module]
pub struct Counter {
    pub value: Variable<u32>
}

#[odra::module]
impl Counter {
    #[odra(init)]
    pub fn init(&mut self, value: u32) {
        self.value.set(value);
        <Init as OdraEvent>::emit(Init {
            value,
        });
    }

    pub fn add(&mut self, value: u32) {
        self.value.set(self.value.get_or_default() + value);
    }

    pub fn add_sum(&mut self, a: u32, b: u32) {
        self.value.set(self.value.get_or_default() + a + b);
    }

    pub fn increment_and_die(&mut self) {
        self.increment();
        contract_env::revert(Error::Forbidden);
    }

    pub fn increment(&mut self) {
        let old_value = self.value.get_or_default();
        let new_value = old_value + 1;
        self.value.set(new_value);
        
        <ValueUpdated as OdraEvent>::emit(ValueUpdated {
            old_value,
            new_value,
            operator: contract_env::caller()
        });
    }

    pub fn get_value(&self) -> u32 {
        self.value.get_or_default()
    }

    pub fn get_given_value(&self, result: u32) -> u32 {
        result
    }
}

odra::execution_error! {
    enum Error {
        Forbidden => 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deploy() {
        let mut counter = CounterDeployer::init(10);

        assert_eq!(33, counter.get_given_value(33));
        assert_eq!(10, counter.get_value());

        counter.increment();
        assert_eq!(11, counter.get_value());
    }

    #[test]
    fn increment() {
        let mut counter = CounterDeployer::init(10);
        counter.increment();
        assert_eq!(11, counter.get_value());
    }

    #[test]
    fn fail() {
        odra::test_env::assert_exception(Error::Forbidden, || {
            let mut counter = CounterDeployer::init(10);
            counter.increment_and_die();
        });
    }
}
