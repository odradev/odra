use std::collections::BTreeMap;

use proc_macro2::Ident;
use quote::{format_ident, ToTokens};

use super::config::{ConfigItem, ModuleConfiguration};
use super::errors;
use crate::utils;

/// A module has at most this many fields: the storage index of a field is a `u8`.
const MAX_FIELDS: usize = 255;

/// The struct `#[odra::module(..)]` is applied to, with the macro's arguments. Its fields are
/// named, or there are none, and there are at most [MAX_FIELDS] of them.
pub struct ModuleStructIR {
    code: syn::ItemStruct,
    config: ConfigItem,
    fields: Vec<EnumeratedTypedField>
}

/// The IR from the macro's token streams, for the test mocks.
#[cfg(test)]
impl TryFrom<(&proc_macro2::TokenStream, &proc_macro2::TokenStream)> for ModuleStructIR {
    type Error = syn::Error;

    fn try_from(
        (attr, item): (&proc_macro2::TokenStream, &proc_macro2::TokenStream)
    ) -> Result<Self, Self::Error> {
        Self::try_new(attr, syn::parse2(item.clone())?)
    }
}

impl ModuleStructIR {
    /// The IR of the struct, with `attr` the arguments of the macro.
    pub fn try_new(attr: &proc_macro2::TokenStream, code: syn::ItemStruct) -> syn::Result<Self> {
        let config = syn::parse2::<ConfigItem>(attr.clone())?;
        if code.fields.len() > MAX_FIELDS {
            return Err(errors::module_too_many_fields(&code.fields, MAX_FIELDS));
        }
        let fields = match &code.fields {
            fields if fields.is_empty() => Vec::new(),
            syn::Fields::Named(named) => named
                .named
                .iter()
                .enumerate()
                .map(|(idx, f)| EnumeratedTypedField {
                    idx: idx as u8 + 1,
                    // The field is named: the fields are.
                    ident: f.ident.clone().unwrap_or_else(|| unreachable!()),
                    ty: f.ty.clone()
                })
                .collect(),
            fields => return Err(errors::module_fields_unnamed(fields))
        };
        Ok(Self {
            code,
            config,
            fields
        })
    }

    pub fn self_code(&self) -> &syn::ItemStruct {
        &self.code
    }

    pub fn module_ident(&self) -> syn::Ident {
        self.code.ident.clone()
    }

    pub fn factory_module_ident(&self) -> syn::Ident {
        let module_ident = self.module_ident();
        Ident::new(&format!("{}Factory", module_ident), module_ident.span())
    }

    pub fn module_mod_ident(&self) -> syn::Ident {
        format_ident!(
            "__{}_module",
            utils::string::camel_to_snake(self.module_ident())
        )
    }

    pub fn contract_schema_mod_ident(&self) -> Ident {
        let ident = self.module_ident();
        Ident::new(
            utils::string::camel_to_snake(format!("__{}_contract_schema", ident)).as_str(),
            ident.span()
        )
    }

    /// The fields in declaration order, each with its storage index.
    pub fn typed_fields(&self) -> &[EnumeratedTypedField] {
        &self.fields
    }

    /// The types of the fields, each once, ordered by their spelling so the generated code is
    /// stable.
    pub fn unique_fields_ty(&self) -> Vec<syn::Type> {
        self.fields
            .iter()
            .map(|f| (f.ty.to_token_stream().to_string(), f.ty.clone()))
            .collect::<BTreeMap<_, _>>()
            .into_values()
            .collect()
    }

    /// The `#[odra::module(..)]` arguments, when any were given.
    fn module_cfg(&self) -> Option<&ModuleConfiguration> {
        match &self.config {
            ConfigItem::Module(cfg) => Some(cfg),
            ConfigItem::Empty => None
        }
    }

    /// The types of `events = [..]`.
    pub fn events(&self) -> Vec<syn::Type> {
        self.module_cfg()
            .map(|cfg| cfg.events.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// The type of `errors = ..`.
    pub fn errors(&self) -> Option<syn::Type> {
        self.module_cfg().and_then(|cfg| (*cfg.errors).clone())
    }

    /// The expression of `layout = ..`.
    pub fn storage_layout(&self) -> Option<syn::Expr> {
        self.module_cfg().and_then(|cfg| (*cfg.layout).clone())
    }

    /// `factory = on`.
    pub fn is_factory(&self) -> bool {
        self.module_cfg().is_some_and(|cfg| *cfg.factory)
    }

    /// The `name = ".."`, empty when not given.
    pub fn contract_name(&self) -> String {
        self.module_cfg()
            .map(|cfg| (*cfg.name).clone())
            .unwrap_or_default()
    }

    /// The `version = ".."`, empty when not given.
    pub fn contract_version(&self) -> String {
        self.module_cfg()
            .map(|cfg| (*cfg.version).clone())
            .unwrap_or_default()
    }
}

/// A field of the module struct with its 1-based position, which is its storage index.
#[derive(Clone)]
pub struct EnumeratedTypedField {
    pub idx: u8,
    pub ident: syn::Ident,
    pub ty: syn::Type
}
