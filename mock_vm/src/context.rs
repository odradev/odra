use odra_types::Address;

#[derive(Debug, Default, Clone)]
pub struct ExecutionContext {
    callstack: Vec<Address>,
}

impl ExecutionContext {
    pub fn current(&self) -> &Address {
        self.callstack.last().expect("Callstack is empty")
    }

    pub fn previous(&self) -> &Address {
        self.callstack
            .get(self.callstack.len() - 2)
            .expect("Not enough elements on callstack")
    }

    pub fn push(&mut self, ctx: Address) {
        self.callstack.push(ctx);
    }

    pub fn drop(&mut self) {
        self.callstack.pop();
    }

    pub fn len(&self) -> usize {
        self.callstack.len()
    }
}
