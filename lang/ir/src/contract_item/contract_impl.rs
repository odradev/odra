use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::FnArg;

pub struct ContractImpl {
    original_impl: syn::ItemImpl,
    entrypoints: Vec<ContractEntrypoint>,
    ident: Ident,
}

impl ContractImpl {
    pub fn original_impl(&self) -> &syn::ItemImpl {
        &self.original_impl
    }

    pub fn entrypoints(&self) -> &[ContractEntrypoint] {
        self.entrypoints.as_ref()
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }
}

impl From<syn::ItemImpl> for ContractImpl {
    fn from(item_impl: syn::ItemImpl) -> Self {
        let path = match &*item_impl.self_ty {
            syn::Type::Path(path) => path,
            _ => todo!(),
        };
        let contract_ident = path.path.segments.last().unwrap().clone().ident;
        let methods = extract_methods(item_impl.clone());
        Self {
            original_impl: item_impl,
            entrypoints: methods
                .into_iter()
                .map(|method| ContractEntrypoint::from(method))
                .collect(),
            ident: contract_ident,
        }
    }
}

fn extract_methods<'a>(item: syn::ItemImpl) -> Vec<syn::ImplItemMethod> {
    item.items
        .into_iter()
        .filter_map(|item| match item {
            syn::ImplItem::Method(method) => Some(method),
            _ => None,
        })
        .collect::<Vec<_>>()
}

pub struct ContractEntrypoint {
    pub ident: Ident,
    pub args: Vec<syn::PatType>,
    pub ret: syn::ReturnType,
}

impl From<syn::ImplItemMethod> for ContractEntrypoint {
    fn from(method: syn::ImplItemMethod) -> Self {
        let ident = method.sig.ident.to_owned();
        let args = method
            .sig
            .inputs
            .iter()
            .filter_map(|arg| match arg {
                FnArg::Receiver(_) => None,
                FnArg::Typed(pat) => Some(pat.clone()),
            })
            .collect::<Vec<_>>();
        let ret = method.sig.output;
        Self { ident, args, ret }
    }
}

impl ToTokens for ContractEntrypoint {
    fn to_tokens(&self, tokens: &mut TokenStream) {
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
            .flatten()
            .collect::<TokenStream>();
        let ret = match &self.ret {
            syn::ReturnType::Default => quote!(odra::types::CLType::Unit),
            syn::ReturnType::Type(_, ty) => quote!(<#ty as odra::types::CLTyped>::cl_type()),
        };
        tokens.extend(quote! {
            odra::contract_def::Entrypoint {
                ident: String::from(#name),
                args: vec![#args],
                ret: #ret
            },
        });
    }
}
