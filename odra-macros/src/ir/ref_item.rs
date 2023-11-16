use quote::{quote, ToTokens};
use syn::parse_quote;

use crate::ir::{FnIR, ModuleIR};

pub struct RefItem<'a> {
    module: &'a ModuleIR,
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
        let args = match fun.args_len() {
            0 => quote!(()),
            1 => {
                let arg = args.first().unwrap();
                quote!(#arg)
            }
            _ => quote!((#(#args),*)),
        };

        let return_type = fun.return_type();
        let mutability = fun.is_mut().then(|| quote!(mut));
        parse_quote!(
            pub fn #fun_name(& #mutability self #typed_args) -> #return_type {
                self.env.call_contract(
                    self.address,
                    CallDef::new(String::from(#fun_name_str), #args.into()),
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
        let module_ref = self.module.ref_ident();
        let methods = self.methods();
        tokens.extend(quote!(
            pub struct #module_ref {
                env: Env,
                address: Address,
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
    fn module_ref() {
        let module = test_utils::mock_module();
        let expected = quote! {
            pub struct Erc20Ref {
                env: Env,
                address: Address,
            }

            impl Erc20Ref {
                // TODO: this means "address", can't be entrypoint name.
                pub fn address(&self) -> &Address {
                    &self.address
                }

                pub fn init(&mut self, name: String) -> Void {
                    self.env.call_contract(
                        self.address,
                        CallDef::new(String::from("init"), name.into()),
                    )
                }

                pub fn name(&self) -> OdraResult<String> {
                    self.env
                        .call_contract(self.address, CallDef::new(String::from("name"), ().into()))
                }

                pub fn balance_of(&self, address: Address) -> OdraResult<U256> {
                    self.env.call_contract(
                        self.address,
                        CallDef::new(String::from("balance_of"), address.into()),
                    )
                }

                pub fn mint(&mut self, address: Address, amount: U256) -> Void {
                    self.env.call_contract(
                        self.address,
                        CallDef::new(String::from("mint"), (address, amount).into()),
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
            pub fn init(&mut self, name: String) -> Void {
                self.env.call_contract(
                    self.address,
                    CallDef::new(String::from("init"), name.into()),
                )
            }
        };
        let actual = RefItem::new(&module).method(&module.methods()[0]);
        test_utils::assert_eq(actual, expected);
    }
}