use odra_mock_vm_types::{Address, Balance};

#[derive(Clone)]
pub struct CallstackElement {
    pub address: Address,
    pub attached_value: Option<Balance>
}

impl CallstackElement {
    pub fn new(address: Address, value: Option<Balance>) -> Self {
        Self {
            address,
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

    pub fn current_amount(&self) -> Balance {
        self.0
            .last()
            .and_then(|e| e.attached_value)
            .unwrap_or_default()
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
