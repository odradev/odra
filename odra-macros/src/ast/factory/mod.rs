use quote::{ToTokens, TokenStreamExt};
use syn::parse_quote;

use crate::{ir::FnIR, ModuleImplIR, ModuleStructIR};

pub mod impl_item;
mod parts;
pub mod struct_item;

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

pub struct FactoryModuleImplItem {
    is_factory: bool,
    factory_module_ident: syn::Ident,
    functions: Vec<FnIR>
}

impl TryFrom<&ModuleImplIR> for FactoryModuleImplItem {
    type Error = syn::Error;

    fn try_from(ir: &ModuleImplIR) -> Result<Self, Self::Error> {
        let functions = ir.functions()?;
        // functions.push(ir.factory_fn());
        Ok(Self {
            is_factory: ir.is_factory(),
            factory_module_ident: ir.factory_module_ident()?,
            functions
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
            .filter_map(|f| match f {
                FnIR::Impl(fn_impl_ir) => Some(fn_impl_ir),
                FnIR::Def(_) => None
            })
            .map(|f| {
                syn::ImplItemFn {
                    block: parse_quote!({
                        panic!("Factory modules cannot have regular methods");
                    }),
                    ..f.raw()
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
mod test {
    use super::*;
    use crate::test_utils::{self, mock};

    #[test]
    fn test_factory_module_item_generation() {
        let ir = mock::factory_module_definition();
        let actual = FactoryModuleItem::try_from(&ir).expect("A valid FactoryModuleItem");

        let expected = quote::quote! {
            #[automatically_derived]
            #[odra::factory]
            pub struct CounterPackFactory;
        };
        test_utils::assert_eq(actual, expected);
    }

    #[test]
    fn test_non_factory_module_item_generation() {
        let ir = mock::module_definition();
        let actual = FactoryModuleItem::try_from(&ir).expect("A valid FactoryModuleItem");

        let expected = quote::quote! {};
        test_utils::assert_eq(actual, expected);
    }

    #[test]
    fn test_factory_module_impl_item_generation() {
        let ir = mock::module_factory_on();
        let actual = FactoryModuleImplItem::try_from(&ir).expect("A valid FactoryModuleImplItem");

        let expected = quote::quote! {
            #[automatically_derived]
            #[odra::factory]
            impl Erc20Factory {
                /// Returns the total supply of the token.
                pub fn total_supply(&self) -> U256 {
                    panic!("Factory modules cannot have regular methods");
                }
                /// Pay to mint.
                #[odra(payable)]
                pub fn pay_to_mint(&mut self) {
                    panic!("Factory modules cannot have regular methods");
                }
            }
        };
        test_utils::assert_eq(actual, expected);
    }
}
