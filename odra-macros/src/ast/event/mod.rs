use quote::ToTokens;

pub struct OdraEventItem {
    item_struct: syn::ItemStruct,
}

impl ToTokens for OdraEventItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let item = &self.item_struct;
        let ident = &item.ident;
        let name = ident.to_string();

        let chain = item.fields
            .iter()
            .map(|f| {
                let ty = &f.ty;
                quote::quote!(.chain(<#ty as odra::schema::SchemaEvents>::custom_types()))
            })
            .collect::<Vec<_>>();

        let self_item = custom_struct(&name, &item.fields);

        let item = quote::quote! {
            #[derive(odra::Event, PartialEq, Eq, Debug)]
            #item

            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::NamedCLTyped for #ident {
                fn ty() -> odra::schema::casper_contract_schema::NamedCLType {
                    odra::schema::casper_contract_schema::NamedCLType::Custom(odra::prelude::String::from(#name))
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaCustomTypes for #ident {
                fn schema_types() -> odra::prelude::vec::Vec<Option<odra::schema::casper_contract_schema::CustomType>> {
                    odra::prelude::BTreeSet::<Option<odra::schema::casper_contract_schema::CustomType>>::new()
                        .into_iter()
                        .chain(odra::prelude::vec![Some(#self_item)])
                        #(#chain)*
                        .collect()
                }
            }
        };

        item.to_tokens(tokens);
    }
}

impl TryFrom<&proc_macro2::TokenStream> for OdraEventItem {
    type Error = syn::Error;

    fn try_from(code: &proc_macro2::TokenStream) -> Result<Self, Self::Error> {
        Ok(Self {
            item_struct: syn::parse2::<syn::ItemStruct>(code.clone())?
        })
    }
}

fn custom_struct(name: &str, fields: &syn::Fields) -> proc_macro2::TokenStream {
    let members = fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap().to_string();
        let ty = &f.ty;
        quote::quote! {
            odra::schema::struct_member::<#ty>(#name),
        }
    });

    quote::quote!(odra::schema::custom_struct(#name, odra::prelude::vec![#(#members)*]))
}