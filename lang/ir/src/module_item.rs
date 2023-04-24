use std::convert::TryFrom;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{parse::Parse, punctuated::Punctuated, Token};

use self::{module_impl::ModuleImpl, module_struct::ModuleStruct};

pub mod constructor;
pub mod delegate;
pub mod impl_item;
pub mod method;
pub mod module_impl;
pub mod module_struct;
mod utils;

/// Odra module item.
///
/// Can be either
/// - an Odra [`odra_ir::module::ModuleStruct`](`crate::module::ModuleStruct`)
/// - an Odra [`odra_ir::module::ModuleImpl`](`crate::module::ModuleImpl`)
///
/// All the items are based on syn with special variants for Odra `impl` items.
pub enum ModuleItem {
    Struct(Box<ModuleStruct>),
    Impl(Box<ModuleImpl>)
}

impl ModuleItem {
    pub fn parse(_attr: TokenStream, item: TokenStream) -> Result<Self, syn::Error> {
        let config = syn::parse2::<ModuleConfiguration>(_attr)?;

        let item_struct = syn::parse2::<syn::ItemStruct>(item.clone());
        let item_impl = syn::parse2::<syn::ItemImpl>(item.clone());

        if let Ok(item) = item_struct {
            let module_struct = ModuleStruct::from(item).with_config(config)?;
            return Ok(ModuleItem::Struct(Box::new(module_struct)));
        }

        if let Ok(item) = item_impl {
            let item = ModuleImpl::try_from(item)?;
            return Ok(ModuleItem::Impl(Box::new(item)));
        }

        Err(syn::Error::new_spanned(
            item,
            "ContractItem is neither a struct nor an impl block."
        ))
    }
}

mod kw {
    syn::custom_keyword!(events);
    syn::custom_keyword!(skip_instance);
}

#[derive(Debug, Default, Clone)]
pub struct ModuleConfiguration {
    pub events: ModuleEvents,
    pub skip_instance: bool
}

impl Parse for ModuleConfiguration {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut events = None;
        let mut skip_instance = None;

        while !input.is_empty() {
            if events.is_none() && input.peek(kw::events) {
                events = Some(input.parse::<ModuleEvents>()?);
                let _ = input.parse::<Token![,]>(); // optional comma
                continue;
            }

            if skip_instance.is_none() && input.peek(kw::skip_instance) {
                input.parse::<kw::skip_instance>()?;
                skip_instance = Some(true);
                let _ = input.parse::<Token![,]>();
                continue;
            }

            return Err(input.error("Unexpected token"));
        }

        Ok(Self {
            events: events.unwrap_or_default(),
            skip_instance: skip_instance.unwrap_or_default()
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct ModuleEvents {
    pub events: Punctuated<ModuleEvent, Token![,]>,
    pub submodules_events: Punctuated<ModuleEvent, Token![,]>,
    pub mappings_events: Punctuated<ModuleEvent, Token![,]>
}

impl Parse for ModuleEvents {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // a sample input: events = [Event1, Event2, Event3]
        if input.is_empty() {
            return Ok(Self::default());
        }
        input.parse::<kw::events>()?;
        input.parse::<Token![=]>()?;

        let content;
        let _brace_token = syn::bracketed!(content in input);
        let events = content.parse_terminated::<ModuleEvent, Token![,]>(ModuleEvent::parse)?;
        Ok(Self {
            events,
            submodules_events: Default::default(),
            mappings_events: Default::default()
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModuleEvent {
    pub name: syn::Ident
}

impl Parse for ModuleEvent {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse::<syn::Ident>()?;
        Ok(ModuleEvent { name })
    }
}

impl ToTokens for ModuleEvent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.name.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;

    use super::*;

    #[test]
    fn invalid_usage() {
        let result = ModuleItem::parse(
            quote!(),
            quote!(
                fn some_fn(x: u32) -> u32 {
                    x + 1
                }
            )
        );
        assert!(result.is_err());

        let result = ModuleItem::parse(
            quote!(),
            quote!(
                enum A {}
            )
        );
        assert!(result.is_err());
    }

    #[test]
    fn struct_block() {
        let result = ModuleItem::parse(
            quote!(),
            quote!(
                struct ContractItem {
                    x: u32,
                    name: String
                }
            )
        );
        assert!(result.is_ok())
    }

    #[test]
    fn impl_block() {
        let result = ModuleItem::parse(
            quote!(),
            quote!(
                impl ContractItem {
                    fn a() {}
                }
            )
        );
        assert!(result.is_ok())
    }
}
