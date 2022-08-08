use odra_types::{bytesrepr::Bytes, OdraError, RuntimeArgs, VmError};
use std::collections::HashMap;

pub type EntrypointCall = fn(String, RuntimeArgs) -> Option<Bytes>;

#[derive(Default, Clone)]
pub struct ContractContainer {
    name: String,
    entrypoints: HashMap<String, EntrypointCall>,
    constructors: HashMap<String, EntrypointCall>,
}

impl ContractContainer {
    pub fn new(
        name: &str,
        entrypoints: HashMap<String, EntrypointCall>,
        constructors: HashMap<String, EntrypointCall>,
    ) -> Self {
        Self {
            name: String::from(name),
            entrypoints,
            constructors,
        }
    }

    pub fn call(&self, entrypoint: String, args: RuntimeArgs) -> Result<Option<Bytes>, OdraError> {
        if self.constructors.get(&entrypoint).is_some() {
            return Err(OdraError::VmError(VmError::InvalidContext));
        }

        match self.entrypoints.get(&entrypoint) {
            Some(f) => Ok(f(self.name.clone(), args)),
            None => Err(OdraError::VmError(VmError::NoSuchMethod(entrypoint))),
        }
    }

    pub fn call_constructor(
        &self,
        entrypoint: String,
        args: RuntimeArgs,
    ) -> Result<Option<Bytes>, OdraError> {
        match self.constructors.get(&entrypoint) {
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
        let instance = ContractContainer::new("c1", HashMap::new(), HashMap::new());

        let result = instance.call(String::from("call"), RuntimeArgs::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_call_valid_entrypoint() {
        let ep_name = String::from("call");

        let ep: EntrypointCall = |_, _| Some(vec![1, 2, 3].into());
        let mut entrypoints = HashMap::new();
        entrypoints.insert(ep_name.clone(), ep);
        let instance = ContractContainer::new("c1", entrypoints, HashMap::new());

        let result = instance.call(ep_name, RuntimeArgs::new());
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_valid_constructor() {
        let name = String::from("init");

        let call: EntrypointCall = |_, _| Some(vec![1, 2, 3].into());
        let mut constructors = HashMap::new();
        constructors.insert(name.clone(), call);
        let instance = ContractContainer::new("c1", HashMap::new(), constructors);

        let result = instance.call_constructor(name, RuntimeArgs::new());
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_invalid_constructor() {
        let instance = ContractContainer::new("c1", HashMap::new(), HashMap::new());

        let result = instance.call_constructor(String::from("call"), RuntimeArgs::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_call_constructor_in_invalid_context() {
        let name = String::from("init");

        let call: EntrypointCall = |_, _| Some(vec![1, 2, 3].into());
        let mut constructors = HashMap::new();
        constructors.insert(name.clone(), call);
        let instance = ContractContainer::new("c1", HashMap::new(), constructors);

        let result = instance.call(name, RuntimeArgs::new());
        assert!(result.is_err());
    }
}
