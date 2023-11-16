use proc_macro2::{Ident, TokenStream};
use syn::ItemImpl;

use crate::syn_utils;

pub mod deployer_item;
pub mod ref_item;

pub struct ModuleIR {
    code: ItemImpl,
}

impl ModuleIR {
    pub fn new(input: &TokenStream) -> Self {
        let code = syn::parse2::<syn::ItemImpl>(input.clone()).unwrap();
        ModuleIR { code }
    }

    pub fn self_code(&self) -> &ItemImpl {
        &self.code
    }

    pub fn module_ident(&self) -> Ident {
        syn_utils::ident_from_impl(&self.code)
    }

    pub fn ref_ident(&self) -> Ident {
        let module_ident = self.module_ident();
        Ident::new(&format!("{}Ref", module_ident), module_ident.span())
    }

    pub fn deployer_ident(&self) -> Ident {
        let module_ident = self.module_ident();
        Ident::new(&format!("{}Deployer", module_ident), module_ident.span())
    }

    pub fn methods(&self) -> Vec<FnIR> {
        let methods = self
            .code
            .items
            .iter()
            .filter_map(|item| match item {
                syn::ImplItem::Fn(method) => Some(FnIR::new(method.clone())),
                _ => None,
            })
            .collect::<Vec<_>>();
        methods
    }
}

pub struct FnIR {
    code: syn::ImplItemFn,
}

impl FnIR {
    pub fn new(code: syn::ImplItemFn) -> Self {
        FnIR { code }
    }

    pub fn name(&self) -> Ident {
        self.code.sig.ident.clone()
    }

    pub fn name_str(&self) -> String {
        self.name().to_string()
    }

    pub fn arg_names(&self) -> Vec<Ident> {
        syn_utils::function_arg_names(&self.code)
            .into_iter()
            .filter(|ident| ident != "env")
            .collect()
    }

    pub fn args_len(&self) -> usize {
        syn_utils::function_args(&self.code).len() - 1
    }

    pub fn return_type(&self) -> syn::Type {
        syn_utils::function_return_type(&self.code)
    }

    pub fn typed_args(&self) -> Vec<syn::PatType> {
        syn_utils::function_args(&self.code)
            .into_iter()
            .filter(|arg| match &*arg.pat {
                syn::Pat::Ident(pat_ident) => pat_ident.ident != "env",
                _ => panic!("Only support function arg as ident"),
            })
            .collect()
    }

    // Checks if `env` is mutable reference.
    pub fn is_mut(&self) -> bool {
        let args = syn_utils::function_args(&self.code);
        let env_arg = args.first().unwrap();
        match &*env_arg.ty {
            syn::Type::Reference(type_ref) => type_ref.mutability.is_some(),
            _ => panic!("env arg must be reference"),
        }
    }
}