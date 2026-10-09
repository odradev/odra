use quote::ToTokens;
use std::collections::HashSet;
use syn::{punctuated::Punctuated, token::Comma};

pub struct OdraEventItem {
    item_struct: syn::ItemStruct
}

impl ToTokens for OdraEventItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let item = &self.item_struct;
        let ident = &item.ident;
        let name = ident.to_string();
        let fields = item
            .fields
            .iter()
            .map(|f| {
                let ident = f.ident.as_ref().unwrap();
                let ty = &f.ty;
                quote::quote!(#ident: #ty)
            })
            .collect::<Punctuated<_, Comma>>();
        let field_names = item
            .fields
            .iter()
            .map(|f| f.ident.as_ref().unwrap())
            .collect::<Punctuated<_, Comma>>();
        let comment = format!("Creates a new instance of the {} event.", ident);
        let doc_attr = quote::quote!(#[doc = #comment]);

        let mut tmp = HashSet::<String>::new();
        let mut chain = vec![];

        item.fields.iter().for_each(|f| {
            let ty = &f.ty;
            let v = quote::quote!(.chain(<#ty as odra::schema::SchemaEvents>::custom_types()));
            if tmp.insert(v.to_string()) {
                chain.push(v);
            }
        });

        let self_item = custom_struct(&name, &item.fields);
        let event_impls = event_impls(ident, &item.fields);

        let item = quote::quote! {
            #[derive(PartialEq, Eq, Debug)]
            #item

            #event_impls

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

/// The impls `#[derive(odra::Event)]` (`casper_event_standard::Event`) generates, with the same
/// serialization and schema, but with prefixed locals: the derive binds each field to a local
/// named like it next to its own `bytes`, so a field named `bytes` did not compile.
fn event_impls(ident: &syn::Ident, fields: &syn::Fields) -> proc_macro2::TokenStream {
    // Paths as in the derive: `casper_event_standard` must be in scope (`odra::prelude` has it).
    let ces = quote::quote!(casper_event_standard);
    let types = quote::quote!(#ces::casper_types);
    let event_name = format!("event_{}", ident);
    let name = ident.to_string();
    let field_idents = fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect::<Vec<_>>();
    let field_tys = fields.iter().map(|f| &f.ty).collect::<Vec<_>>();
    let field_names = field_idents
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>();

    quote::quote! {
        impl #types::CLTyped for #ident {
            fn cl_type() -> #types::CLType {
                #types::CLType::Any
            }
        }

        impl #types::bytesrepr::ToBytes for #ident {
            fn to_bytes(&self) -> Result<#ces::alloc::vec::Vec<u8>, #types::bytesrepr::Error> {
                use #types::bytesrepr::ToBytes as _;
                let mut __odra_vec = #ces::alloc::vec::Vec::with_capacity(self.serialized_length());
                __odra_vec.append(&mut #event_name.to_bytes()?);
                #(
                    #ces::validate_type(&self.#field_idents)?;
                    __odra_vec.extend(self.#field_idents.to_bytes()?);
                )*
                Ok(__odra_vec)
            }

            fn serialized_length(&self) -> usize {
                use #types::bytesrepr::ToBytes as _;
                let mut __odra_size = 0;
                __odra_size += #event_name.serialized_length();
                #(__odra_size += self.#field_idents.serialized_length();)*
                __odra_size
            }
        }

        impl #types::bytesrepr::FromBytes for #ident {
            fn from_bytes(__odra_bytes: &[u8]) -> Result<(Self, &[u8]), #types::bytesrepr::Error> {
                let (_, __odra_bytes): (#ces::alloc::string::String, &[u8]) =
                    #types::bytesrepr::FromBytes::from_bytes(__odra_bytes)?;
                #(
                    let (#field_idents, __odra_bytes) =
                        #types::bytesrepr::FromBytes::from_bytes(__odra_bytes)?;
                )*
                Ok((#ident { #(#field_idents,)* }, __odra_bytes))
            }
        }

        impl #ces::EventInstance for #ident {
            fn name() -> #ces::alloc::string::String {
                #ces::alloc::string::String::from(#name)
            }

            fn schema() -> #ces::Schema {
                let mut __odra_schema = #ces::Schema::new();
                #(__odra_schema.with_elem(#field_names, <#field_tys as #types::CLTyped>::cl_type());)*
                __odra_schema
            }
        }
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

#[cfg(test)]
mod tests {
    use super::OdraEventItem;
    use crate::test_utils;
    use quote::quote;

    #[test]
    fn event() {
        let item = OdraEventItem::try_from(&quote!(
            pub struct Transfer {
                pub bytes: U256,
                pub to: Address
            }
        ))
        .unwrap();
        let expected = quote! {
            #[derive(PartialEq, Eq, Debug)]
            pub struct Transfer {
                pub bytes: U256,
                pub to: Address
            }

            impl casper_event_standard::casper_types::CLTyped for Transfer {
                fn cl_type() -> casper_event_standard::casper_types::CLType {
                    casper_event_standard::casper_types::CLType::Any
                }
            }

            impl casper_event_standard::casper_types::bytesrepr::ToBytes for Transfer {
                fn to_bytes(&self) -> Result<casper_event_standard::alloc::vec::Vec<u8>, casper_event_standard::casper_types::bytesrepr::Error> {
                    use casper_event_standard::casper_types::bytesrepr::ToBytes as _;
                    let mut __odra_vec = casper_event_standard::alloc::vec::Vec::with_capacity(self.serialized_length());
                    __odra_vec.append(&mut "event_Transfer".to_bytes()?);
                    casper_event_standard::validate_type(&self.bytes)?;
                    __odra_vec.extend(self.bytes.to_bytes()?);
                    casper_event_standard::validate_type(&self.to)?;
                    __odra_vec.extend(self.to.to_bytes()?);
                    Ok(__odra_vec)
                }

                fn serialized_length(&self) -> usize {
                    use casper_event_standard::casper_types::bytesrepr::ToBytes as _;
                    let mut __odra_size = 0;
                    __odra_size += "event_Transfer".serialized_length();
                    __odra_size += self.bytes.serialized_length();
                    __odra_size += self.to.serialized_length();
                    __odra_size
                }
            }

            impl casper_event_standard::casper_types::bytesrepr::FromBytes for Transfer {
                fn from_bytes(__odra_bytes: &[u8]) -> Result<(Self, &[u8]), casper_event_standard::casper_types::bytesrepr::Error> {
                    let (_, __odra_bytes): (casper_event_standard::alloc::string::String, &[u8]) =
                        casper_event_standard::casper_types::bytesrepr::FromBytes::from_bytes(__odra_bytes)?;
                    let (bytes, __odra_bytes) =
                        casper_event_standard::casper_types::bytesrepr::FromBytes::from_bytes(__odra_bytes)?;
                    let (to, __odra_bytes) =
                        casper_event_standard::casper_types::bytesrepr::FromBytes::from_bytes(__odra_bytes)?;
                    Ok((Transfer { bytes, to, }, __odra_bytes))
                }
            }

            impl casper_event_standard::EventInstance for Transfer {
                fn name() -> casper_event_standard::alloc::string::String {
                    casper_event_standard::alloc::string::String::from("Transfer")
                }

                fn schema() -> casper_event_standard::Schema {
                    let mut __odra_schema = casper_event_standard::Schema::new();
                    __odra_schema.with_elem("bytes", <U256 as casper_event_standard::casper_types::CLTyped>::cl_type());
                    __odra_schema.with_elem("to", <Address as casper_event_standard::casper_types::CLTyped>::cl_type());
                    __odra_schema
                }
            }

            impl Transfer {
                #[doc = "Creates a new instance of the Transfer event."]
                pub fn new(bytes: U256, to: Address) -> Self {
                    Self { bytes, to }
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::NamedCLTyped for Transfer {
                fn ty() -> odra::schema::casper_contract_schema::NamedCLType {
                    odra::schema::casper_contract_schema::NamedCLType::Custom(odra::prelude::String::from("Transfer"))
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaCustomTypes for Transfer {
                fn schema_types() -> odra::prelude::vec::Vec<Option<odra::schema::casper_contract_schema::CustomType>> {
                    odra::prelude::BTreeSet::<Option<odra::schema::casper_contract_schema::CustomType>>::new()
                        .into_iter()
                        .chain(odra::prelude::vec![Some(odra::schema::custom_struct("Transfer", odra::prelude::vec![
                            odra::schema::struct_member::<U256>("bytes"),
                            odra::schema::struct_member::<Address>("to"),
                        ]))])
                        .chain(<U256 as odra::schema::SchemaEvents>::custom_types())
                        .chain(<Address as odra::schema::SchemaEvents>::custom_types())
                        .collect()
                }
            }
        };
        test_utils::assert_eq(item, expected);
    }
}
