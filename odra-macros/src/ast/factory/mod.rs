use quote::{ToTokens, TokenStreamExt};
use syn::parse_quote;

use crate::{ir::FnIR, ModuleImplIR, ModuleStructIR};

pub mod impl_item;
mod parts;
pub mod struct_item;

pub(crate) use parts::{FactoryDeployImplItem, FactoryHostRefItem, FactoryWasmFnsItem};

/// Emits `#[odra::factory] pub struct XxxFactory;` for a module with `factory = on`: the
/// factory's own struct, expanded by the factory macro in turn. Nothing for other modules.
pub struct FactoryModuleItem {
    is_factory: bool,
    factory_module_ident: syn::Ident,
}

impl TryFrom<&ModuleStructIR> for FactoryModuleItem {
    type Error = syn::Error;

    fn try_from(ir: &ModuleStructIR) -> Result<Self, Self::Error> {
        Ok(Self {
            is_factory: ir.is_factory(),
            factory_module_ident: ir.factory_module_ident(),
        })
    }
}

impl ToTokens for FactoryModuleItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let is_factory = self.is_factory;
        if !is_factory {
            return;
        }

        let factory_module_ident = &self.factory_module_ident;

        tokens.append_all(quote::quote! {
            #[automatically_derived]
            #[odra::factory]
            pub struct #factory_module_ident;
        });
    }
}

/// Emits `#[odra::factory] impl XxxFactory { .. }` for a module with `factory = on`: the
/// module's entry points with bodies that revert, as a factory's instance is never called with
/// them; only their signatures matter, for the child's templates. Nothing for other modules.
pub struct FactoryModuleImplItem {
    is_factory: bool,
    factory_module_ident: syn::Ident,
    /// The functions of the impl block, as written.
    functions: Vec<syn::ImplItemFn>
}

impl TryFrom<&ModuleImplIR> for FactoryModuleImplItem {
    type Error = syn::Error;

    fn try_from(ir: &ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            is_factory: ir.is_factory(),
            factory_module_ident: ir.factory_module_ident(),
            functions: ir
                .functions()
                .iter()
                .filter_map(|f| match f {
                    FnIR::Impl(fn_impl_ir) => Some(fn_impl_ir.raw()),
                    _ => None
                })
                .collect()
        })
    }
}

impl ToTokens for FactoryModuleImplItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let is_factory = self.is_factory;
        if !is_factory {
            return;
        }
        let factory_module_ident = &self.factory_module_ident;

        let functions = &self
            .functions
            .iter()
            .map(|raw| {
                // The body only reverts, so the arguments of the copied signature go unused.
                let mut attrs = raw.attrs.clone();
                attrs.push(parse_quote!(#[allow(unused_variables)]));
                syn::ImplItemFn {
                    attrs,
                    block: parse_quote!({
                        self.env().revert(OdraError::ExecutionError(ExecutionError::FactoryModuleCall));
                    }),
                    ..raw.clone()
                }
            })
            .collect::<Vec<_>>();

        tokens.append_all(quote::quote! {
            #[automatically_derived]
            #[odra::factory]
            impl #factory_module_ident {
                #(#functions)*
            }
        });
    }
}

#[cfg(test)]
mod tests;
