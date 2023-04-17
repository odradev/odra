use crate::attrs::partition_attributes;
use anyhow::{Result, Context};
use proc_macro2::Ident;

use super::{ModuleEvents, ModuleEvent};

/// Odra module struct.
///
/// Wraps up [syn::ItemStruct].
pub struct ModuleStruct {
    pub is_instantiable: bool,
    pub item: syn::ItemStruct,
    pub events: ModuleEvents
}

impl ModuleStruct {
    pub fn with_events(mut self, mut events: ModuleEvents) -> Result<Self, syn::Error> {
        let submodules = self.item
            .fields
            .iter()
            .filter(|field| field.ident.is_some())
            .filter_map(filter_ident_excluding_variable_and_mapping)
            .map(|ident| ModuleEvent { name: ident })
            .collect::<Vec<_>>();

        let mut mappings = self.item
            .fields
            .iter()
            .filter(|field| field.ident.is_some())
            .filter_map(|f| match &f.ty {
                syn::Type::Path(path) => extract_mapping_value_ident_from_path(path).ok(),
                _ => None
            })
            .map(|ident| ModuleEvent { name: ident })
            .collect::<Vec<_>>();
        mappings.dedup();

        events.submodules_events.extend(submodules);
        events.mappings_events.extend(mappings);

        self.events = events;

        Ok(self)
    }
}

impl From<syn::ItemStruct> for ModuleStruct {
    fn from(item: syn::ItemStruct) -> Self {
        let (_, other_attrs) = partition_attributes(item.attrs).unwrap();
        Self {
            is_instantiable: true,
            item: syn::ItemStruct {
                attrs: other_attrs,
                ..item
            },
            events: Default::default()
        }
    }
}

fn extract_mapping_value_ident_from_path(path: &syn::TypePath) -> Result<Ident> {
    // Eg. odra::type::Mapping<String, Mapping<String, Mapping<u8, String>>>
    let mut segment = path.path.segments.last().cloned().context("At least one segment expected")?;
    if segment.ident != "Mapping" {
        return Err(anyhow::anyhow!("Mapping expected but found {}", segment.ident));
    }
    loop {
        let args = &segment.arguments;
        if args.is_empty() {
            break;
        }
        if let syn::PathArguments::AngleBracketed(args) = args {
            match args.args.last().context("syn::GenericArgument expected but not found")? {
                syn::GenericArgument::Type(syn::Type::Path(path))  => {
                    let path = &path.path;
                    segment = path.segments.last().cloned().context("At least one segment expected")?;
                },
                other => return Err(anyhow::anyhow!("syn::TypePath expected but found {:?}", other))
            }
        } else {
            return Err(anyhow::anyhow!("syn::AngleBracketedGenericArguments expected but found {:?}", args));
        }
    }
    Ok(segment.ident.clone())
}

fn filter_ident_excluding_variable_and_mapping(field: &syn::Field) -> Option<syn::Ident> {
    filter_ident(field, &["Variable", "Mapping"])
}

fn filter_ident(field: &syn::Field, exclusions: &'static [&str]) -> Option<syn::Ident> {
    match &field.ty {
        syn::Type::Path(path) => {
            let path = &path.path;
            match &path.segments.last() {
                Some(segment) => {
                    if exclusions.contains(&segment.ident.to_string().as_str()) {
                        return None;
                    }
                    Some(segment.ident.clone())
                }
                _ => None
            }
        }
        _ => None
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let path = syn::parse_str::<syn::TypePath>("Mapping<String, Mapping<String, Mapping<u8, String>>>").unwrap();
        let ident = extract_mapping_value_ident_from_path(&path);
        assert_eq!(ident.unwrap().to_string(), "String");

        // Mapping expected but found String
        let path = syn::parse_str::<syn::TypePath>("String<i32, u8, u16>").unwrap();
        let ident = extract_mapping_value_ident_from_path(&path);
        assert!(ident.is_err());
        
        // Invalid Mapping - parenthesized arguments instead of angle bracketed
        let path = syn::parse_str::<syn::TypePath>("Mapping<String, Mapping<String, Mapping(u8, String)>>").unwrap();
        let ident = extract_mapping_value_ident_from_path(&path);
        assert!(ident.is_err());
        
        // Invalid Mapping - function type instead of type
        let path = syn::parse_str::<syn::TypePath>("Mapping<String, Mapping<String, Mapping<fn(usize) -> bool>>>").unwrap();
        let ident = extract_mapping_value_ident_from_path(&path);
        assert!(ident.is_err());
    }
}