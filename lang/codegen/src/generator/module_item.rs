use derive_more::From;
use quote::quote;

use crate::{generator::common, GenerateCode};

#[derive(From)]
pub struct ModuleStruct<'a> {
    pub contract: &'a odra_ir::module::ModuleStruct
}

impl GenerateCode for ModuleStruct<'_> {
    fn generate_code(&self) -> proc_macro2::TokenStream {
        let item_struct = &self.contract.item;
        let span = item_struct.ident.span();
        let instance = match &self.contract.is_instantiable {
            true => quote::quote_spanned!(span => #[derive(odra::Instance)]),
            false => quote!()
        };

        quote! {
            #instance
            #item_struct
        }
    }
}
