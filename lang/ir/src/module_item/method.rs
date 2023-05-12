use quote::{quote, ToTokens};

use crate::attrs::{partition_attributes, OdraAttribute};

use super::utils;

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
    pub is_mut: bool,
    pub args: syn::punctuated::Punctuated<syn::PatType, syn::token::Comma>,
    pub ret: syn::ReturnType,
    pub full_sig: syn::Signature,
    pub visibility: syn::Visibility
}

impl Method {
    pub fn is_public(&self) -> bool {
        matches!(self.visibility, syn::Visibility::Public(_))
    }

    pub fn is_payable(&self) -> bool {
        self.attrs.iter().any(|attr| attr.is_payable())
    }
}

impl ToTokens for Method {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let is_non_reentrant = self.attrs.iter().any(OdraAttribute::is_non_reentrant);
        let name = &self.ident.to_string();
        let args = &self
            .args
            .iter()
            .map(|arg| {
                let name = &*arg.pat;

                let ty = match &*arg.ty {
                    syn::Type::Reference(r) => &r.elem,
                    other => other
                };
                let is_ref = matches!(&*arg.ty, syn::Type::Reference(_));
                let ty = quote!(<#ty as odra::types::Typed>::ty());
                quote! {
                    odra::types::contract_def::Argument {
                        ident: String::from(stringify!(#name)),
                        ty: #ty,
                        is_ref: #is_ref,
                    },
                }
            })
            .collect::<proc_macro2::TokenStream>();

        let ret = match &self.ret {
            syn::ReturnType::Default => quote!(odra::types::Type::Unit),
            syn::ReturnType::Type(_, ty) => quote!(<#ty as odra::types::Typed>::ty())
        };

        let ty = match self.attrs.iter().any(|attr| attr.is_payable()) {
            true => {
                quote!(odra::types::contract_def::EntrypointType::PublicPayable { non_reentrant: #is_non_reentrant })
            }
            false => {
                quote!(odra::types::contract_def::EntrypointType::Public { non_reentrant: #is_non_reentrant })
            }
        };

        let is_mut = self.is_mut;

        let ep = quote! {
            odra::types::contract_def::Entrypoint {
                ident: String::from(#name),
                args: vec![#args],
                is_mut: #is_mut,
                ret: #ret,
                ty: #ty,
            },
        };

        tokens.extend(ep)
    }
}

impl TryFrom<syn::ImplItemMethod> for Method {
    type Error = syn::Error;

    fn try_from(method: syn::ImplItemMethod) -> Result<Self, Self::Error> {
        // validate_args(&method)?;

        let (odra_attrs, attrs) = partition_attributes(method.clone().attrs)?;
        let ident = method.sig.ident.to_owned();
        let args = method
            .sig
            .inputs
            .iter()
            .filter_map(|arg| match arg {
                syn::FnArg::Receiver(_) => None,
                syn::FnArg::Typed(pat) => Some(pat.clone())
            })
            .collect::<syn::punctuated::Punctuated<_, _>>();
        let ret = method.clone().sig.output;
        let full_sig = method.clone().sig;
        let visibility = method.vis.clone();
        let is_mut = utils::is_mut(&full_sig);

        Ok(Self {
            attrs: odra_attrs,
            impl_item: syn::ImplItemMethod { attrs, ..method },
            ident,
            is_mut,
            args,
            ret,
            full_sig,
            visibility
        })
    }
}
