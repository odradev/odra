#![doc = "This crate provides a set of macros and derive attributes to simplify the process of writing"]
#![doc = "smart contracts for the Odra platform."]
#![feature(box_patterns, result_flattening)]
use crate::utils::IntoCode;
use ast::*;
use ir::{ModuleImplIR, ModuleStructIR, TypeIR};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

mod ast;
mod ir;
#[cfg(test)]
mod test_utils;
mod utils;

macro_rules! span_error {
    ($span:ident, $msg:expr) => {
        syn::Error::new(syn::spanned::Spanned::span(&$span), $msg)
            .to_compile_error()
            .into()
    };
}

/// Core element of the Odra framework, entry point for writing smart contracts.
///
/// Each module consists of two parts:
/// 1. Module definition - a struct which composition of stored values (Vars and Mappings)
/// and modules.
/// 2. Module implementation - an implementation block.
///
/// The macro produces all the required code to use the module as a standalone smart contract.
#[proc_macro_attribute]
pub fn module(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr: TokenStream2 = attr.into();
    let item: TokenStream2 = item.into();
    if let Ok(ir) = ModuleImplIR::try_from((&attr, &item)) {
        return ModuleImplItem::try_from(&ir).into_code();
    }
    if let Ok(ir) = ModuleStructIR::try_from((&attr, &item)) {
        return ModuleStructItem::try_from(&ir).into_code();
    }
    span_error!(item, "Struct or impl block expected")
}

/// Implements boilerplate for a type to be used in an Odra module.
///
/// This macro implements serialization and deserialization for the type, as well as
/// cloning and [HasEvents](../odra_core/contract_def/trait.HasEvents.html) trait.
#[proc_macro_derive(OdraType)]
pub fn derive_odra_type(item: TokenStream) -> TokenStream {
    let item = item.into();
    if let Ok(ir) = TypeIR::try_from(&item) {
        let schema = SchemaCustomTypeItem::try_from(&ir).into_code();
        let odra_type = OdraTypeItem::try_from(&ir).into_code();
        let mut tokens = TokenStream::new();
        tokens.extend(schema);
        tokens.extend(odra_type);
        return tokens;
    }
    span_error!(item, "Struct or Enum expected")
}

/// Implements `Into<odra::OdraError>` for an error enum.
#[proc_macro_derive(OdraError)]
pub fn derive_odra_error(item: TokenStream) -> TokenStream {
    let item = item.into();
    if let Ok(ir) = TypeIR::try_from(&item) {
        return OdraErrorItem::try_from(&ir).into_code();
    }
    span_error!(item, "Struct or Enum expected")
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
    if let Ok(ir) = ModuleImplIR::try_from((&attr, &item)) {
        return ExternalContractImpl::try_from(&ir).into_code();
    }
    span_error!(
        item,
        "#[external_contract] can be only applied to trait only"
    )
}

// #[proc_macro_derive(SchemaElement)]
// pub fn schema_custom_type(attr: TokenStream, item: TokenStream) -> TokenStream {
//     let attr: TokenStream2 = attr.into();
//     let item: TokenStream2 = item.into();
//     if let Ok(ir) = ModuleImplIR::try_from((&attr, &item)) {
//         return SchemaCustomTypeItem::try_from(&ir).into_code();
//     }
//     span_error!(
//         item,
//         "#[external_contract] can be only applied to trait only"
//     )
// }

/// Implements `Into<odra::casper_types::RuntimeArgs>` for a struct.
///
/// This macro is used to convert a struct into a `RuntimeArgs` object.
/// If applied to an enum or a union type, it will panic.
#[proc_macro_derive(IntoRuntimeArgs)]
pub fn derive_into_runtime_args(item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as syn::DeriveInput);
    let args_ident = syn::Ident::new("args", item.ident.span());
    match item {
        syn::DeriveInput {
            ident,
            data: syn::Data::Struct(syn::DataStruct { fields, .. }),
            ..
        } => {
            let fields = fields
                .into_iter()
                .map(|f| {
                    let name = f.ident.unwrap();
                    quote::quote!(odra::args::EntrypointArgument::insert_runtime_arg(self.#name, stringify!(#name), &mut #args_ident);)
                })
                .collect::<proc_macro2::TokenStream>();
            let res = quote::quote! {
                impl Into<odra::casper_types::RuntimeArgs> for #ident {
                    fn into(self) -> odra::casper_types::RuntimeArgs {
                        let mut #args_ident = odra::casper_types::RuntimeArgs::new();
                        #fields
                        #args_ident
                    }
                }
            };
            res.into()
        }
        _ => panic!("Struct expected")
    }
}
