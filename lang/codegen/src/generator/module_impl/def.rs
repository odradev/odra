use derive_more::From;
use odra_ir::module::ModuleImpl;
use quote::quote;

use crate::GenerateCode;

#[derive(From)]
pub struct ContractDef<'a> {
    contract: &'a ModuleImpl,
}

as_ref_for_contract_impl_generator!(ContractDef);

impl GenerateCode for ContractDef<'_> {
    fn generate_code(&self) -> proc_macro2::TokenStream {
        let struct_ident = self.contract.ident();
        let struct_name = struct_ident.to_string();

        let entrypoints = self.contract.public_methods();

        quote! {
            impl odra_primitives::contract_def::HasContractDef for #struct_ident {
                fn contract_def() -> odra_primitives::contract_def::ContractDef {
                    odra_primitives::contract_def::ContractDef {
                        ident: String::from(#struct_name),
                        entrypoints: vec![
                            # (#entrypoints)*
                        ],
                    }
                }
            }
        }
    }
}
