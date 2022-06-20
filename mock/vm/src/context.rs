use odra_types::Address;

#[derive(Debug, Clone)]
pub struct Context {
    pub address: Address,
}

impl Context {}

impl From<Address> for Context {
    fn from(address: Address) -> Self {
        Self { address }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ExecutionContext {
    callstack: Vec<Context>,
}

impl ExecutionContext {
    pub fn current(&self) -> &Context {
        self.callstack
            .last()
            .expect("Cannot modify storage in empty context")
    }

    pub fn previous(&self) -> &Context {
        self.callstack.get(self.callstack.len() - 2).unwrap()
    }

    pub fn push(&mut self, ctx: Context) {
        self.callstack.push(ctx);
    }

    pub fn drop(&mut self) {
        self.callstack.pop();
    }
}
