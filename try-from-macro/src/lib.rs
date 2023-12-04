use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_error::{abort, proc_macro_error};

#[proc_macro_derive(TryFromRef, attributes(source, default, expr))]
#[proc_macro_error]
pub fn derive_try_from(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    if !input.generics.params.is_empty() {
        abort! {
            input.generics,
            "Generics are not supported!"
        }
    }
    let from_ident = match parse_attrs(&input.attrs) {
        Ok(Attr::Source(ident)) => ident,
        _ => abort! {
            input.ident,
            "Missing source attribute";
            help = "Add #[source(YourStruct)]";
        }
    };
    let to_ident = &input.ident;

    match &input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(named),
            ..
        }) => derive_convert_struct(&from_ident, to_ident, named),
        _ => abort! {
            input,
            "Unions and Enums are not supported!"
        }
    }
    .into()
}

fn derive_convert_struct(
    from_ident: &proc_macro2::Ident,
    to_ident: &proc_macro2::Ident,
    fields: &syn::FieldsNamed
) -> proc_macro2::TokenStream {
    let fields = fields
        .named
        .iter()
        .map(to_field_definition)
        .collect::<Vec<_>>();
    quote::quote!(
        impl TryFrom<&'_ #from_ident> for #to_ident {
            type Error = syn::Error;

            fn try_from(item: &'_ #from_ident) -> Result<Self, Self::Error> {
                Ok(Self {
                    #( #fields ),*
                })
            }
        }
    )
}

fn to_field_definition(field: &syn::Field) -> proc_macro2::TokenStream {
    let ident = &field.ident;
    match parse_attrs(&field.attrs) {
        Ok(Attr::Default) => quote::quote!(#ident: Default::default()),
        Ok(Attr::Expr(expr)) => quote::quote!(#ident: #expr),
        _ => quote::quote!(#ident: item.try_into()?)
    }
}

enum Attr {
    Source(syn::Ident),
    Default,
    Expr(syn::Expr)
}

fn parse_attrs(attrs: &[syn::Attribute]) -> syn::Result<Attr> {
    if let Some(attr) = find_attr(attrs, "source") {
        return Ok(Attr::Source(attr.parse_args()?));
    }
    if find_attr(attrs, "default").is_some() {
        return Ok(Attr::Default);
    }
    if let Some(attr) = find_attr(attrs, "expr") {
        return Ok(Attr::Expr(attr.parse_args()?));
    }
    Err(syn::Error::new(Span::call_site(), "No attr found"))
}

fn find_attr<'a>(attrs: &'a [syn::Attribute], ident: &str) -> Option<&'a syn::Attribute> {
    attrs.iter().find(|attr| attr.path().is_ident(ident))
}
