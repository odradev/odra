use quote::{quote, ToTokens};

use crate::attrs::{partition_attributes, OdraAttribute};

/// Odra method definition.
///
/// # Examples
/// ```
/// # <odra_ir::module::Method as From<syn::ImplItemMethod>>::from(syn::parse_quote! {
/// pub fn set_value(&self, value: u32) {
///    // logic goes here
/// }
/// # });
/// ```
pub struct Method {
    pub attrs: Vec<OdraAttribute>,
    pub impl_item: syn::ImplItemMethod,
    pub ident: syn::Ident,
    pub args: syn::punctuated::Punctuated<syn::PatType, syn::token::Comma>,
    pub ret: syn::ReturnType,
    pub full_sig: syn::Signature,
    pub visibility: syn::Visibility,
}

impl Method {
    pub fn is_public(&self) -> bool {
        matches!(self.visibility, syn::Visibility::Public(_))
    }
}

impl ToTokens for Method {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.ident.to_string();
        let args = &self
            .args
            .iter()
            .map(|arg| {
                let name = &*arg.pat;
                let ty = &*arg.ty;
                let ty = quote!(<#ty as odra::types::CLTyped>::cl_type());
                quote! {
                    odra::contract_def::Argument {
                        ident: String::from(stringify!(#name)),
                        ty: #ty,
                    },
                }
            })
            .collect::<proc_macro2::TokenStream>();

        let ret = match &self.ret {
            syn::ReturnType::Default => quote!(odra::types::CLType::Unit),
            syn::ReturnType::Type(_, ty) => quote!(<#ty as odra::types::CLTyped>::cl_type()),
        };

        let ep = quote! {
            odra::contract_def::Entrypoint {
                ident: String::from(#name),
                args: vec![#args],
                ret: #ret,
                ty: odra::contract_def::EntrypointType::Public,
            },
        };

        tokens.extend(ep)
    }
}

impl From<syn::ImplItemMethod> for Method {
    fn from(method: syn::ImplItemMethod) -> Self {
        let (odra_attrs, attrs) = partition_attributes(method.clone().attrs).unwrap();
        let ident = method.sig.ident.to_owned();
        let args = method
            .sig
            .inputs
            .iter()
            .filter_map(|arg| match arg {
                syn::FnArg::Receiver(_) => None,
                syn::FnArg::Typed(pat) => Some(pat.clone()),
            })
            .collect::<syn::punctuated::Punctuated<_, _>>();
        let ret = method.clone().sig.output;
        let full_sig = method.clone().sig;
        let visibility = method.vis.clone();
        Self {
            attrs: odra_attrs,
            impl_item: syn::ImplItemMethod { attrs, ..method },
            ident,
            args,
            ret,
            full_sig,
            visibility,
        }
    }
}
