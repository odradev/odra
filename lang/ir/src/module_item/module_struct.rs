use super::{ModuleEvent, ModuleEvents};
use crate::attrs::partition_attributes;
use anyhow::{Context, Result};
use proc_macro2::Ident;
use syn::{punctuated::Punctuated, Field, Fields, FieldsNamed, Token};

use super::ModuleConfiguration;

/// Odra module struct.
///
/// Wraps up [syn::ItemStruct].
pub struct ModuleStruct {
    pub is_instantiable: bool,
    pub item: syn::ItemStruct,
    pub events: ModuleEvents,
    pub delegated_fields: Vec<DelegatedField>
}

pub struct DelegatedField {
    pub field: syn::Field,
    pub delegated_fields: Vec<String>
}

impl DelegatedField {
    pub(crate) fn validate(&self, fields: &syn::Fields) -> Result<(), syn::Error> {
        let fields = fields
            .iter()
            .filter_map(|f| f.ident.clone())
            .map(|i| i.to_string())
            .collect::<Vec<_>>();
        let is_valid = self.delegated_fields.iter().find(|f| !fields.contains(f));
        if let Some(invalid_ref) = is_valid {
            let error_msg = format!("Using non-existing field {}", invalid_ref);
            return Err(syn::Error::new_spanned(&self.field, error_msg));
        }

        Ok(())
    }
}

impl TryFrom<syn::Field> for DelegatedField {
    type Error = syn::Error;

    fn try_from(value: syn::Field) -> std::result::Result<Self, Self::Error> {
        let (odra_attrs, other_attrs) = partition_attributes(value.attrs.clone()).unwrap();

        let delegated_fields = odra_attrs
            .iter()
            .flat_map(|attr| attr.using())
            .collect::<Vec<_>>();

        let field_ident = value.ident.clone().unwrap().to_string();

        if delegated_fields.contains(&field_ident) {
            return Err(syn::Error::new_spanned(&value, "Self-using is not allowed"));
        }

        Ok(Self {
            field: Field {
                attrs: other_attrs,
                ..value
            },
            delegated_fields
        })
    }
}

impl ModuleStruct {
    pub fn with_config(mut self, mut config: ModuleConfiguration) -> Result<Self, syn::Error> {
        let submodules = self
            .item
            .fields
            .iter()
            .filter(|field| field.ident.is_some())
            .filter_map(filter_primitives)
            .map(|ident| ModuleEvent { name: ident })
            .collect::<Vec<_>>();

        let mut mappings = self
            .item
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

        config.events.submodules_events.extend(submodules);
        config.events.mappings_events.extend(mappings);

        self.events = config.events;

        Ok(self)
    }
}

impl TryFrom<syn::ItemStruct> for ModuleStruct {
    type Error = syn::Error;

    fn try_from(value: syn::ItemStruct) -> std::result::Result<Self, Self::Error> {
        let (_, other_attrs) = partition_attributes(value.attrs).unwrap();

        let named = value
            .fields
            .clone()
            .into_iter()
            .map(|field| {
                let (_, other_attrs) = partition_attributes(field.attrs).unwrap();
                Field {
                    attrs: other_attrs,
                    ..field
                }
            })
            .collect::<Punctuated<Field, Token![,]>>();

        let fields: Fields = Fields::Named(FieldsNamed {
            brace_token: Default::default(),
            named
        });

        let delegated_fields = value
            .fields
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<DelegatedField>, syn::Error>>()?;

        delegated_fields
            .iter()
            .try_for_each(|f| f.validate(&fields))?;

        Ok(Self {
            is_instantiable: true,
            item: syn::ItemStruct {
                attrs: other_attrs,
                fields,
                ..value
            },
            events: Default::default(),
            delegated_fields
        })
    }
}

fn extract_mapping_value_ident_from_path(path: &syn::TypePath) -> Result<Ident> {
    // Eg. odra::type::Mapping<String, Mapping<String, Mapping<u8, String>>>
    let mut segment = path
        .path
        .segments
        .last()
        .cloned()
        .context("At least one segment expected")?;
    if segment.ident != "Mapping" {
        return Err(anyhow::anyhow!(
            "Mapping expected but found {}",
            segment.ident
        ));
    }
    loop {
        let args = &segment.arguments;
        if args.is_empty() {
            break;
        }
        if let syn::PathArguments::AngleBracketed(args) = args {
            match args
                .args
                .last()
                .context("syn::GenericArgument expected but not found")?
            {
                syn::GenericArgument::Type(syn::Type::Path(path)) => {
                    let path = &path.path;
                    segment = path
                        .segments
                        .last()
                        .cloned()
                        .context("At least one segment expected")?;
                }
                other => {
                    return Err(anyhow::anyhow!(
                        "syn::TypePath expected but found {:?}",
                        other
                    ))
                }
            }
        } else {
            return Err(anyhow::anyhow!(
                "syn::AngleBracketedGenericArguments expected but found {:?}",
                args
            ));
        }
    }
    Ok(segment.ident)
}

fn filter_primitives(field: &syn::Field) -> Option<syn::Ident> {
    filter_ident(field, &["Variable", "Mapping", "List", "Sequence"])
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
        let path = syn::parse_str::<syn::TypePath>(
            "Mapping<String, Mapping<String, Mapping<u8, String>>>"
        )
        .unwrap();
        let ident = extract_mapping_value_ident_from_path(&path);
        assert_eq!(ident.unwrap().to_string(), "String");

        // Mapping expected but found String
        let path = syn::parse_str::<syn::TypePath>("String<i32, u8, u16>").unwrap();
        let ident = extract_mapping_value_ident_from_path(&path);
        assert!(ident.is_err());

        // Invalid Mapping - parenthesized arguments instead of angle bracketed
        let path = syn::parse_str::<syn::TypePath>(
            "Mapping<String, Mapping<String, Mapping(u8, String)>>"
        )
        .unwrap();
        let ident = extract_mapping_value_ident_from_path(&path);
        assert!(ident.is_err());

        // Invalid Mapping - function type instead of type
        let path = syn::parse_str::<syn::TypePath>(
            "Mapping<String, Mapping<String, Mapping<fn(usize) -> bool>>>"
        )
        .unwrap();
        let ident = extract_mapping_value_ident_from_path(&path);
        assert!(ident.is_err());
    }
}
