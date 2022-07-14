use odra_types::{bytesrepr::Bytes, OdraError, RuntimeArgs, VmError};
use std::collections::HashMap;

pub type EntrypointCall = fn(String, RuntimeArgs) -> Option<Bytes>;

#[derive(Default, Clone)]
pub struct ContractContainer {
    name: String,
    entrypoints: HashMap<String, EntrypointCall>,
}

impl ContractContainer {
    pub fn new(name: &str, entrypoints: HashMap<String, EntrypointCall>) -> Self {
        Self {
            name: String::from(name),
            entrypoints,
        }
    }

    pub fn call(&self, entrypoint: String, args: RuntimeArgs) -> Result<Option<Bytes>, OdraError> {
        match self.entrypoints.get(&entrypoint) {
            Some(f) => Ok(f(self.name.clone(), args)),
            None => Err(OdraError::VmError(VmError::NoSuchMethod(entrypoint))),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use odra_types::RuntimeArgs;

    use crate::EntrypointCall;

    use super::ContractContainer;

    #[test]
    fn test_call_wrong_entrypoint() {
        let instance = ContractContainer::new("c1", HashMap::new());

        let result = instance.call(String::from("call"), RuntimeArgs::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_call_valid_entrypoint() {
        let ep_name = String::from("call");

        let ep: EntrypointCall = |_, _| Some(vec![1, 2, 3].into());
        let mut entrypoints = HashMap::new();
        entrypoints.insert(ep_name.clone(), ep);
        let instance = ContractContainer::new("c1", entrypoints);

        let result = instance.call(ep_name.clone(), RuntimeArgs::new());
        assert!(result.is_ok());
    }
}
