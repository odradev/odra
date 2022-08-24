use odra_types::{bytesrepr::Bytes, OdraError, RuntimeArgs, VmError};
use std::collections::HashMap;

pub type EntrypointCall = fn(String, RuntimeArgs) -> Option<Bytes>;
pub type EntrypointArgs = Vec<String>;

#[derive(Default, Clone)]
pub struct ContractContainer {
    name: String,
    entrypoints: HashMap<String, (EntrypointArgs, EntrypointCall)>,
    constructors: HashMap<String, (EntrypointArgs, EntrypointCall)>,
}

impl ContractContainer {
    pub fn new(
        name: &str,
        entrypoints: HashMap<String, (EntrypointArgs, EntrypointCall)>,
        constructors: HashMap<String, (EntrypointArgs, EntrypointCall)>,
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
            Some((ep_args, call)) => {
                self.validate_args(ep_args, &args)?;
                Ok(call(self.name.clone(), args))
            }
            None => Err(OdraError::VmError(VmError::NoSuchMethod(entrypoint))),
        }
    }

    pub fn call_constructor(
        &self,
        entrypoint: String,
        args: RuntimeArgs,
    ) -> Result<Option<Bytes>, OdraError> {
        match self.constructors.get(&entrypoint) {
            Some((ep_args, call)) => {
                self.validate_args(ep_args, &args)?;
                Ok(call(self.name.clone(), args))
            }
            None => Err(OdraError::VmError(VmError::NoSuchMethod(entrypoint))),
        }
    }

    pub fn validate_args(
        &self,
        args: &[String],
        input_args: &RuntimeArgs,
    ) -> Result<(), OdraError> {
        let named_args = input_args
            .named_args()
            .map(|arg| arg.name().to_owned())
            .collect::<Vec<_>>();

        let missing_args = args
            .iter()
            .filter(|arg| !named_args.contains(arg))
            .map(|arg| arg.to_owned())
            .collect::<Vec<_>>();

        if missing_args.is_empty() {
            Ok(())
        } else {
            Err(OdraError::VmError(VmError::MissingArgs(missing_args)))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use odra_types::{bytesrepr::Bytes, runtime_args, OdraError, RuntimeArgs, VmError};

    use crate::{EntrypointArgs, EntrypointCall};

    use super::ContractContainer;

    #[test]
    fn test_call_wrong_entrypoint() {
        // Given an instance with no entrypoints.
        let instance = ContractContainer::empty();

        // When call some entrypoint.
        let result = instance.call(String::from("ep"), RuntimeArgs::new());

        // Then an error occurs.
        assert!(result.is_err());
    }

    #[test]
    fn test_call_valid_entrypoint() {
        // Given an instance with a single no-args entrypoint.
        let ep_name = String::from("ep");
        let instance = ContractContainer::setup_entrypoint(ep_name.clone(), vec![]);

        // When call the registered entrypoint.
        let result = instance.call(ep_name, RuntimeArgs::new());

        // Then teh call succeeds.
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_valid_entrypoint_with_wrong_arg_name() {
        // Given an instance with a single entrypoint with one arg named "first".
        let ep_name = String::from("ep");
        let instance = ContractContainer::setup_entrypoint(ep_name.clone(), vec!["first"]);

        // When call the registered entrypoint with an arg named "second".
        let result = instance.call(ep_name, runtime_args! { "second" => 0 });

        // Then MissingArgs error is returned.
        assert_missing_args(result, vec!["first"]);
    }

    #[test]
    fn test_call_valid_entrypoint_with_missing_arg() {
        // Given an instance with a single entrypoint with one arg named "first".
        let ep_name = String::from("ep");
        let instance = ContractContainer::setup_entrypoint(ep_name.clone(), vec!["first"]);

        // When call a valid entrypoint without args.
        let result = instance.call(ep_name, RuntimeArgs::new());

        // Then MissingArgs error is returned.
        assert_missing_args(result, vec!["first"]);
    }

    #[test]
    fn test_all_missing_args_are_caught() {
        // Given an instance with a single entrypoint with "first", "second" and "third" args.
        let ep_name = String::from("ep");
        let instance =
            ContractContainer::setup_entrypoint(ep_name.clone(), vec!["first", "second", "third"]);

        // When call a valid entrypoint with a single valid args,
        let result = instance.call(ep_name, runtime_args! { "third" => 0 });

        // Then MissingArgs error is returned with the two remaining args.
        assert_missing_args(result, vec!["first", "second"]);
    }

    #[test]
    fn test_call_valid_constructor() {
        // Given an instance with a single no-arg constructor.
        let name = String::from("init");
        let instance = ContractContainer::setup_constructor(name.clone(), vec![]);

        // When call a valid constructor with a single valid args,
        let result = instance.call_constructor(name, RuntimeArgs::new());

        // Then the call succeeds.
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_invalid_constructor() {
        // Given an instance with no constructors.
        let instance = ContractContainer::empty();

        // When try to call some constructor.
        let result = instance.call_constructor(String::from("c"), RuntimeArgs::new());

        // Then the call fails.
        assert!(result.is_err());
    }

    #[test]
    fn test_call_valid_constructor_with_missing_arg() {
        // Given an instance with a single constructor with one arg named "first".
        let name = String::from("init");
        let instance = ContractContainer::setup_constructor(name.clone(), vec!["first"]);

        // When call a valid constructor, but with no args.
        let result = instance.call_constructor(name, RuntimeArgs::new());

        // Then MissingArgs error is returned.
        assert_missing_args(result, vec!["first"]);
    }

    #[test]
    fn test_call_constructor_in_invalid_context() {
        // Given an instance with a single constructor.
        let name = String::from("init");
        let instance = ContractContainer::setup_constructor(name.clone(), vec![]);

        // When call the constructor in the entrypoint context.
        let result = instance.call(name, RuntimeArgs::new());

        // Then the call fails.
        assert!(result.is_err());
    }

    fn assert_missing_args(result: Result<Option<Bytes>, OdraError>, names: Vec<&str>) {
        let args = names.iter().map(|arg| arg.to_string()).collect::<Vec<_>>();
        assert_eq!(
            result.unwrap_err(),
            OdraError::VmError(VmError::MissingArgs(args))
        );
    }

    impl ContractContainer {
        fn empty() -> Self {
            Self {
                name: String::from("contract"),
                entrypoints: HashMap::new(),
                constructors: HashMap::new(),
            }
        }

        fn setup_entrypoint(ep_name: String, args: Vec<&str>) -> Self {
            let call: EntrypointCall = |_, _| Some(vec![1, 2, 3].into());
            let args: EntrypointArgs = args.iter().map(|arg| arg.to_string()).collect();

            let mut entrypoints = HashMap::new();
            entrypoints.insert(ep_name.clone(), (args, call));

            Self {
                name: String::from("contract"),
                entrypoints,
                constructors: HashMap::new(),
            }
        }

        fn setup_constructor(ep_name: String, args: Vec<&str>) -> Self {
            let call: EntrypointCall = |_, _| Some(vec![1, 2, 3].into());
            let args: EntrypointArgs = args.iter().map(|arg| arg.to_string()).collect();

            let mut constructors = HashMap::new();
            constructors.insert(ep_name.clone(), (args, call));

            Self {
                name: String::from("contract"),
                entrypoints: HashMap::new(),
                constructors,
            }
        }
    }
}
