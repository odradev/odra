use proc_macro2::Ident;
use syn::{Data, DataStruct, DeriveInput, Fields, Type};

/// Odra event definition.
pub struct EventItem {
    struct_ident: Ident,
    fields: Vec<Field>
}

impl EventItem {
    pub fn parse(input: DeriveInput) -> Result<Self, syn::Error> {
        let struct_ident = input.ident.clone();
        let fields = extract_fields(input)?;

        Ok(Self {
            struct_ident,
            fields
        })
    }

    pub fn fields(&self) -> &Vec<Field> {
        &self.fields
    }

    pub fn struct_ident(&self) -> &Ident {
        &self.struct_ident
    }
}

pub struct Field {
    pub ident: Ident,
    pub is_optional: bool
}

fn extract_fields(input: DeriveInput) -> Result<Vec<Field>, syn::Error> {
    let fields = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(named_fields),
            ..
        }) => named_fields
            .named
            .into_iter()
            .map(|f| {
                let is_optional = match f.ty {
                    Type::Path(path) => {
                        if let Some(seg) = path.path.segments.first() {
                            seg.ident.to_string().as_str() == "Option"
                        } else {
                            false
                        }
                    }
                    _ => false
                };
                Field {
                    ident: f.ident.unwrap(),
                    is_optional
                }
            })
            .collect::<Vec<_>>(),
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "Expected a struct with named fields."
            ))
        }
    };
    Ok(fields)
}
