use super::checked_unwrap;
use crate::ir::{FnIR, ModuleIR};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse_quote;

pub struct RefItem<'a> {
    module: &'a ModuleIR
}

impl<'a> RefItem<'a> {
    pub fn new(module: &'a ModuleIR) -> Self {
        RefItem { module }
    }

    pub fn method(&self, fun: &FnIR) -> syn::ItemFn {
        let fun_name = fun.name();
        let fun_name_str = fun.name_str();
        let args = fun.arg_names();
        let typed_args = {
            let args = fun.typed_args();
            quote!(#(, #args)*)
        };

        let args = if fun.args_len() == 0 {
            quote!(odra::types::RuntimeArgs::new())
        } else {
            let args = args
                .iter()
                .map(|i| quote!(let _ = named_args.insert(stringify!(#i), #i);))
                .collect::<TokenStream>();
            quote!({
                let mut named_args = odra::types::RuntimeArgs::new();
                #args
                named_args
            })
        };

        let return_type = fun.return_type();
        let mutability = fun.is_mut().then(|| quote!(mut));
        parse_quote!(
            pub fn #fun_name(& #mutability self #typed_args) #return_type {
                self.env.call_contract(
                    self.address,
                    CallDef::new(String::from(#fun_name_str), #args),
                )
            }
        )
    }

    pub fn methods(&self) -> Vec<syn::ItemFn> {
        self.module
            .methods()
            .iter()
            .map(|f| self.method(f))
            .collect()
    }
}

impl<'a> ToTokens for RefItem<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let module_ref = checked_unwrap!(self.module.contract_ref_ident());
        let methods = self.methods();
        tokens.extend(quote!(
            pub struct #module_ref {
                env: Rc<odra::ContractEnv>,
                address: odra::types::Address,
            }

            impl #module_ref {
                pub fn address(&self) -> &Address {
                    &self.address
                }

                #(#methods)*
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
                env: Rc<odra::ContractEnv>,
                address: odra::types::Address,
            }

            impl Erc20ContractRef {
                // TODO: this means "address", can't be entrypoint name.
                pub fn address(&self) -> &Address {
                    &self.address
                }

                pub fn init(&mut self, total_supply: Option<U256>) {
                    self.env.call_contract(
                        self.address,
                        CallDef::new(
                            String::from("init"),
                            {
                                let mut named_args = odra::types::RuntimeArgs::new();
                                let _ = named_args.insert(stringify!(total_supply), total_supply);
                                named_args
                            }
                        ),
                    )
                }

                pub fn total_supply(&self) -> U256 {
                    self.env.call_contract(
                        self.address,
                        CallDef::new(
                            String::from("total_supply"),
                            odra::types::RuntimeArgs::new(),
                        ),
                    )
                }
            }
        };
        let actual = RefItem::new(&module);
        test_utils::assert_eq(actual, expected);
    }

    #[test]
    fn method() {
        let module = test_utils::mock_module();
        let expected = quote! {
            pub fn init(&mut self, total_supply: Option<U256>) {
                self.env.call_contract(
                    self.address,
                    CallDef::new(
                        String::from("init"),
                        {
                            let mut named_args = odra::types::RuntimeArgs::new();
                            let _ = named_args.insert(stringify!(total_supply), total_supply);
                            named_args
                        }
                    ),
                )
            }
        };
        let actual = RefItem::new(&module).method(&module.methods()[0]);
        test_utils::assert_eq(actual, expected);
    }
}
