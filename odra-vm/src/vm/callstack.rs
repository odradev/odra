use odra_core::{casper_types::U512, Address, CallDef};

#[derive(Clone)]
pub enum CallstackElement {
    Account(Address),
    Entrypoint(Entrypoint)
}

impl CallstackElement {
    pub fn address(&self) -> &Address {
        match self {
            CallstackElement::Account(address) => address,
            CallstackElement::Entrypoint(entrypoint) => &entrypoint.address
        }
    }
}

#[derive(Clone)]
pub struct Entrypoint {
    pub address: Address,
    pub call_def: CallDef
}

impl Entrypoint {
    pub fn new(address: Address, call_def: CallDef) -> Self {
        Self { address, call_def }
    }
}

#[derive(Clone, Default)]
pub struct Callstack(Vec<CallstackElement>);

impl Callstack {
    pub fn pop(&mut self) -> Option<CallstackElement> {
        self.0.pop()
    }

    pub fn push(&mut self, element: CallstackElement) {
        self.0.push(element);
    }

    pub fn attached_value(&self) -> U512 {
        let ce = self.0.last().unwrap();
        match ce {
            CallstackElement::Account(_) => U512::zero(),
            CallstackElement::Entrypoint(e) => e.call_def.amount
        }
    }

    pub fn attach_value(&mut self, amount: U512) {
        if let Some(CallstackElement::Entrypoint(entrypoint)) = self.0.last_mut() {
            entrypoint.call_def.amount = amount;
        }
    }

    pub fn current(&self) -> &CallstackElement {
        self.0.last().expect("Not enough elements on callstack")
    }

    pub fn previous(&self) -> &CallstackElement {
        self.0
            .get(self.0.len() - 2)
            .expect("Not enough elements on callstack")
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}
