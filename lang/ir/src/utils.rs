use syn::{
    visit::{self, Visit},
    Data, DataStruct, DeriveInput, Fields, FieldsNamed
};

pub fn extract_fields(input: DeriveInput) -> Result<FieldsNamed, syn::Error> {
    let fields = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(named_fields),
            ..
        }) => named_fields,
        Data::Struct(DataStruct {
            fields: Fields::Unit,
            ..
        }) => FieldsNamed {
            brace_token: Default::default(),
            named: Default::default()
        },
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "Expected a struct with named fields."
            ))
        }
    };
    Ok(fields)
}

pub struct FieldsValidator(Result<(), syn::Error>);

impl From<&syn::ItemStruct> for FieldsValidator {
    fn from(value: &syn::ItemStruct) -> Self {
        let mut visitor = FieldsValidator(Ok(()));
        visitor.visit_item_struct(value);
        visitor
    }
}

impl FieldsValidator {
    pub fn result(self) -> Result<(), syn::Error> {
        self.0
    }
}

impl<'ast> Visit<'ast> for FieldsValidator {
    fn visit_field(&mut self, f: &'ast syn::Field) {
        if let Some(f) = &f.ident {
            if f.to_string().contains(odra_utils::KEY_DELIMITER) {
                self.0 = Err(syn::Error::new_spanned(
                    f,
                    "Invalid character '#' in the field name"
                ))
            }
        }
        visit::visit_field(self, f);
    }
}
