use odra::prelude::*;
use odra::Var;

/// Contract designed to benchmark the Odra framework.
#[odra::module]
pub struct Benchmark {
    value: Var<bool>
}

#[odra::module]
impl Benchmark {
    pub fn init(&mut self) {
        self.value.set(false);
    }

    pub fn set_value(&mut self, value: bool) {
        self.value.set(value && self.value.get().unwrap());
    }
}
