use odra_types::Address;

#[derive(Clone, Default)]
pub struct Callstack(Vec<Address>);

impl Callstack {
    pub fn pop(&mut self) -> Option<Address> {
        self.0.pop()
    }

    pub fn push(&mut self, element: Address) {
        self.0.push(element);
    }

    pub fn current(&self) -> Address {
        *self.0.last().unwrap()
    }
}
