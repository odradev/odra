use quote::{quote, ToTokens};

use crate::attrs::partition_attributes;

pub struct ContractStruct {
    is_instantiable: bool,
    item: syn::ItemStruct,
}

impl From<syn::ItemStruct> for ContractStruct {
    fn from(item: syn::ItemStruct) -> Self {
        let (_, other_attrs) = partition_attributes(item.attrs).unwrap();
        Self {
            is_instantiable: true,
            item: syn::ItemStruct {
                attrs: other_attrs,
                ..item
            },
        }
    }
}

impl ToTokens for ContractStruct {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let item_struct = &self.item;
        let span = item_struct.ident.span();
        let instance = match &self.is_instantiable {
            true => quote::quote_spanned!(span => #[odra::instance]),
            false => quote!(),
        };
        tokens.extend(quote! {
            #instance
            #item_struct
        });
    }
}
