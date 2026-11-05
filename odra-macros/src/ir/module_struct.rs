use std::collections::HashSet;

use proc_macro2::Ident;
use quote::{format_ident, ToTokens};

use super::config::{ConfigItem, ModuleConfiguration};
use crate::utils;

/// The struct `#[odra::module(..)]` is applied to, with the macro's arguments.
pub struct ModuleStructIR {
    code: syn::ItemStruct,
    config: ConfigItem
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
        Ok(Self { code, config })
    }

    pub fn self_code(&self) -> &syn::ItemStruct {
        &self.code
    }

    pub fn module_ident(&self) -> syn::Ident {
        utils::syn::ident_from_struct(&self.code)
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

    pub fn typed_fields(&self) -> syn::Result<Vec<EnumeratedTypedField>> {
        utils::syn::struct_typed_fields(&self.code)?
            .iter()
            .enumerate()
            .map(|(idx, (ident, ty))| {
                Ok(EnumeratedTypedField {
                    idx: idx as u8 + 1,
                    ident: ident.clone(),
                    ty: ty.clone()
                })
            })
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

    pub fn unique_fields_ty(&self) -> syn::Result<Vec<syn::Type>> {
        // A hack to sort types by their string representation. Otherwise, we would get an unstable
        // order of types in the generated code and tests would fail.
        #[derive(Eq, PartialEq)]
        struct OrdType(syn::Type);

        impl PartialOrd for OrdType {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for OrdType {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.0
                    .to_token_stream()
                    .to_string()
                    .cmp(&other.0.to_token_stream().to_string())
            }
        }
        let fields = utils::syn::struct_typed_fields(&self.code)?;
        let set = HashSet::<syn::Type>::from_iter(fields.iter().map(|(_, ty)| ty.clone()));
        let mut fields = set.into_iter().map(OrdType).collect::<Vec<_>>();
        fields.sort();

        Ok(fields.into_iter().map(|i| i.0).collect())
    }
}

/// A field of the module struct with its 1-based position, which is its storage index.
pub struct EnumeratedTypedField {
    pub idx: u8,
    pub ident: syn::Ident,
    pub ty: syn::Type
}
