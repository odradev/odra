use std::collections::HashMap;

use odra_mock_vm_types::CallArgs;
use odra_types::{OdraError, VmError};

pub type EntrypointCall = fn(String, CallArgs) -> Option<Vec<u8>>;
pub type EntrypointArgs = Vec<String>;

#[derive(Default, Clone)]
pub struct ContractContainer {
    name: String,
    entrypoints: HashMap<String, (EntrypointArgs, EntrypointCall)>,
    constructors: HashMap<String, (EntrypointArgs, EntrypointCall)>
}

impl ContractContainer {
    pub fn new(
        name: &str,
        entrypoints: HashMap<String, (EntrypointArgs, EntrypointCall)>,
        constructors: HashMap<String, (EntrypointArgs, EntrypointCall)>
    ) -> Self {
        Self {
            name: String::from(name),
            entrypoints,
            constructors
        }
    }

    pub fn call(&self, entrypoint: String, args: CallArgs) -> Result<Option<Vec<u8>>, OdraError> {
        if self.constructors.get(&entrypoint).is_some() {
            return Err(OdraError::VmError(VmError::InvalidContext));
        }

        match self.entrypoints.get(&entrypoint) {
            Some((ep_args, call)) => {
                self.validate_args(ep_args, &args)?;
                Ok(call(self.name.clone(), args))
            }
            None => Err(OdraError::VmError(VmError::NoSuchMethod(entrypoint)))
        }
    }

    pub fn call_constructor(
        &self,
        entrypoint: String,
        args: CallArgs
    ) -> Result<Option<Vec<u8>>, OdraError> {
        match self.constructors.get(&entrypoint) {
            Some((ep_args, call)) => {
                self.validate_args(ep_args, &args)?;
                Ok(call(self.name.clone(), args))
            }
            None => Err(OdraError::VmError(VmError::NoSuchMethod(entrypoint)))
        }
    }

    fn validate_args(&self, args: &[String], input_args: &CallArgs) -> Result<(), OdraError> {
        let named_args = input_args.arg_names();

        if args
            .iter()
            .filter(|arg| !named_args.contains(arg))
            .map(|arg| arg.to_owned())
            .next()
            .is_none()
        {
            Ok(())
        } else {
            Err(OdraError::VmError(VmError::MissingArg))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use odra_mock_vm_types::CallArgs;
    use odra_types::{OdraError, VmError};

    use crate::{EntrypointArgs, EntrypointCall};

    use super::ContractContainer;

    #[test]
    fn test_call_wrong_entrypoint() {
        // Given an instance with no entrypoints.
        let instance = ContractContainer::empty();

        // When call some entrypoint.
        let result = instance.call(String::from("ep"), CallArgs::new());

        // Then an error occurs.
        assert!(result.is_err());
    }

    #[test]
    fn test_call_valid_entrypoint() {
        // Given an instance with a single no-args entrypoint.
        let ep_name = String::from("ep");
        let instance = ContractContainer::setup_entrypoint(ep_name.clone(), vec![]);

        // When call the registered entrypoint.
        let result = instance.call(ep_name, CallArgs::new());

        // Then teh call succeeds.
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_valid_entrypoint_with_wrong_arg_name() {
        // Given an instance with a single entrypoint with one arg named "first".
        let ep_name = String::from("ep");
        let instance = ContractContainer::setup_entrypoint(ep_name.clone(), vec!["first"]);

        // When call the registered entrypoint with an arg named "second".
        let mut args = CallArgs::new();
        args.insert("second", 0);
        let result = instance.call(ep_name, args);

        // Then MissingArg error is returned.
        assert_eq!(result.unwrap_err(), OdraError::VmError(VmError::MissingArg));
    }

    #[test]
    fn test_call_valid_entrypoint_with_missing_arg() {
        // Given an instance with a single entrypoint with one arg named "first".
        let ep_name = String::from("ep");
        let instance = ContractContainer::setup_entrypoint(ep_name.clone(), vec!["first"]);

        // When call a valid entrypoint without args.
        let result = instance.call(ep_name, CallArgs::new());

        // Then MissingArg error is returned.
        assert_eq!(result.unwrap_err(), OdraError::VmError(VmError::MissingArg));
    }

    #[test]
    #[ignore = "At the moment is impossible to find the name of all missing args."]
    fn test_all_missing_args_are_caught() {
        // Given an instance with a single entrypoint with "first", "second" and "third" args.
        let ep_name = String::from("ep");
        let instance =
            ContractContainer::setup_entrypoint(ep_name.clone(), vec!["first", "second", "third"]);

        // When call a valid entrypoint with a single valid args,
        let mut args = CallArgs::new();
        args.insert("third", 0);
        let result = instance.call(ep_name, args);

        // Then MissingArg error is returned with the two remaining args.
        assert_eq!(result.unwrap_err(), OdraError::VmError(VmError::MissingArg));
    }

    #[test]
    fn test_call_valid_constructor() {
        // Given an instance with a single no-arg constructor.
        let name = String::from("init");
        let instance = ContractContainer::setup_constructor(name.clone(), vec![]);

        // When call a valid constructor with a single valid args,
        let result = instance.call_constructor(name, CallArgs::new());

        // Then the call succeeds.
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_invalid_constructor() {
        // Given an instance with no constructors.
        let instance = ContractContainer::empty();

        // When try to call some constructor.
        let result = instance.call_constructor(String::from("c"), CallArgs::new());

        // Then the call fails.
        assert!(result.is_err());
    }

    #[test]
    fn test_call_valid_constructor_with_missing_arg() {
        // Given an instance with a single constructor with one arg named "first".
        let name = String::from("init");
        let instance = ContractContainer::setup_constructor(name.clone(), vec!["first"]);

        // When call a valid constructor, but with no args.
        let result = instance.call_constructor(name, CallArgs::new());

        // Then MissingArgs error is returned.
        assert_eq!(result.unwrap_err(), OdraError::VmError(VmError::MissingArg));
    }

    #[test]
    fn test_call_constructor_in_invalid_context() {
        // Given an instance with a single constructor.
        let name = String::from("init");
        let instance = ContractContainer::setup_constructor(name.clone(), vec![]);

        // When call the constructor in the entrypoint context.
        let result = instance.call(name, CallArgs::new());

        // Then the call fails.
        assert!(result.is_err());
    }

    impl ContractContainer {
        fn empty() -> Self {
            Self {
                name: String::from("contract"),
                entrypoints: HashMap::new(),
                constructors: HashMap::new()
            }
        }

        fn setup_entrypoint(ep_name: String, args: Vec<&str>) -> Self {
            let call: EntrypointCall = |_, _| Some(vec![1, 2, 3]);
            let args: EntrypointArgs = args.iter().map(|arg| arg.to_string()).collect();

            let mut entrypoints = HashMap::new();
            entrypoints.insert(ep_name, (args, call));

            Self {
                name: String::from("contract"),
                entrypoints,
                constructors: HashMap::new()
            }
        }

        fn setup_constructor(ep_name: String, args: Vec<&str>) -> Self {
            let call: EntrypointCall = |_, _| Some(vec![1, 2, 3]);
            let args: EntrypointArgs = args.iter().map(|arg| arg.to_string()).collect();

            let mut constructors = HashMap::new();
            constructors.insert(ep_name, (args, call));

            Self {
                name: String::from("contract"),
                entrypoints: HashMap::new(),
                constructors
            }
        }
    }
}