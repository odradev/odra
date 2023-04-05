use derive_more::From;
use odra_ir::module::{Constructor, Method, ModuleImpl};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    generator::common::{self, build_ref},
    GenerateCode
};

#[derive(From)]
pub struct ContractReference<'a> {
    contract: &'a ModuleImpl
}

as_ref_for_contract_impl_generator!(ContractReference);

impl GenerateCode for ContractReference<'_> {
    fn generate_code(&self) -> TokenStream {
        let struct_ident = self.contract.ident();
        let ref_ident = format_ident!("{}Ref", struct_ident);

        let ref_entrypoints = build_entrypoints(self.contract.get_method_iter());

        let ref_constructors = build_constructors(self.contract.get_constructor_iter());

        let contract_ref = build_ref(&ref_ident);

        quote! {
            #contract_ref

            impl #ref_ident {
                #ref_entrypoints

                #ref_constructors
            }
        }
    }
}
// check git history for more context
fn build_entrypoints<'a, T>(methods: T) -> TokenStream
where
    T: Iterator<Item = &'a Method>
{
    methods
        .map(|entrypoint| {
            let sig = &entrypoint.full_sig;
            let entrypoint_name = &entrypoint.ident.to_string();
            let fn_body =
                common::generate_fn_body(entrypoint.args.clone(), entrypoint_name, &entrypoint.ret);

            quote! {
                pub #sig {
                    #fn_body
                }
            }
        })
        .collect::<TokenStream>()
}

fn build_constructors<'a, T>(constructors: T) -> TokenStream
where
    T: Iterator<Item = &'a Constructor>
{
    constructors
        .map(|constructor| {
            let sig = &constructor.full_sig;
            let constructor_name = constructor.ident.to_string();
            let fn_body = common::generate_fn_body(
                constructor.args.clone(),
                &constructor_name,
                &syn::ReturnType::Default
            );

            quote! {
                pub #sig {
                    #fn_body
                }
            }
        })
        .collect::<TokenStream>()
}
