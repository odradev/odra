use super::checked_unwrap;
use crate::ir::{ref_utils, FnIR, ModuleIR};
use quote::{quote, ToTokens};
use syn::parse_quote;

const CONSTRUCTOR_NAME: &str = "init";
const TRY_CONSTRUCTOR_NAME: &str = "try_init";

pub struct HostRefItem<'a> {
    module: &'a ModuleIR
}

impl<'a> HostRefItem<'a> {
    pub fn new(module: &'a ModuleIR) -> Self {
        Self { module }
    }

    fn try_function(fun: &FnIR) -> syn::ItemFn {
        let signature = ref_utils::try_function_signature(fun);
        let call_def = ref_utils::call_def(fun);

        Self::function_call(signature, call_def)
    }

    fn function(fun: &FnIR) -> syn::ItemFn {
        let signature = ref_utils::function_signature(fun);
        // init does not have a matching try_init function.
        if &fun.name_str() == CONSTRUCTOR_NAME {
            let call_def = ref_utils::call_def(fun);
            Self::function_call(signature, call_def)
        } else {
            let try_func_name = fun.try_name();
            let args = fun.arg_names();
            parse_quote!(
                pub #signature {
                    self.#try_func_name(#(#args),*).unwrap()
                }
            )
        }
    }

    fn function_call(signature: syn::Signature, call_def: syn::Expr) -> syn::ItemFn {
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
        self.module
            .functions()
            .iter()
            .map(|f| vec![Self::try_function(f), Self::function(f)])
            .flatten()
            .filter(|f| &f.sig.ident.to_string() != TRY_CONSTRUCTOR_NAME)
            .collect()
    }
}

impl<'a> ToTokens for HostRefItem<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let module_ref = checked_unwrap!(self.module.host_ref_ident());
        let functions = self.functions();
        tokens.extend(quote!(
            pub struct #module_ref {
                pub address: odra2::types::Address,
                pub env: odra2::HostEnv,
                pub attached_value: odra2::types::U512
            }

            impl #module_ref {
                pub fn with_tokens(&self, tokens: odra2::types::U512) -> Self {
                    Self {
                        address: self.address,
                        env: self.env.clone(),
                        attached_value: tokens
                    }
                }

                #(#functions)*
            }
        ));
    }
}

#[cfg(test)]
mod ref_item_tests {
    use super::HostRefItem;
    use crate::test_utils;
    use quote::quote;

    #[test]
    fn contract_ref() {
        let module = test_utils::mock_module();
        let expected = quote! {
            pub struct Erc20HostRef {
                pub address: odra2::types::Address,
                pub env: odra2::HostEnv,
                pub attached_value: odra2::types::U512
            }

            impl Erc20HostRef {
                // TODO: this means "with_tokens", can't be entrypoint name.
                pub fn with_tokens(&self, tokens: odra2::types::U512) -> Self {
                    Self {
                        address: self.address,
                        env: self.env.clone(),
                        attached_value: tokens
                    }
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

                pub fn try_total_supply(&self) -> Result<U256, OdraError> {
                    self.env.call_contract(
                        self.address,
                        odra2::CallDef::new(
                            String::from("total_supply"),
                            odra2::types::RuntimeArgs::new(),
                        ),
                    )
                }

                pub fn total_supply(&self) -> U256 {
                    self.try_total_supply().unwrap()
                }
            }
        };
        let actual = HostRefItem::new(&module);
        test_utils::assert_eq(actual, expected);
    }

    #[test]
    fn function() {
        let module = test_utils::mock_module();
        let expected = quote! {
            pub fn total_supply(&self) -> U256 {
                self.try_total_supply().unwrap()
            }
        };
        let actual = HostRefItem::function(&module.functions()[1]);
        test_utils::assert_eq(actual, expected);

        let expected = quote! {
            pub fn try_total_supply(&self) -> Result<U256, OdraError> {
                self.env.call_contract(
                    self.address,
                    odra2::CallDef::new(
                        String::from("total_supply"),
                        odra2::types::RuntimeArgs::new()
                    ),
                )
            }
        };
        let actual = HostRefItem::try_function(&module.functions()[1]);
        test_utils::assert_eq(actual, expected);
    }
}
