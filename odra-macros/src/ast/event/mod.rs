use std::collections::HashSet;
use quote::ToTokens;
use syn::{punctuated::Punctuated, token::Comma};

pub struct OdraEventItem {
    item_struct: syn::ItemStruct,
}

impl ToTokens for OdraEventItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let item = &self.item_struct;
        let ident = &item.ident;
        let name = ident.to_string();
        let fields = item.fields.iter()
            .map(|f| {
                let ident = f.ident.as_ref().unwrap();
                let ty = &f.ty;
                quote::quote!(#ident: #ty)
            })
            .collect::<Punctuated<_, Comma>>();
        let field_names = item.fields.iter().map(|f| f.ident.as_ref().unwrap()).collect::<Punctuated<_, Comma>>();
        let comment = format!("Creates a new instance of the {} event.", ident);
        let doc_attr = quote::quote!(#[doc = #comment]);

        let mut tmp = HashSet::<String>::new();
        let mut chain = vec![];

        item.fields
            .iter()
            .for_each(|f| {
                let ty = &f.ty;
                let v = quote::quote!(.chain(<#ty as odra::schema::SchemaEvents>::custom_types()));
                if tmp.insert(v.to_string()) {
                    chain.push(v);
                }
            });


        let self_item = custom_struct(&name, &item.fields);

        let item = quote::quote! {
            #[derive(odra::Event, PartialEq, Eq, Debug)]
            #item

            impl #ident {
                #doc_attr
                pub fn new(#fields) -> Self {
                    Self {
                        #field_names
                    }
                }
            }

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