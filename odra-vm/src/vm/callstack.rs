use odra_types::{casper_types::U512, Address};

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
    pub entrypoint: String,
    pub attached_value: U512
}

impl Entrypoint {
    pub fn new(address: Address, entrypoint: &str, value: U512) -> Self {
        Self {
            address: address,
            entrypoint: entrypoint.to_string(),
            attached_value: value
        }
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
            CallstackElement::Entrypoint(e) => e.attached_value
        }
    }

    pub fn attach_value(&mut self, amount: U512) {
        if let Some(CallstackElement::Entrypoint(entrypoint)) = self.0.last_mut() {
            entrypoint.attached_value = amount;
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
