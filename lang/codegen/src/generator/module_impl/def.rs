use derive_more::From;
use odra_ir::module::ModuleImpl;
use quote::quote;

use crate::GenerateCode;

#[derive(From)]
pub struct ContractDef<'a> {
    contract: &'a ModuleImpl
}

as_ref_for_contract_impl_generator!(ContractDef);

impl GenerateCode for ContractDef<'_> {
    fn generate_code(&self) -> proc_macro2::TokenStream {
        let struct_ident = self.contract.ident();
        let struct_name = struct_ident.to_string();

        let entrypoints = self.contract.public_custom_impl_items();

        quote! {
            #[cfg(feature = "casper")]
            impl odra::types::contract_def::HasIdent for #struct_ident {
                fn ident() -> String {
                    String::from(#struct_name)
                }
            }
            #[cfg(feature = "casper")]
            impl odra::types::contract_def::HasEntrypoints for #struct_ident {
                fn entrypoints() -> Vec<odra::types::contract_def::Entrypoint> {
                    vec![# (#entrypoints)*]
                }
            }
        }
    }
}
