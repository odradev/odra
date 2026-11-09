use proc_macro2::TokenStream;
use syn::parse_quote;

pub fn function_typed_args(sig: &syn::Signature) -> Vec<syn::PatType> {
    sig.inputs
        .iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Typed(pat_type) => Some(pat_type.clone()),
            _ => None
        })
        .collect()
}

pub fn receiver_arg(sig: &syn::Signature) -> Option<syn::Receiver> {
    sig.inputs.iter().find_map(|arg| match arg {
        syn::FnArg::Receiver(receiver) => Some(receiver.clone()),
        _ => None
    })
}

pub fn visibility_pub() -> syn::Visibility {
    parse_quote!(pub)
}

pub fn visibility_default() -> syn::Visibility {
    parse_quote!()
}

pub fn docs_attrs(attrs: &[syn::Attribute]) -> Vec<syn::Attribute> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .cloned()
        .collect()
}

/// Joins every doc-comment line into a single description, trimming each line.
///
/// `///` doc comments are often wrapped across several lines; using only the first one truncates
/// the description, so all lines are folded into one space-separated sentence. Blank lines
/// (paragraph breaks) are dropped, otherwise they would leave a double space in the description.
pub fn docs_description(docs: &[String]) -> String {
    docs.iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn string_docs(attrs: &[syn::Attribute]) -> Vec<String> {
    let attrs = docs_attrs(attrs);

    let mut docs = Vec::new();
    for attr in attrs {
        if let syn::Meta::NameValue(nv) = &attr.meta {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(str),
                ..
            }) = &nv.value
            {
                docs.push(str.value());
            }
        }
    }
    docs
}

// A path like <Option::<U256> as odra::casper_types::CLTyped>
pub fn as_casted_ty_stream(ty: &syn::Type, as_ty: syn::Type) -> TokenStream {
    let ty = match ty {
        syn::Type::Path(type_path) => {
            let mut segments: syn::punctuated::Punctuated<syn::PathSegment, syn::Token![::]> =
                type_path.path.segments.clone();
            // the syntax <Option<U256> as odra::casper_types::CLTyped>::cl_type() is invalid
            // it should be <Option::<U256> as odra::casper_types::CLTyped>::cl_type()
            if let Some(ps) = segments.first_mut() {
                if let syn::PathArguments::AngleBracketed(ab) = &ps.arguments {
                    let generic_arg: syn::AngleBracketedGenericArguments = parse_quote!(::#ab);
                    ps.arguments = syn::PathArguments::AngleBracketed(generic_arg);
                }
            }
            syn::Type::Path(syn::TypePath {
                path: syn::Path {
                    leading_colon: None,
                    segments
                },
                ..type_path.clone()
            })
        }
        _ => ty.clone()
    };

    parse_quote!(<#ty as #as_ty>)
}

pub fn is_ref(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Reference(_))
}

#[cfg(test)]
mod tests {
    use super::docs_description;

    #[test]
    fn docs_description_joins_lines_with_a_single_space() {
        let docs = vec![" First line. ".to_string(), "Second line.".to_string()];
        assert_eq!(docs_description(&docs), "First line. Second line.");
    }

    #[test]
    fn docs_description_drops_blank_lines() {
        let docs = vec![
            " First paragraph.".to_string(),
            "".to_string(),
            " Second paragraph.".to_string(),
            "   ".to_string(),
            "".to_string(),
            " Third paragraph.".to_string(),
        ];
        assert_eq!(
            docs_description(&docs),
            "First paragraph. Second paragraph. Third paragraph."
        );
    }
}
