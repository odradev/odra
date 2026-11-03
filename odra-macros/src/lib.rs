#![doc = "This crate provides a set of macros and derive attributes to simplify the process of writing"]
#![doc = "smart contracts for the Odra platform."]
use crate::utils::IntoCode;
use ast::*;
use ir::{errors, ModuleImplIR, ModuleStructIR, TypeIR};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

mod ast;
mod ir;
#[cfg(test)]
mod test_utils;
mod utils;

/// Core element of the Odra framework, entry point for writing smart contracts.
///
/// Each module consists of two parts:
/// 1. Module definition - a struct which composition of stored values (Vars and Mappings)
///    and modules.
/// 2. Module implementation - an implementation block.
///
/// The macro produces all the required code to use the module as a standalone smart contract.
#[proc_macro_attribute]
pub fn module(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr: TokenStream2 = attr.into();
    let item: TokenStream2 = item.into();
    if is_impl_or_trait(&item) {
        return match ModuleImplIR::try_from((&attr, &item)) {
            Ok(ir) => ModuleImplItem::try_from(&ir).into_code(),
            Err(e) => e.to_compile_error().into()
        };
    }
    if is_struct(&item) {
        return match ModuleStructIR::try_from((&attr, &item)) {
            Ok(ir) => ModuleStructItem::try_from(&ir).into_code(),
            Err(e) => e.to_compile_error().into()
        };
    }
    errors::module_on_other_item(&item)
        .to_compile_error()
        .into()
}

fn is_impl_or_trait(item: &TokenStream2) -> bool {
    syn::parse2::<syn::ItemImpl>(item.clone()).is_ok()
        || syn::parse2::<syn::ItemTrait>(item.clone()).is_ok()
}

fn is_struct(item: &TokenStream2) -> bool {
    syn::parse2::<syn::ItemStruct>(item.clone()).is_ok()
}

/// Implements boilerplate for a factory module.
#[proc_macro_attribute]
pub fn factory(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr: TokenStream2 = attr.into();
    let item: TokenStream2 = item.into();

    if is_struct(&item) {
        return match ModuleStructIR::try_from((&attr, &item)) {
            Ok(ir) => FactoryModuleStructItem::try_from(&ir).into_code(),
            Err(e) => e.to_compile_error().into()
        };
    }
    if is_impl_or_trait(&item) {
        return match ModuleImplIR::try_from((&attr, &item)) {
            Ok(ir) => FactoryModuleImplItem::try_from(&ir).into_code(),
            Err(e) => e.to_compile_error().into()
        };
    }
    errors::factory_on_other_item(&item)
        .to_compile_error()
        .into()
}

/// Implements boilerplate for a type to be used in an Odra module.
///
/// This macro implements serialization and deserialization for the type, as well as
/// cloning and [HasEvents](../odra_core/contract_def/trait.HasEvents.html) trait.
#[proc_macro_attribute]
pub fn odra_type(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = item.into();
    TypeIR::try_from(&item)
        .and_then(|ir| OdraTypeAttrItem::try_from(&ir))
        .into_code()
}

/// Implements `Into<OdraError>` for an error enum.
#[proc_macro_attribute]
pub fn odra_error(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = item.into();
    TypeIR::try_from(&item)
        .and_then(|ir| OdraErrorAttrItem::try_from(&ir))
        .into_code()
}

/// Provides implementation of a reference to an external contract.
///
/// If you don't have access to the contract source code, but want to call it,
/// you can create a reference to it and interact exactly the same way as with a contract
/// written using [macro@module] macro.
#[proc_macro_attribute]
pub fn external_contract(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr: TokenStream2 = attr.into();
    let item: TokenStream2 = item.into();

    if let Ok(item_impl) = syn::parse2::<syn::ItemImpl>(item.clone()) {
        return errors::external_contract_on_impl(&item_impl)
            .to_compile_error()
            .into();
    }
    match ModuleImplIR::try_from((&attr, &item)) {
        Ok(ir) => ExternalContractImpl::try_from(&ir).into_code(),
        Err(e) => e.to_compile_error().into()
    }
}

/// Adds the same helper functions to the generated `XxxContractRef` and `XxxHostRef`.
///
/// Apply it to an inherent impl block named after a module or an `#[odra::external_contract]`
/// trait. The block is not emitted for `Xxx` itself: its items are copied into
/// `impl XxxContractRef` and into `impl XxxHostRef` (the latter only outside wasm), so a
/// helper written once can be called with method syntax in a contract and in a test.
///
/// ```ignore
/// #[odra::ref_helpers]
/// impl NameToken {
///     pub fn metadata_by_hash(&self, hash: String) -> String {
///         self.metadata(Maybe::None, Maybe::Some(hash))
///     }
/// }
/// ```
///
/// The bodies may use only what both refs have - the entry points. A helper calling an entry point
/// that takes `&mut self` has to take `&mut self` too.
#[proc_macro_attribute]
pub fn ref_helpers(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr: TokenStream2 = attr.into();
    if !attr.is_empty() {
        return errors::ref_helpers_with_args(&attr)
            .to_compile_error()
            .into();
    }
    let item: TokenStream2 = item.into();
    RefHelpersItem::try_from(&item).into_code()
}

/// This macro is used to implement the boilerplate code for the event and contract schema.
#[proc_macro_attribute]
pub fn event(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input: TokenStream2 = input.into();
    OdraEventItem::try_from(&input).into_code()
}

/// Implements `Into<odra::casper_types::RuntimeArgs>` for a struct.
///
/// This macro is used to convert a struct into a `RuntimeArgs` object, one named field per
/// argument. Applied to an enum, a union or a tuple struct, it is a compile error.
#[proc_macro_derive(IntoRuntimeArgs)]
pub fn derive_into_runtime_args(item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as syn::DeriveInput);
    into_runtime_args(&item).into_code()
}

fn into_runtime_args(item: &syn::DeriveInput) -> syn::Result<TokenStream2> {
    let syn::Data::Struct(syn::DataStruct { fields, .. }) = &item.data else {
        return Err(errors::runtime_args_on_non_struct(item));
    };
    let names = match fields {
        syn::Fields::Named(named) => named.named.iter().filter_map(|f| f.ident.as_ref()),
        syn::Fields::Unnamed(_) => return Err(errors::runtime_args_on_unnamed_fields(fields)),
        syn::Fields::Unit => return Ok(into_runtime_args_impl(&item.ident, [].into_iter()))
    };
    Ok(into_runtime_args_impl(&item.ident, names))
}

fn into_runtime_args_impl<'a>(
    ident: &syn::Ident,
    names: impl Iterator<Item = &'a syn::Ident>
) -> TokenStream2 {
    let args_ident = syn::Ident::new("args", ident.span());
    let inserts = names.map(|name| {
        quote::quote!(odra::args::EntrypointArgument::insert_runtime_arg(self.#name, stringify!(#name), &mut #args_ident);)
    });
    quote::quote! {
        impl Into<odra::casper_types::RuntimeArgs> for #ident {
            fn into(self) -> odra::casper_types::RuntimeArgs {
                let mut #args_ident = odra::casper_types::RuntimeArgs::new();
                #(#inserts)*
                #args_ident
            }
        }
    }
}
