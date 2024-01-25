use crate::entry_point_callback::{Argument, EntryPointsCaller};
use crate::{prelude::*, OdraResult};
use crate::{CallDef, OdraError, VmError};
use casper_types::bytesrepr::Bytes;
use casper_types::RuntimeArgs;

/// A wrapper struct for a EntryPointsCaller that is a layer of abstraction between the host and the entry points caller.
///
/// The container validates a contract call definition before calling the entry point.
#[derive(Clone)]
pub struct ContractContainer {
    entry_points_caller: EntryPointsCaller
}

impl ContractContainer {
    /// Creates a new instance of `ContractContainer`.
    pub fn new(entry_points_caller: EntryPointsCaller) -> Self {
        Self {
            entry_points_caller
        }
    }

    /// Calls the entry point with the given call definition.
    pub fn call(&self, call_def: CallDef) -> OdraResult<Bytes> {
        // find the entry point
        let ep = self
            .entry_points_caller
            .entry_points()
            .iter()
            .find(|ep| ep.name == call_def.entry_point())
            .ok_or_else(|| {
                OdraError::VmError(VmError::NoSuchMethod(call_def.entry_point().to_string()))
            })?;
        // validate the args, return an error if the args are invalid
        self.validate_args(&ep.args, call_def.args())?;
        self.entry_points_caller.call(call_def)
    }

    fn validate_args(&self, args: &[Argument], input_args: &RuntimeArgs) -> OdraResult<()> {
        for arg in args {
            // check if the input args contain the arg
            if let Some(input) = input_args
                .named_args()
                .find(|input| input.name() == arg.name.as_str())
            {
                // check if the input arg has the expected type
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

// #[cfg(test)]
// mod tests {
//     use casper_types::CLType;

//     use super::ContractContainer;
//     use crate::entry_point_callback::{EntryPoint, Argument, EntryPointsCaller};
//     use crate::{prelude::*, CallDef};
//     use crate::{
//         casper_types::{runtime_args, RuntimeArgs},
//         OdraError, VmError
//     };

//     const TEST_ENTRYPOINT: &'static str = "ep";

//     #[test]
//     fn test_call_wrong_entrypoint() {
//         // Given an instance with no entrypoints.
//         let instance = ContractContainer::empty();

//         // When call some entrypoint.
//         let call_def = CallDef::new(TEST_ENTRYPOINT, false, RuntimeArgs::new());
//         let result = instance.call(call_def);

//         // Then an error occurs.
//         assert!(result.is_err());
//     }

//     #[test]
//     fn test_call_valid_entrypoint() {
//         // Given an instance with a single no-args entrypoint.
//         let instance = ContractContainer::with_entrypoint(vec![]);

//         // When call the registered entrypoint.
//         let call_def = CallDef::new(TEST_ENTRYPOINT, false, RuntimeArgs::new());
//         let result = instance.call(call_def);

//         // Then teh call succeeds.
//         assert!(result.is_ok());
//     }

//     #[test]
//     fn test_call_valid_entrypoint_with_wrong_arg_name() {
//         // Given an instance with a single entrypoint with one arg named "first".
//         let instance = ContractContainer::with_entrypoint(vec![("first", CLType::U32)]);

//         // When call the registered entrypoint with an arg named "second".
//         let call_def = CallDef::new(TEST_ENTRYPOINT, false, runtime_args! { "second" => 0u32 });
//         let result = instance.call(call_def);

//         // Then MissingArg error is returned.
//         assert_eq!(result.unwrap_err(), OdraError::VmError(VmError::MissingArg));
//     }

//     #[test]
//     fn test_call_valid_entrypoint_with_wrong_arg_type() {
//         // Given an instance with a single entrypoint with one arg named "first".
//         let instance = ContractContainer::with_entrypoint(vec![("first", CLType::U32)]);

//         // When call the registered entrypoint with an arg named "second".
//         let call_def = CallDef::new(TEST_ENTRYPOINT, false, runtime_args! { "first" => true });
//         let result = instance.call(call_def);

//         // Then MissingArg error is returned.
//         assert_eq!(
//             result.unwrap_err(),
//             OdraError::VmError(VmError::TypeMismatch {
//                 expected: CLType::U32,
//                 found: CLType::Bool
//             })
//         );
//     }

//     #[test]
//     fn test_call_valid_entrypoint_with_missing_arg() {
//         // Given an instance with a single entrypoint with one arg named "first".
//         let instance = ContractContainer::with_entrypoint(vec![("first", CLType::U32)]);

//         // When call a valid entrypoint without args.
//         let call_def = CallDef::new(TEST_ENTRYPOINT, false, RuntimeArgs::new());
//         let result = instance.call(call_def);

//         // Then MissingArg error is returned.
//         assert_eq!(result.unwrap_err(), OdraError::VmError(VmError::MissingArg));
//     }

//     #[test]
//     fn test_many_missing_args() {
//         // Given an instance with a single entrypoint with "first", "second" and "third" args.
//         let instance = ContractContainer::with_entrypoint(vec![
//             ("first", CLType::U32),
//             ("second", CLType::U32),
//             ("third", CLType::U32),
//         ]);

//         // When call a valid entrypoint with a single valid args,
//         let call_def = CallDef::new(TEST_ENTRYPOINT, false, runtime_args! { "third" => 0u32 });
//         let result = instance.call(call_def);

//         // Then MissingArg error is returned.
//         assert_eq!(result.unwrap_err(), OdraError::VmError(VmError::MissingArg));
//     }

//     impl ContractContainer {
//         fn empty() -> Self {
//             let env = odra_test::env();
//             let entry_points_caller = EntryPointsCaller::new(env, vec![], |_, call_def| {
//                 Err(OdraError::VmError(VmError::NoSuchMethod(
//                     call_def.entry_point().to_string()
//                 )))
//             });
//             Self {
//                 entry_points_caller
//             }
//         }

//         fn with_entrypoint(args: Vec<(&str, CLType)>) -> Self {
//             let entry_points = vec![EntryPoint::new(
//                 String::from(TEST_ENTRYPOINT),
//                 args.iter()
//                     .map(|(name, ty)| Argument::new(String::from(*name), ty.to_owned()))
//                     .collect()
//             )];

//             let entry_points_caller =
//                 EntryPointsCaller::new(odra_test::env(), entry_points, |env, call_def| {
//                     if call_def.entry_point() == TEST_ENTRYPOINT {
//                         Ok(vec![1, 2, 3].into())
//                     } else {
//                         Err(OdraError::VmError(VmError::NoSuchMethod(
//                             call_def.entry_point().to_string()
//                         )))
//                     }
//                 });

//             Self {
//                 entry_points_caller
//             }
//         }
//     }
// }
