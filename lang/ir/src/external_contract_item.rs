use proc_macro2::TokenStream;

pub struct ExternalContractItem {
    item_trait: syn::ItemTrait
}

impl ExternalContractItem {
    pub fn parse(_attr: TokenStream, item: TokenStream) -> Result<Self, syn::Error> {
        let item_trait = syn::parse2::<syn::ItemTrait>(item)?;

        Ok(Self { item_trait })
    }

    pub fn item_trait(&self) -> &syn::ItemTrait {
        &self.item_trait
    }
}