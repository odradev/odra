use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use std::collections::HashSet;

use crate::ir::errors;

/// Emits everything `#[odra::event]` adds to an event struct: the derives, the
/// `casper_event_standard` impls (serialization and schema), a `new` constructor, and the
/// schema impls naming the event's custom type and those of its fields.
pub struct OdraEventItem {
    item_struct: syn::ItemStruct,
    /// The fields, named: an event is a struct with named fields.
    fields: Vec<(Ident, syn::Type)>
}

impl ToTokens for OdraEventItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let item = &self.item_struct;
        let ident = &item.ident;
        let name = ident.to_string();
        let field_idents = self.fields.iter().map(|(ident, _)| ident).collect::<Vec<_>>();
        let field_tys = self.fields.iter().map(|(_, ty)| ty).collect::<Vec<_>>();
        let comment = format!("Creates a new instance of the {} event.", ident);
        let doc_attr = quote!(#[doc = #comment]);

        // The custom types of each field type, each type once.
        let mut seen = HashSet::<String>::new();
        let chain = field_tys
            .iter()
            .map(|ty| quote!(.chain(<#ty as odra::schema::SchemaEvents>::custom_types())))
            .filter(|chained| seen.insert(chained.to_string()))
            .collect::<Vec<_>>();

        let self_item = custom_struct(&name, &self.fields);
        let event_impls = event_impls(ident, &self.fields);

        let item = quote! {
            #[derive(PartialEq, Eq, Debug)]
            #item

            #event_impls

            impl #ident {
                #doc_attr
                pub fn new(#(#field_idents: #field_tys),*) -> Self {
                    Self {
                        #(#field_idents),*
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

impl TryFrom<&TokenStream> for OdraEventItem {
    type Error = syn::Error;

    fn try_from(code: &TokenStream) -> Result<Self, Self::Error> {
        let item = syn::parse2::<syn::Item>(code.clone())?;
        let syn::Item::Struct(item_struct) = item else {
            return Err(errors::event_on_other_item(&item));
        };
        let syn::Fields::Named(named) = &item_struct.fields else {
            return Err(errors::event_fields_unnamed(&item_struct.fields));
        };
        let fields = named
            .named
            .iter()
            .filter_map(|f| Some((f.ident.clone()?, f.ty.clone())))
            .collect();
        Ok(Self { item_struct, fields })
    }
}

/// The impls `#[derive(odra::Event)]` (`casper_event_standard::Event`) generates, with the same
/// serialization and schema, but with prefixed locals: the derive binds each field to a local
/// named like it next to its own `bytes`, so a field named `bytes` did not compile.
fn event_impls(ident: &Ident, fields: &[(Ident, syn::Type)]) -> TokenStream {
    // Paths as in the derive: `casper_event_standard` must be in scope (`odra::prelude` has it).
    let ces = quote!(casper_event_standard);
    let types = quote!(#ces::casper_types);
    let event_name = format!("event_{}", ident);
    let name = ident.to_string();
    let field_idents = fields.iter().map(|(ident, _)| ident).collect::<Vec<_>>();
    let field_tys = fields.iter().map(|(_, ty)| ty).collect::<Vec<_>>();
    let field_names = field_idents
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>();

    quote! {
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

fn custom_struct(name: &str, fields: &[(Ident, syn::Type)]) -> TokenStream {
    let members = fields.iter().map(|(ident, ty)| {
        let name = ident.to_string();
        quote! {
            odra::schema::struct_member::<#ty>(#name),
        }
    });

    quote!(odra::schema::custom_struct(#name, odra::prelude::vec![#(#members)*]))
}

#[cfg(test)]
mod tests;
