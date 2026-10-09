use crate::entry_point_callback::EntryPointsCaller;
use crate::prelude::*;
use crate::{CallDef, ContractEnv, VmError};
use casper_types::bytesrepr::Bytes;
use casper_types::U512;

/// A wrapper struct for a EntryPointsCaller that is a layer of abstraction between the host and the entry points caller.
///
/// The container validates a contract call definition before calling the entry point.
#[derive(Clone)]
pub struct ContractContainer {
    contract_name: String,
    entry_points_caller: EntryPointsCaller,
    ctx: ExecutionContext
}

impl ContractContainer {
    /// Creates a new instance of `ContractContainer`.
    pub fn new(name: &str, entry_points_caller: EntryPointsCaller) -> Self {
        Self {
            contract_name: name.to_string(),
            entry_points_caller,
            ctx: ExecutionContext::Installation
        }
    }

    pub(crate) fn post_install(&mut self) {
        self.ctx = ExecutionContext::Runtime;
    }

    /// Calls the entry point with the given call definition in the given contract environment.
    pub fn call(&self, contract_env: ContractEnv, call_def: CallDef) -> OdraResult<Bytes> {
        // find the entry point
        let ep = self
            .entry_points_caller
            .entry_points()
            .iter()
            .find(|ep| ep.name == call_def.entry_point())
            .ok_or_else(|| {
                OdraError::VmError(VmError::NoSuchMethod(call_def.entry_point().to_owned()))
            })?;
        if !ep.is_payable && call_def.amount() > U512::zero() {
            return Err(OdraError::ExecutionError(ExecutionError::NonPayable));
        }
        if ep.name == "init" && self.ctx == ExecutionContext::Runtime {
            return Err(OdraError::VmError(VmError::InvalidContext));
        }
        self.entry_points_caller.call(contract_env, call_def)
    }

    /// Returns the entry points caller of the contract.
    pub fn entry_points_caller(&self) -> &EntryPointsCaller {
        &self.entry_points_caller
    }

    /// Returns the name of the contract.
    pub fn name(&self) -> &str {
        &self.contract_name
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum ExecutionContext {
    Installation,
    Runtime
}

#[cfg(test)]
mod tests {
    use super::{ContractContainer, ExecutionContext};
    use crate::contract_context::MockContractContext;
    use crate::entry_point_callback::{Argument, EntryPoint, EntryPointsCaller};
    use crate::{casper_types::RuntimeArgs, VmError};
    use crate::{prelude::*, CallDef, ContractEnv};

    const TEST_ENTRYPOINT: &str = "ep";

    fn test_env() -> ContractEnv {
        ContractEnv::new(Rc::new(RefCell::new(MockContractContext::new())))
    }

    #[test]
    fn test_call_wrong_entrypoint() {
        // Given an instance with no entrypoints.
        let instance = ContractContainer::empty();

        // When call some entrypoint.
        let call_def = CallDef::new(TEST_ENTRYPOINT, false, RuntimeArgs::new());
        let result = instance.call(test_env(), call_def);

        // Then an error occurs.
        assert!(result.is_err());
    }

    #[test]
    fn test_call_valid_entrypoint() {
        // Given an instance with a single no-args entrypoint.
        let instance = ContractContainer::with_entrypoint(vec![]);

        // When call the registered entrypoint.
        let call_def = CallDef::new(TEST_ENTRYPOINT, false, RuntimeArgs::new());
        let result = instance.call(test_env(), call_def);

        // Then teh call succeeds.
        assert!(result.is_ok());
    }

    impl ContractContainer {
        fn empty() -> Self {
            let entry_points_caller = EntryPointsCaller::new(vec![], |_, call_def| {
                Err(OdraError::VmError(VmError::NoSuchMethod(
                    call_def.entry_point().to_string()
                )))
            });
            Self {
                contract_name: "empty".to_string(),
                entry_points_caller,
                ctx: ExecutionContext::Installation
            }
        }

        fn with_entrypoint(args: Vec<&str>) -> Self {
            let entry_points = vec![EntryPoint::new(
                String::from(TEST_ENTRYPOINT),
                args.iter()
                    .map(|name| Argument::new::<u32>(String::from(*name)))
                    .collect()
            )];
            let entry_points_caller = EntryPointsCaller::new(entry_points, |_, call_def| {
                if call_def.entry_point() == TEST_ENTRYPOINT {
                    Ok(vec![1, 2, 3].into())
                } else {
                    Err(OdraError::VmError(VmError::NoSuchMethod(
                        call_def.entry_point().to_string()
                    )))
                }
            });

            Self {
                contract_name: "with_entrypoints".to_string(),
                entry_points_caller,
                ctx: ExecutionContext::Installation
            }
        }
    }
}
