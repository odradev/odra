use quote::ToTokens;

use crate::ir::ModuleIR;

pub struct DeployerItem<'a> {
    module: &'a ModuleIR
}

impl<'a> DeployerItem<'a> {
    pub fn new(module: &'a ModuleIR) -> Self {
        DeployerItem { module }
    }
}

impl<'a> ToTokens for DeployerItem<'a> {
    fn to_tokens(&self, _tokens: &mut proc_macro2::TokenStream) {
        // let module = checked_unwrap!(self.module.module_ident());
        // let module_ref = checked_unwrap!(self.module.host_ref_ident());
        // let module_deployer = checked_unwrap!(self.module.deployer_ident());
        // tokens.extend(quote!(
        //     pub struct #module_deployer;
        //     impl #module_deployer {
        //         pub fn deploy(env: &mut Env) -> #module_ref {
        //             let caller = odra::ModuleCaller(|env: Env, call_def: odra::types::CallDef| {
        //                 let contract = #module;
        //                 odra::Callable::call(&contract, env, call_def)
        //             });
        //             let addr = env.new_contract(caller);
        //             #module_ref {
        //                 env: env.clone_empty(),
        //                 address: addr,
        //             }
        //         }
        //     }
        // ));
    }
}

#[cfg(test)]
mod deployer_impl {
    use super::DeployerItem;
    use crate::test_utils;
    use quote::quote;

    #[test]
    fn deployer_impl() {
        let module = test_utils::mock_module();
        let expected = quote! {
            pub struct Erc20Deployer;

            impl Erc20Deployer {
                pub fn deploy(env: &mut Env) -> Erc20Ref {
                    let caller = odra::ModuleCaller(|env: Env, call_def: odra::types::CallDef| {
                        let contract = Erc20;
                        odra::Callable::call(&contract, env, call_def)
                    });
                    let addr = env.new_contract(caller);
                    Erc20Ref {
                        env: env.clone_empty(),
                        address: addr,
                    }
                }
            }
        };
        let deployer_item = DeployerItem::new(&module);
        test_utils::assert_eq(deployer_item, &expected);
    }
}
