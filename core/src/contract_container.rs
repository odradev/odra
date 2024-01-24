use crate::prelude::*;
use crate::{CallDef, EntryPointsCaller, OdraError, VmError, EntryPointArgument};
use casper_types::bytesrepr::Bytes;
use casper_types::{NamedArg, RuntimeArgs};

#[doc(hidden)]
pub type EntrypointCall = fn(String, &RuntimeArgs) -> Vec<u8>;
#[doc(hidden)]
pub type EntrypointArgs = Vec<String>;

#[derive(Clone)]
pub struct ContractContainer {
    _name: String,
    entry_points_caller: EntryPointsCaller
}

impl ContractContainer {
    pub fn new(name: &str, entry_points_caller: EntryPointsCaller) -> Self {
        Self {
            _name: String::from(name),
            entry_points_caller
        }
    }

    pub fn call(&self, call_def: CallDef) -> Result<Bytes, OdraError> {
        let ep = self
            .entry_points_caller
            .entry_points()
            .iter()
            .find(|ep| ep.name == call_def.method())
            .ok_or_else(|| {
                OdraError::VmError(VmError::NoSuchMethod(call_def.method().to_string()))
            })?;
        self.validate_args(&ep.args, call_def.args())?;
        self.entry_points_caller.call(call_def)
    }

    fn validate_args(
        &self,
        args: &[EntryPointArgument],
        input_args: &RuntimeArgs
    ) -> Result<(), OdraError> {
        let _named_args = input_args
            .named_args()
            .map(NamedArg::name)
            .collect::<Vec<_>>();

        for arg in args {
            if let Some(input) = input_args
                .named_args()
                .find(|input| input.name() == arg.name.as_str())
            {
                let input_ty = input.cl_value().cl_type();
                let expected_ty = &arg.ty;
                if input_ty != expected_ty {
                    return Err(OdraError::VmError(VmError::TypeMismatch {
                        expected: expected_ty.clone(),
                        found: input_ty.clone()
                    }));
                }
            } else {
                return Err(OdraError::VmError(VmError::MissingArg));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{ContractContainer, EntrypointArgs, EntrypointCall};
    use crate::prelude::*;
    use crate::{
        casper_types::{runtime_args, RuntimeArgs},
        OdraError, VmError
    };
    use crate::{CLType, CallDef, EntryPoint, EntryPointArgument, EntryPointsCaller, HostEnv};

    const TEST_ENTRYPOINT: &'static str = "ep";

    #[test]
    fn test_call_wrong_entrypoint() {
        // Given an instance with no entrypoints.
        let instance = ContractContainer::empty();

        // When call some entrypoint.
        let call_def = CallDef::new(TEST_ENTRYPOINT, false, RuntimeArgs::new());
        let result = instance.call(call_def);

        // Then an error occurs.
        assert!(result.is_err());
    }

    #[test]
    fn test_call_valid_entrypoint() {
        // Given an instance with a single no-args entrypoint.
        let instance = ContractContainer::with_entrypoint(vec![]);

        // When call the registered entrypoint.
        let call_def = CallDef::new(TEST_ENTRYPOINT, false, RuntimeArgs::new());
        let result = instance.call(call_def);

        // Then teh call succeeds.
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_valid_entrypoint_with_wrong_arg_name() {
        // Given an instance with a single entrypoint with one arg named "first".
        let instance = ContractContainer::with_entrypoint(vec![("first", CLType::U32)]);

        // When call the registered entrypoint with an arg named "second".
        let call_def = CallDef::new(TEST_ENTRYPOINT, runtime_args! { "second" => 0u32 });
        let result = instance.call(call_def);

        // Then MissingArg error is returned.
        assert_eq!(result.unwrap_err(), OdraError::VmError(VmError::MissingArg));
    }

    #[test]
    fn test_call_valid_entrypoint_with_wrong_arg_type() {
        // Given an instance with a single entrypoint with one arg named "first".
        let instance = ContractContainer::with_entrypoint(vec![("first", CLType::U32)]);

        // When call the registered entrypoint with an arg named "second".
        let call_def = CallDef::new(TEST_ENTRYPOINT, runtime_args! { "first" => true });
        let result = instance.call(call_def);

        // Then MissingArg error is returned.
        assert_eq!(
            result.unwrap_err(),
            OdraError::VmError(VmError::TypeMismatch {
                expected: CLType::U32,
                found: CLType::Bool
            })
        );
    }

    #[test]
    fn test_call_valid_entrypoint_with_missing_arg() {
        // Given an instance with a single entrypoint with one arg named "first".
        let instance = ContractContainer::with_entrypoint(vec![("first", CLType::U32)]);

        // When call a valid entrypoint without args.
        let call_def = CallDef::new(TEST_ENTRYPOINT, false, RuntimeArgs::new());
        let result = instance.call(call_def);

        // Then MissingArg error is returned.
        assert_eq!(result.unwrap_err(), OdraError::VmError(VmError::MissingArg));
    }

    #[test]
    fn test_many_missing_args() {
        // Given an instance with a single entrypoint with "first", "second" and "third" args.
        let instance = ContractContainer::with_entrypoint(vec![
            ("first", CLType::U32),
            ("second", CLType::U32),
            ("third", CLType::U32),
        ]);

        // When call a valid entrypoint with a single valid args,
        let call_def = CallDef::new(TEST_ENTRYPOINT, false, runtime_args! { "third" => 0u32 });
        let result = instance.call(call_def);

        // Then MissingArg error is returned.
        assert_eq!(result.unwrap_err(), OdraError::VmError(VmError::MissingArg));
    }

    impl ContractContainer {
        fn empty() -> Self {
            let env = odra::odra_test::odra_env();
            let entry_points_caller = EntryPointsCaller::new(env, vec![], |_, call_def| {
                Err(OdraError::VmError(VmError::NoSuchMethod(
                    call_def.method().to_string()
                )))
            });
            Self {
                _name: String::from("contract"),
                entry_points_caller
            }
        }

        fn with_entrypoint(args: Vec<(&str, CLType)>) -> Self {
            let entry_points = vec![EntryPoint::new(
                String::from(TEST_ENTRYPOINT),
                args.iter()
                    .map(|(name, ty)| EntryPointArgument::new(String::from(*name), ty.to_owned()))
                    .collect()
            )];

            let entry_points_caller = EntryPointsCaller::new(
                odra::odra_test::odra_env(),
                entry_points,
                |env, call_def| {
                    if call_def.method() == TEST_ENTRYPOINT {
                        Ok(vec![1, 2, 3].into())
                    } else {
                        Err(OdraError::VmError(VmError::NoSuchMethod(
                            call_def.method().to_string()
                        )))
                    }
                }
            );

            Self {
                _name: String::from("contract"),
                entry_points_caller
            }
        }
    }
}
