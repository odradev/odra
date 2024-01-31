use crate::prelude::*;
use casper_types::bytesrepr::FromBytes;
use casper_types::{CLTyped, RuntimeArgs, U512};

/// Represents a call definition, which includes the method name, runtime arguments, and attached value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallDef {
    entry_point: String,
    args: RuntimeArgs,
    amount: U512,
    is_mut: bool
}

impl CallDef {
    /// Creates a new `CallDef` instance with the given method name and runtime arguments.
    ///
    /// # Arguments
    ///
    /// * `method` - The method name.
    /// * `args` - The runtime arguments.
    ///
    /// # Example
    ///
    /// ```
    /// # use odra_core::CallDef;
    /// # use casper_types::RuntimeArgs;
    /// let method = "my_method";
    /// let args = RuntimeArgs::new();
    /// let call_def = CallDef::new(method, false, args);
    /// ```
    pub fn new<T: ToString>(method: T, is_mut: bool, args: RuntimeArgs) -> Self {
        CallDef {
            entry_point: method.to_string(),
            args,
            amount: U512::zero(),
            is_mut
        }
    }

    /// Sets the attached value for the `CallDef` instance.
    ///
    /// # Arguments
    ///
    /// * `amount` - The attached value.
    ///
    /// # Returns
    ///
    /// The modified `CallDef` instance.
    ///
    /// # Example
    ///
    /// ```
    /// # use odra_core::CallDef;
    /// # use casper_types::RuntimeArgs;
    /// # use casper_types::U512;
    /// let call_def = CallDef::new("my_method", false, RuntimeArgs::new())
    ///     .with_amount(U512::from(100));
    /// ```
    pub fn with_amount(mut self, amount: U512) -> Self {
        self.amount = amount;
        self
    }

    /// Returns a reference to the entry point name of the `CallDef` instance.
    pub fn entry_point(&self) -> &str {
        &self.entry_point
    }

    /// Returns the attached value of the `CallDef` instance.
    pub fn amount(&self) -> U512 {
        self.amount
    }

    /// Returns a reference to the runtime arguments of the `CallDef` instance.
    pub fn args(&self) -> &RuntimeArgs {
        &self.args
    }

    /// Retrieves a value from the runtime arguments of the `CallDef` instance.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the value to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option` containing the retrieved value, or `None` if the value does not exist or cannot be converted to the specified type.
    ///
    /// # Example
    ///
    /// ```
    /// # use odra_core::CallDef;
    /// # use casper_types::{CLTyped, bytesrepr::FromBytes, RuntimeArgs};
    /// let call_def = CallDef::new("my_method", false, RuntimeArgs::new());
    /// let value: Option<u32> = call_def.get("my_value");
    /// ```
    pub fn get<T: CLTyped + FromBytes>(&self, name: &str) -> Option<T> {
        self.args.get(name).and_then(|v| v.clone().into_t().ok())
    }

    /// Returns whether the call mutates the state.
    pub fn is_mut(&self) -> bool {
        self.is_mut
    }
}

#[cfg(test)]
mod test {
    use crate::CallDef;
    use casper_types::{runtime_args, RuntimeArgs};

    #[test]
    fn test_get_arg() {
        let args = runtime_args! {
            "my_value" => 42u32
        };
        let call_def = CallDef::new("my_method", false, args);
        let value: Option<u32> = call_def.get("my_value");
        assert_eq!(value, Some(42u32));

        let value: Option<u32> = call_def.get("your_value");
        assert_eq!(value, None);
    }

    #[test]
    fn test_is_mut() {
        let call_def = CallDef::new("my_method", false, RuntimeArgs::new());
        assert!(!call_def.is_mut());

        let call_def = CallDef::new("my_method", true, RuntimeArgs::new());
        assert!(call_def.is_mut());
    }
}
