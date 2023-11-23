use std::collections::BTreeMap;

use odra_core::{casper_types::RuntimeArgs, Address, OdraError, VmError};

use crate::{EntrypointArgs, EntrypointCall};

#[derive(Clone)]
pub struct ContractContainer {
    address: Address,
    entrypoints: BTreeMap<String, (EntrypointArgs, EntrypointCall)>
}

impl ContractContainer {
    pub fn new(
        address: Address,
        entrypoints: BTreeMap<String, (EntrypointArgs, EntrypointCall)>
    ) -> Self {
        Self {
            address,
            entrypoints
        }
    }

    pub fn call(&self, entrypoint: String, args: RuntimeArgs) -> Result<Vec<u8>, OdraError> {
        match self.entrypoints.get(&entrypoint) {
            Some((ep_args, call)) => {
                self.validate_args(ep_args, &args)?;
                Ok(call(String::new(), &args))
            }
            None => Err(OdraError::VmError(VmError::NoSuchMethod(entrypoint)))
        }
    }

    pub fn address(&self) -> Address {
        self.address
    }

    fn validate_args(&self, args: &[String], input_args: &RuntimeArgs) -> Result<(), OdraError> {
        let named_args = input_args
            .named_args()
            .map(|arg| arg.name().to_owned())
            .collect::<Vec<_>>();

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
