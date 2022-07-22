extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod event;
mod execution_error;
mod external_contract;
mod instance;
mod module;
mod odra_error;

/// Core element of the Odra framework, entry point for writing smart contracts.
///
/// Each module consists of two parts:
/// 1. Module definition - a struct which composition of stored values (Variables and Mappings)
/// and modules.
/// 1. Module implementation - an implementation block.
///
/// The macro produces all the required code to use the module as a standalone smart contract.
///
///
/// # Examples
///
/// ```
/// use odra;
///
/// #[odra::module]
/// pub struct Flipper {
///     value: odra::Variable<bool>,
/// }
///
/// #[odra::module]
/// impl Flipper {
///     #[odra(init)]
///     pub fn initial_settings(&self, value: bool) {
///         self.value.set(value);
///     }
///
///     pub fn flip(&self) {
///         self.value.set(!self.get());
///     }
///
///     pub fn get(&self) -> bool {
///         self.value.get_or_default()
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn module(attr: TokenStream, item: TokenStream) -> TokenStream {
    module::generate_code(attr, item).into()
}

/// Provides implementation of [Instance](../odra/instance/trait.Instance.html) trait.
#[proc_macro_derive(Instance)]
pub fn derive_instance(input: TokenStream) -> TokenStream {
    instance::generate_code(parse_macro_input!(input as DeriveInput)).into()
}

/// Provides implementation of a reference to an external contract.
///
/// If you don't have access to the contract source code, but want to call it,
/// you can create a reference to it and interact exactly the same way as with a contract
/// written using [macro@module] macro.
///
/// # Examples
///
/// ```
/// use odra;
///
/// #[odra::external_contract]
/// pub trait Getter {
///     fn get(&self) -> u32;
/// }
///
/// let contract_address = odra::types::Address::new(b"address");
/// // in your contract
/// let getter = GetterRef::at(contract_address);
/// // let value = getter.get();
/// ```
#[proc_macro_attribute]
pub fn external_contract(attr: TokenStream, item: TokenStream) -> TokenStream {
    external_contract::generate_code(attr, item).into()
}

/// Implements boilerplate code required by an event.
///
/// Implements [Event](../odra_types/event/trait.Event.html) trait, and serialization/deserialization.
///
/// # Examples
///
/// ```
/// use odra;
///
/// #[derive(odra::Event)]
/// pub struct ValueUpdated {
///     pub value: u32,
/// }
/// let event = ValueUpdated { value: 42 };
///
/// assert_eq!(&<ValueUpdated as odra::types::event::Event>::name(&event), "ValueUpdated");
/// ```
#[proc_macro_derive(Event)]
pub fn derive_event(input: TokenStream) -> TokenStream {
    event::generate_code(parse_macro_input!(input as DeriveInput)).into()
}

/// Implements `Into<odra::types::ExecutionError>` and `Into<odra::types::OdraError>` for an error enum.
///
/// An enum should use a custom syntax, and each variant is mapped to n error code e.g. `Name => 1`.
///
/// # Examples
///
/// ```
/// use odra;
///
/// odra::execution_error! {
///     pub enum Error {
///         Fatal => 1,
///         Checked => 2,
///     }
/// };
///
/// let exec_error: odra::types::ExecutionError = Error::Fatal.into();
/// let odra_error: odra::types::OdraError = Error::Checked.into();
/// ```
///
/// Each variant must have a code.
/// ```compile_fail
/// use odra;
///
/// odra::execution_error! {
///     pub enum Error {
///         Fatal => 1,
///         Checked,
///     }
/// };
///
/// ```
///
/// Each code must be unique.
///
/// ```compile_fail
/// use odra;
///
/// odra::execution_error! {
///     pub enum Error {
///         Fatal => 1,
///         Checked => 1,
///     }
/// };
/// ```
#[proc_macro]
pub fn execution_error(item: TokenStream) -> TokenStream {
    execution_error::generate_code(item).into()
}

/// Implements `Into<odra::types::OdraError>` for an error enum.
///
/// In most cases the [execution_error!] is preferred, but if `Into<odra::types::ExecutionError>` is
/// implemented manually, the implementation of `Into<odra::types::OdraError>` still can be delegated to the macro.
#[proc_macro_attribute]
pub fn odra_error(_attr: TokenStream, item: TokenStream) -> TokenStream {
    odra_error::generate_code(item).into()
}
