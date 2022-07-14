use proc_macro2::TokenStream;

pub struct InstanceItem {
    item_struct: syn::ItemStruct,
}

impl InstanceItem {
    pub fn parse(_attr: TokenStream, item: TokenStream) -> Result<Self, syn::Error> {
        let item_struct = syn::parse2::<syn::ItemStruct>(item)?;

        Ok(Self { item_struct })
    }

    pub fn item_struct(&self) -> &syn::ItemStruct {
        &self.item_struct
    }
}
