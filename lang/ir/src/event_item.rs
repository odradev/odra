use proc_macro2::Ident;
use syn::{Data, DataStruct, DeriveInput, Field, Fields};

pub struct EventItem {
    struct_ident: Ident,
    fields: Vec<Field>,
}

impl EventItem {
    pub fn parse(input: DeriveInput) -> Result<Self, syn::Error> {
        let struct_ident = input.ident.clone();
        let fields = extract_fields(input)?;

        Ok(Self {
            struct_ident,
            fields,
        })
    }

    pub fn fields(&self) -> &[syn::Field] {
        self.fields.as_ref()
    }

    pub fn struct_ident(&self) -> &Ident {
        &self.struct_ident
    }
}

fn extract_fields(input: DeriveInput) -> Result<Vec<Field>, syn::Error> {
    let fields = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(named_fields),
            ..
        }) => named_fields.named.into_iter().collect::<Vec<_>>(),
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "Expected a struct with named fields.",
            ))
        }
    };
    Ok(fields)
}
