use derive_more::From;
use odra_ir::module_item::module_impl::ModuleImpl;
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

        let entrypoints = self.contract.methods();

        quote! {
            impl odra::contract_def::HasContractDef for #struct_ident {
                fn contract_def() -> odra::contract_def::ContractDef {
                    odra::contract_def::ContractDef {
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
