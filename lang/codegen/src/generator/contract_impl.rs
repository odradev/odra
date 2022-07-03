use derive_more::From;
use odra_ir::ImplItem;
use quote::ToTokens;

use crate::{GenerateCode, OdraPoetUsingImpl};

use self::{def::ContractDef, deploy::Deploy, refrence::ContractReference};

mod def;
mod deploy;
mod refrence;

#[derive(From)]
pub struct ContractImpl<'a> {
    contract: &'a odra_ir::ContractImpl,
}

as_ref_for_contract_impl_generator!(ContractImpl);

impl GenerateCode for ContractImpl<'_> {
    fn generate_code(&self) -> proc_macro2::TokenStream {
        let ident = self.contract.ident();
        let original_item_impls = self.contract.impl_items().iter().map(|item| match item {
            ImplItem::Constructor(item) => item.impl_item.to_token_stream(),
            ImplItem::Method(item) => item.impl_item.to_token_stream(),
            ImplItem::Other(item) => item.to_token_stream(),
        });

        let contract_def = self.generate_code_using::<ContractDef>();
        let deploy = self.generate_code_using::<Deploy>();
        let contract_ref = self.generate_code_using::<ContractReference>();

        quote::quote! {
            impl #ident {
                # (#original_item_impls)*
            }

            #contract_def

            #deploy

            #contract_ref
        }
    }
}
