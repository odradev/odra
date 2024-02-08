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
}
