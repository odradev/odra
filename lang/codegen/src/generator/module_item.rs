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
        let item_ident = &item_struct.ident;
        let span = item_struct.ident.span();
        let instance = match &self.contract.is_instantiable {
            true => quote::quote_spanned!(span => #[derive(odra::Instance)]),
            false => quote!()
        };

        let fields = item_struct
            .fields
            .iter()
            .filter_map(|f| f.ident.as_ref().cloned())
            .collect::<Vec<_>>();

        let mock_serde = common::mock_vm::serialize_struct(item_ident, &fields);
        let casper_serde = common::casper::serialize_struct(item_ident, &fields);

        quote! {
            #instance
            #item_struct

            #mock_serde
            #casper_serde
        }
    }
}
