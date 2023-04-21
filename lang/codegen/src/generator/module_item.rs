use derive_more::From;
use quote::{quote, quote_spanned};

use crate::{GenerateCode, generator::module_item::composer::ModuleComposer};

mod composer;

#[derive(From)]
pub struct ModuleStruct<'a> {
    pub contract: &'a odra_ir::module::ModuleStruct
}

impl GenerateCode for ModuleStruct<'_> {
    fn generate_code(&self) -> proc_macro2::TokenStream {
        let item_struct = &self.contract.item;
        let span = item_struct.ident.span();
        let instance = if self.contract.is_instantiable && !self.contract.skip_instance {
            quote_spanned!(span => #[derive(odra::Instance)])
        } else {
            quote!()
        };

        let composer = <ModuleComposer as GenerateCode>::generate_code(&ModuleComposer::from(self.contract));

        quote! {
            #instance
            #item_struct

            #composer
        }
    }
}
