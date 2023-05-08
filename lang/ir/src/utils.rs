use syn::{Data, DataStruct, DeriveInput, Fields, FieldsNamed};

pub fn extract_fields(input: DeriveInput) -> Result<FieldsNamed, syn::Error> {
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
