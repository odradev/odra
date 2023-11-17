use super::checked_unwrap;
use crate::{
    ir::{ref_utils, FnIR, ModuleIR},
    syn_utils
};
use quote::{quote, ToTokens};
use syn::parse_quote;

pub struct RefItem<'a> {
    module: &'a ModuleIR
}

impl<'a> RefItem<'a> {
    pub fn new(module: &'a ModuleIR) -> Self {
        RefItem { module }
    }

    fn function(fun: &FnIR) -> syn::ItemFn {
        let signature = ref_utils::function_signature(fun);
        let call_def = ref_utils::call_def(fun);

        parse_quote!(
            pub #signature {
                self.env.call_contract(
                    self.address,
                    #call_def
                )
            }
        )
    }

    fn functions(&self) -> Vec<syn::ItemFn> {
        self.module.functions().iter().map(Self::function).collect()
    }
}

impl<'a> ToTokens for RefItem<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let module_ref = checked_unwrap!(self.module.contract_ref_ident());
        let functions = self.functions();
        let ty_address = syn_utils::type_address();
        let ty_contract_env = syn_utils::type_contract_env();

        tokens.extend(quote!(
            pub struct #module_ref {
                env: Rc<#ty_contract_env>,
                address: #ty_address,
            }

            impl #module_ref {
                pub fn address(&self) -> &#ty_address {
                    &self.address
                }

                #(#functions)*
            }
        ));
    }
}

#[cfg(test)]
mod ref_item_tests {
    use super::RefItem;
    use crate::test_utils;
    use quote::quote;

    #[test]
    fn contract_ref() {
        let module = test_utils::mock_module();
        let expected = quote! {
            pub struct Erc20ContractRef {
                env: Rc<odra2::ContractEnv>,
                address: odra2::types::Address,
            }

            impl Erc20ContractRef {
                // TODO: this means "address", can't be entrypoint name.
                pub fn address(&self) -> &odra2::types::Address {
                    &self.address
                }

                pub fn init(&mut self, total_supply: Option<U256>) {
                    self.env.call_contract(
                        self.address,
                        odra2::CallDef::new(
                            String::from("init"),
                            {
                                let mut named_args = odra2::types::RuntimeArgs::new();
                                let _ = named_args.insert(stringify!(total_supply), total_supply);
                                named_args
                            }
                        ),
                    )
                }

                pub fn total_supply(&self) -> U256 {
                    self.env.call_contract(
                        self.address,
                        odra2::CallDef::new(
                            String::from("total_supply"),
                            odra2::types::RuntimeArgs::new(),
                        ),
                    )
                }
            }
        };
        let actual = RefItem::new(&module);
        test_utils::assert_eq(actual, expected);
    }

    #[test]
    fn function() {
        let module = test_utils::mock_module();
        let expected = quote! {
            pub fn init(&mut self, total_supply: Option<U256>) {
                self.env.call_contract(
                    self.address,
                    odra2::CallDef::new(
                        String::from("init"),
                        {
                            let mut named_args = odra2::types::RuntimeArgs::new();
                            let _ = named_args.insert(stringify!(total_supply), total_supply);
                            named_args
                        }
                    ),
                )
            }
        };
        let actual = RefItem::function(&module.functions()[0]);
        test_utils::assert_eq(actual, expected);
    }
}
