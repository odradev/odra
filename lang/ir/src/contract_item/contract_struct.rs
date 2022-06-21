use quote::{quote, ToTokens};

pub struct ContractStruct {
    item: syn::ItemStruct,
}

impl From<syn::ItemStruct> for ContractStruct {
    fn from(item: syn::ItemStruct) -> Self {
        Self { item }
    }
}

impl ToTokens for ContractStruct {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let item_struct = &self.item;
        tokens.extend(quote! {
            #[odra::instance]
            #item_struct
        });
    }
}
