use proc_macro2::Ident;
use syn::{Data, DataStruct, DeriveInput, Field, Fields, FieldsNamed};

/// Odra event definition.
pub struct EventItem {
    struct_ident: Ident,
    named_fields: FieldsNamed
}

impl EventItem {
    pub fn parse(input: DeriveInput) -> Result<Self, syn::Error> {
        let struct_ident = input.ident.clone();
        let named_fields = extract_fields(input)?;

        Ok(Self {
            struct_ident,
            named_fields
        })
    }

    pub fn fields_iter(&self) -> impl Iterator<Item = &Field> {
        self.named_fields.named.iter()
    }

    pub fn struct_ident(&self) -> &Ident {
        &self.struct_ident
    }
}

fn extract_fields(input: DeriveInput) -> Result<FieldsNamed, syn::Error> {
    let fields = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(named_fields),
            ..
        }) => named_fields,
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "Expected a struct with named fields."
            ))
        }
    };
    Ok(fields)
}
