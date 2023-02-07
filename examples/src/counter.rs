use odra::Variable;

#[odra::module]
pub struct Counter {
    pub value: Variable<u32>
}

#[odra::module]
impl Counter {

    #[odra(init)]
    pub fn init(&mut self, value: u32) {
        self.value.set(value);
    }

    pub fn add(&mut self, value: u32) {
        self.value.set(self.value.get_or_default() + value);
    }

    pub fn add_sum(&mut self, a: u32, b: u32) {
        self.value.set(self.value.get_or_default() + a + b);
    }

    pub fn increment(&mut self) {
        self.value.set(self.value.get_or_default() + 1);
    }

    pub fn get_value(&self) -> u32 {
        self.value.get_or_default()
    }

    pub fn get_given_value(&self, result: u32) -> u32 {
        result
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
}