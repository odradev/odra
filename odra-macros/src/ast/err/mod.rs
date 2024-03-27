use quote::ToTokens;

pub struct OdraErrorItem {
    item: syn::Item,
}

impl ToTokens for OdraErrorItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let item = &self.item;
        let item = quote::quote! {
            #[derive(odra::OdraType, PartialEq, Eq, Debug)]
            #item
        };

        item.to_tokens(tokens);
    }
}

impl TryFrom<&proc_macro2::TokenStream> for OdraErrorItem {
    type Error = syn::Error;

    fn try_from(code: &proc_macro2::TokenStream) -> Result<Self, Self::Error> {
        Ok(Self {
            item: syn::parse2::<syn::Item>(code.clone())?
        })
    }
}
