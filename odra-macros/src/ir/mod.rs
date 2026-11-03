use std::collections::HashSet;

use crate::ir::delegate::{Delegate, DelegatedFn};
use crate::utils;
use config::ConfigItem;
use proc_macro2::Ident;
use quote::{format_ident, ToTokens};
use syn::{parse_quote, ImplItem};

use self::attr::OdraAttribute;

mod attr;
mod config;
pub mod delegate;
pub mod errors;

use errors::OffchainMisuse;

const CONSTRUCTOR_NAME: &str = "init";
const UPGRADER_NAME: &str = "upgrade";
const FACTORY_NAME: &str = "new_contract";
const FACTORY_UPGRADE_NAME: &str = "upgrade_child_contract";
const BATCH_FACTORY_UPGRADE_NAME: &str = "batch_upgrade_child_contract";

#[derive(PartialEq, Eq)]
pub enum FnType {
    Constructor,
    Upgrader,
    Factory,
    FactoryUpgrader,
    FactoryBatchUpgrader,
    Regular
}

macro_rules! try_parse {
    ($from:path => $to:ident) => {
        pub struct $to {
            code: $from,
            #[allow(dead_code)]
            config: ConfigItem
        }

        impl TryFrom<(&proc_macro2::TokenStream, &proc_macro2::TokenStream)> for $to {
            type Error = syn::Error;

            fn try_from(
                stream: (&proc_macro2::TokenStream, &proc_macro2::TokenStream)
            ) -> Result<Self, Self::Error> {
                Ok(Self {
                    code: syn::parse2::<$from>(stream.1.clone())?,
                    config: syn::parse2::<ConfigItem>(stream.0.clone())?
                })
            }
        }
    };
}

try_parse!(syn::ItemStruct => ModuleStructIR);

impl ModuleStructIR {
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

    pub fn events(&self) -> Vec<syn::Type> {
        if let ConfigItem::Module(cfg) = &self.config {
            cfg.events.iter().cloned().collect()
        } else {
            vec![]
        }
    }

    pub fn errors(&self) -> Option<syn::Type> {
        if let ConfigItem::Module(cfg) = &self.config {
            (*cfg.errors).clone()
        } else {
            None
        }
    }

    pub fn storage_layout(&self) -> Option<syn::Expr> {
        if let ConfigItem::Module(cfg) = &self.config {
            (*cfg.layout).clone()
        } else {
            None
        }
    }

    pub fn is_factory(&self) -> bool {
        if let ConfigItem::Module(cfg) = &self.config {
            *cfg.factory
        } else {
            false
        }
    }

    pub fn contract_name(&self) -> String {
        if let ConfigItem::Module(cfg) = &self.config {
            (*cfg.name).clone()
        } else {
            String::from("")
        }
    }

    pub fn contract_version(&self) -> String {
        if let ConfigItem::Module(cfg) = &self.config {
            (*cfg.version).clone()
        } else {
            String::from("")
        }
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

pub struct EnumeratedTypedField {
    pub idx: u8,
    pub ident: syn::Ident,
    pub ty: syn::Type
}

pub enum ModuleImplIR {
    Impl(ModuleIR),
    Trait(ModuleTraitIR)
}

impl TryFrom<(&proc_macro2::TokenStream, &proc_macro2::TokenStream)> for ModuleImplIR {
    type Error = syn::Error;

    fn try_from(
        stream: (&proc_macro2::TokenStream, &proc_macro2::TokenStream)
    ) -> Result<Self, Self::Error> {
        let config = syn::parse2::<ConfigItem>(stream.0.clone())?;
        if let Ok(code) = syn::parse2::<syn::ItemImpl>(stream.1.clone()) {
            if let ConfigItem::Module(cfg) = &config {
                errors::check_all(
                    cfg.struct_only_args()
                        .map(|(name, span)| Err(errors::module_arg_on_impl(name, *span)))
                )?;
            }
            let ir = Self::Impl(ModuleIR { code, config });
            errors::check_all([ir.functions().map(|_| ()), ir.validate_offchain_functions()])?;
            return Ok(ir);
        }

        if let Ok(code) = syn::parse2::<syn::ItemTrait>(stream.1.clone()) {
            if let ConfigItem::Module(cfg) = &config {
                errors::check_all(
                    cfg.args()
                        .map(|(name, span)| Err(errors::module_arg_on_trait(name, *span)))
                )?;
            }
            for c in code.items.iter() {
                if let syn::TraitItem::Verbatim(func) = c {
                    syn::parse2::<syn::TraitItemFn>(func.clone())?;
                }
            }
            let ir = Self::Trait(ModuleTraitIR { code, config });
            ir.functions()?;
            return Ok(ir);
        }

        Err(errors::impl_or_trait_expected(stream.1))
    }
}

impl ModuleImplIR {
    pub fn self_code(&self) -> syn::Result<proc_macro2::TokenStream> {
        match self {
            ModuleImplIR::Impl(ir) => ir.self_code().map(ToTokens::into_token_stream),
            ModuleImplIR::Trait(ir) => ir.self_code().map(ToTokens::into_token_stream)
        }
    }

    pub fn module_ident(&self) -> syn::Result<Ident> {
        match self {
            ModuleImplIR::Impl(ir) => utils::syn::ident_from_impl(&ir.code),
            ModuleImplIR::Trait(ir) => Ok(ir.module_ident())
        }
    }

    pub fn is_trait_impl(&self) -> bool {
        match self {
            ModuleImplIR::Impl(ir) => ir.code.trait_.is_some(),
            ModuleImplIR::Trait(_) => false
        }
    }

    pub fn is_factory(&self) -> bool {
        match self {
            ModuleImplIR::Impl(ir) => {
                if let ConfigItem::Module(cfg) = &ir.config {
                    *cfg.factory
                } else {
                    false
                }
            }
            ModuleImplIR::Trait(_) => false
        }
    }

    pub fn impl_trait_ident(&self) -> Option<Ident> {
        match self {
            ModuleImplIR::Impl(ir) => ir
                .code
                .trait_
                .clone()
                .map(|(_, path, _)| path.get_ident().unwrap().clone()),
            ModuleImplIR::Trait(_) => None
        }
    }

    pub fn module_str(&self) -> syn::Result<String> {
        self.module_ident().map(|i| i.to_string())
    }

    pub fn snake_cased_module_ident(&self) -> syn::Result<Ident> {
        let ident = self.module_ident()?;
        Ok(Ident::new(
            utils::string::camel_to_snake(&ident).as_str(),
            ident.span()
        ))
    }

    pub fn host_ref_ident(&self) -> syn::Result<Ident> {
        let module_ident = self.module_ident()?;
        Ok(Ident::new(
            &format!("{}HostRef", module_ident),
            module_ident.span()
        ))
    }

    pub fn contract_ref_ident(&self) -> syn::Result<Ident> {
        let module_ident = self.module_ident()?;
        Ok(Ident::new(
            &format!("{}ContractRef", module_ident),
            module_ident.span()
        ))
    }

    pub fn factory_module_ident(&self) -> syn::Result<Ident> {
        let module_ident = self.module_ident()?;
        Ok(Ident::new(
            &format!("{}Factory", module_ident),
            module_ident.span()
        ))
    }

    /// The module a factory impl stands for. `#[odra::factory]` sees only the `<Module>Factory`
    /// ident that [factory_module_ident](Self::factory_module_ident) made, so the suffix is
    /// stripped back; for any other module it is the module ident itself.
    pub fn child_module_ident(&self) -> syn::Result<Ident> {
        let module_ident = self.module_ident()?;
        let module_str = module_ident.to_string();
        let child = module_str.strip_suffix("Factory").unwrap_or(&module_str);
        Ok(Ident::new(child, module_ident.span()))
    }

    pub fn init_args_ident(&self) -> syn::Result<syn::Ident> {
        let module_ident = self.module_ident()?;
        Ok(Ident::new(
            &format!("{}InitArgs", module_ident),
            module_ident.span()
        ))
    }

    pub fn upgrade_args_ident(&self) -> syn::Result<syn::Ident> {
        let module_ident = self.module_ident()?;
        Ok(Ident::new(
            &format!("{}UpgradeArgs", module_ident),
            module_ident.span()
        ))
    }

    pub fn schema_mod_ident(&self) -> syn::Result<Ident> {
        let module_ident = self.snake_cased_module_ident()?;
        Ok(Ident::new(
            &format!("__{}_schema", module_ident),
            module_ident.span()
        ))
    }

    pub fn test_parts_mod_ident(&self) -> syn::Result<syn::Ident> {
        let module_ident = self.snake_cased_module_ident()?;
        Ok(Ident::new(
            &format!("__{}_test_parts", module_ident),
            module_ident.span()
        ))
    }

    pub fn wasm_parts_mod_ident(&self) -> syn::Result<syn::Ident> {
        let module_ident = self.snake_cased_module_ident()?;
        Ok(Ident::new(
            &format!("__{}_wasm_parts", module_ident),
            module_ident.span()
        ))
    }

    pub fn exec_parts_mod_ident(&self) -> syn::Result<syn::Ident> {
        let module_ident = self.snake_cased_module_ident()?;
        Ok(Ident::new(
            &format!("__{}_exec_parts", module_ident),
            module_ident.span()
        ))
    }

    pub fn host_functions(&self) -> syn::Result<Vec<FnIR>> {
        Ok(self.functions()?.into_iter().collect())
    }

    /// The functions that become entry points of the deployed contract: everything but the
    /// `#[odra(offchain)]` ones.
    pub fn onchain_functions(&self) -> syn::Result<Vec<FnIR>> {
        Ok(self
            .functions()?
            .into_iter()
            .filter(|f| !f.is_offchain())
            .collect())
    }

    /// The `#[odra(offchain)]` functions: run on the host against the contract's state, never
    /// deployed.
    pub fn offchain_functions(&self) -> syn::Result<Vec<FnIR>> {
        Ok(self
            .functions()?
            .into_iter()
            .filter(FnIR::is_offchain)
            .collect())
    }

    /// `#[odra(offchain)]` is only meaningful on a read-only function of a plain module impl.
    ///
    /// Checked on the functions before their own validation, so a reserved name elsewhere in the
    /// impl does not hide an offchain mistake: all of them are reported in one build.
    fn validate_offchain_functions(&self) -> syn::Result<()> {
        let functions = match self {
            ModuleImplIR::Impl(ir) => ir.unchecked_functions(),
            ModuleImplIR::Trait(_) => return Ok(())
        };
        let offchain_functions = functions.iter().filter(|f| f.is_offchain());
        errors::check_all(offchain_functions.map(|f| {
            let sig = f.sig();
            let receiver = utils::syn::receiver_arg(sig);
            // The `#[odra(..)]` attribute that says `offchain`; it parsed, as `f` is offchain.
            let offchain_attr = f.attributes().iter().find(|attr| {
                OdraAttribute::try_from((*attr).clone()).is_ok_and(|a| a.is_offchain())
            });
            let (why, spanned) = if self.is_trait_impl() {
                (OffchainMisuse::InTraitImpl, offchain_attr.to_token_stream())
            } else if self.is_factory() {
                (OffchainMisuse::InFactory, offchain_attr.to_token_stream())
            } else if f.is_mut() {
                (OffchainMisuse::MutReceiver, receiver.to_token_stream())
            } else if f.is_payable() || f.is_non_reentrant() {
                (
                    OffchainMisuse::PayableOrNonReentrant,
                    offchain_attr.to_token_stream()
                )
            } else if receiver.is_none() {
                (OffchainMisuse::NoReceiver, sig.ident.to_token_stream())
            } else {
                return Ok(());
            };
            Err(errors::offchain_misuse(spanned, &f.name_str(), why))
        }))
    }

    pub fn host_mut_ret_functions(&self) -> syn::Result<Vec<FnIR>> {
        Ok(self
            .host_functions()?
            .into_iter()
            .filter(|f| f.is_mut() && f.return_type() != syn::ReturnType::Default)
            .collect())
    }

    pub fn constructor(&self) -> Option<FnIR> {
        self.functions()
            .unwrap_or_default()
            .into_iter()
            .find(|f| f.name_str() == CONSTRUCTOR_NAME)
    }

    pub fn factory_fn(&self) -> FnIR {
        let args = self.constructor_args();
        let ty_address = utils::ty::address();
        let ty_uref = utils::ty::uref();
        let ty_string = utils::ty::string();
        let factory_fn = parse_quote! {
            pub fn new_contract(&mut self, contract_name: #ty_string, #args) -> (#ty_address, #ty_uref) {
            }
        };
        FnIR::Impl(FnImplIR::new(factory_fn))
    }

    pub fn factory_upgrade_fn(&self) -> FnIR {
        let args = self.upgrader_args();
        let ty_string = utils::ty::string();
        FnIR::Impl(FnImplIR::new(parse_quote! {
            pub fn upgrade_child_contract(&mut self, contract_name: #ty_string, #args) {
            }
        }))
    }

    pub fn factory_batch_upgrade_fn(&self) -> FnIR {
        let rt_args = utils::ty::runtime_args();
        let btree_map = utils::ty::typed_btree_map(&utils::ty::string(), &parse_quote!(T));
        FnIR::Impl(FnImplIR::new(parse_quote! {
            pub fn batch_upgrade_child_contract<T: Into<#rt_args>>(&mut self, args: #btree_map) {
            }
        }))
    }

    pub fn upgrader(&self) -> Option<FnIR> {
        self.functions()
            .unwrap_or_default()
            .into_iter()
            .find(|f| f.name_str() == UPGRADER_NAME)
    }

    pub fn functions(&self) -> syn::Result<Vec<FnIR>> {
        match self {
            ModuleImplIR::Impl(ir) => ir.functions(),
            ModuleImplIR::Trait(ir) => ir.functions()
        }
    }

    fn constructor_args(&self) -> syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma> {
        self.constructor()
            .map(|fn_ir| fn_ir.named_args())
            .unwrap_or_default()
            .iter()
            .map(FnArgIR::raw)
            .collect::<syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>>()
    }

    fn upgrader_args(&self) -> syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma> {
        self.upgrader()
            .map(|fn_ir| fn_ir.named_args())
            .unwrap_or_default()
            .iter()
            .map(FnArgIR::raw)
            .collect::<syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>>()
    }
}

try_parse!(syn::ItemImpl => ModuleIR);

impl ModuleIR {
    fn self_code(&self) -> syn::Result<syn::ItemImpl> {
        let mut code = self.code.clone();
        // include delegated functions
        code.items.extend(
            self.delegated_functions()?
                .into_iter()
                .map(|delegated| syn::ImplItem::Fn(delegated.function))
        );
        // remove odra attributes
        code.items.iter_mut().for_each(|item| {
            if let syn::ImplItem::Fn(func) = item {
                func.attrs = attr::other_attributes(func.attrs.clone());
            }
        });
        // remove inner odra macros
        code.items
            .retain(|item| !matches!(item, syn::ImplItem::Macro(_)));
        Ok(code)
    }

    /// The functions `functions` validates, as they are; for checks that must run even when one
    /// of them is invalid.
    fn unchecked_functions(&self) -> Vec<FnIR> {
        self.code
            .items
            .iter()
            .filter_map(|item| match item {
                syn::ImplItem::Fn(func)
                    if self.is_trait_impl() || matches!(func.vis, syn::Visibility::Public(_)) =>
                {
                    Some(FnIR::Impl(FnImplIR::new(func.clone())))
                }
                _ => None
            })
            .collect()
    }

    fn functions(&self) -> syn::Result<Vec<FnIR>> {
        let own_functions = self
            .code
            .items
            .clone()
            .into_iter()
            .filter_map(|item| match item {
                syn::ImplItem::Fn(func) => Some(func),
                _ => None
            })
            .filter_map(|func| {
                // Private functions of a plain impl are not entry points: only their
                // `#[odra(..)]` attributes are checked, not their names nor arguments.
                if self.is_trait_impl() || matches!(func.vis, syn::Visibility::Public(_)) {
                    Some(FnIR::try_from(func))
                } else {
                    attr::partition_attributes(func.attrs).err().map(Err)
                }
            });
        let delegated_functions = self.delegated_functions()?.into_iter().map(|delegated| {
            FnIR::try_from(delegated.function).map(|fn_ir| match fn_ir {
                FnIR::Impl(ir) => {
                    FnIR::Impl(ir.delegated_to(delegated.target, delegated.has_own_docs))
                }
                FnIR::Def(ir) => FnIR::Def(ir)
            })
        });
        errors::collect_all(own_functions.chain(delegated_functions))
    }

    fn delegated_functions(&self) -> syn::Result<Vec<DelegatedFn>> {
        let delegate_items = self
            .code
            .items
            .iter()
            .filter_map(|item| match item {
                ImplItem::Macro(m) => Some(m),
                _ => None
            })
            .map(|m| syn::parse2::<Delegate>(m.mac.tokens.clone()))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(delegate_items
            .into_iter()
            .flat_map(|delegate| delegate.functions)
            .collect())
    }

    fn is_trait_impl(&self) -> bool {
        self.code.trait_.is_some()
    }
}

try_parse!(syn::ItemTrait => ModuleTraitIR);

impl ModuleTraitIR {
    pub fn self_code(&self) -> syn::Result<syn::ItemTrait> {
        let mut code = self.code.clone();
        code.items.iter_mut().for_each(|item| {
            if let syn::TraitItem::Fn(func) = item {
                func.attrs = attr::other_attributes(func.attrs.clone());
            }
        });
        Ok(code)
    }

    fn module_ident(&self) -> syn::Ident {
        self.code.ident.clone()
    }

    fn functions(&self) -> syn::Result<Vec<FnIR>> {
        errors::collect_all(self.code.items.iter().filter_map(|item| match item {
            syn::TraitItem::Fn(func) => Some(FnIR::try_from(func.clone())),
            _ => None
        }))
    }
}

pub enum FnIR {
    Impl(FnImplIR),
    Def(FnTraitIR)
}

impl FnIR {
    pub fn attributes(&self) -> &[syn::Attribute] {
        match self {
            FnIR::Impl(ir) => ir.attrs(),
            FnIR::Def(ir) => ir.attrs()
        }
    }

    pub fn docs(&self) -> Vec<String> {
        utils::syn::string_docs(self.attributes())
    }

    pub fn fn_type(&self) -> FnType {
        let name = self.name_str();
        match name.as_str() {
            CONSTRUCTOR_NAME => FnType::Constructor,
            UPGRADER_NAME => FnType::Upgrader,
            FACTORY_NAME => FnType::Factory,
            FACTORY_UPGRADE_NAME => FnType::FactoryUpgrader,
            BATCH_FACTORY_UPGRADE_NAME => FnType::FactoryBatchUpgrader,
            _ => FnType::Regular
        }
    }
}

const PROTECTED_FUNCTIONS: [&str; 6] = [
    "new",
    "env",
    "address",
    "new_contract",
    "upgrade_child_contract",
    "batch_upgrade_child_contract"
];
const PROTECTED_ARGS: [&str; 2] = ["gas", "attached_value"];

fn validate_fn_name(name: &Ident) -> syn::Result<()> {
    if PROTECTED_FUNCTIONS.contains(&name.to_string().as_str()) {
        return Err(errors::reserved_fn_name(name));
    }
    Ok(())
}

/// An argument is `name: Type` (the receiver aside), and the name is not one every call carries.
fn validate_fn_arg(arg: &syn::FnArg) -> syn::Result<()> {
    let name = match arg {
        syn::FnArg::Receiver(_) => return Ok(()),
        syn::FnArg::Typed(syn::PatType { pat, .. }) => match pat.as_ref() {
            syn::Pat::Ident(pat) => &pat.ident,
            _ => return Err(errors::arg_without_name(arg))
        }
    };
    if PROTECTED_ARGS.contains(&name.to_string().as_str()) {
        return Err(errors::reserved_arg_name(name));
    }
    Ok(())
}

/// Functions whose wasm entry point is built by a dedicated codegen path
/// (see `AddEntryPointStmtItem`) that does not read the `#[odra(..)]` flags.
/// An attribute here would be dropped without a trace, so it is rejected instead.
const NO_ODRA_ATTRIBUTE_FUNCTIONS: [&str; 2] = [CONSTRUCTOR_NAME, UPGRADER_NAME];

fn validate_fn_attrs(fn_name: &str, attrs: &[syn::Attribute]) -> syn::Result<()> {
    if !NO_ODRA_ATTRIBUTE_FUNCTIONS.contains(&fn_name) {
        return Ok(());
    }

    errors::check_all(
        attrs
            .iter()
            .filter(|attr| attr.path().is_ident("odra"))
            .map(|attr| {
                let odra_attr = OdraAttribute::try_from(attr.clone())?;
                match odra_attr.arg_names().first() {
                    Some(arg) => Err(errors::odra_attr_on_special_fn(attr, arg, fn_name)),
                    None => Ok(())
                }
            })
    )
}

impl TryFrom<syn::TraitItemFn> for FnIR {
    type Error = syn::Error;

    fn try_from(code: syn::TraitItemFn) -> Result<Self, Self::Error> {
        let fn_name = utils::syn::function_name(&code.sig);
        validate_fn_ir(&fn_name, &code.sig, &code.attrs)?;
        Ok(Self::Def(FnTraitIR::new(code)))
    }
}

impl TryFrom<syn::ImplItemFn> for FnIR {
    type Error = syn::Error;

    fn try_from(code: syn::ImplItemFn) -> Result<Self, Self::Error> {
        let fn_name = utils::syn::function_name(&code.sig);
        validate_fn_ir(&fn_name, &code.sig, &code.attrs)?;
        Ok(Self::Impl(FnImplIR::new(code)))
    }
}

fn validate_fn_ir(
    fn_name: &str,
    sig: &syn::Signature,
    attrs: &[syn::Attribute]
) -> syn::Result<()> {
    errors::check_all([
        validate_fn_name(&sig.ident),
        errors::check_all(sig.inputs.iter().map(validate_fn_arg)),
        attr::partition_attributes(attrs.to_vec()).map(|_| ()),
        validate_fn_attrs(fn_name, attrs)
    ])
}

impl FnIR {
    pub fn name(&self) -> Ident {
        self.sig().ident.clone()
    }

    pub fn try_name(&self) -> Ident {
        format_ident!("try_{}", self.name())
    }

    pub fn try_no_ret_name(&self) -> Ident {
        format_ident!("try_{}_no_ret", self.name())
    }

    pub fn execute_name(&self) -> Ident {
        format_ident!("execute_{}", self.name())
    }

    pub fn name_str(&self) -> String {
        self.name().to_string()
    }

    pub fn is_payable(&self) -> bool {
        let (odra_attrs, _) = attr::partition_attributes(self.attrs()).unwrap_or_default();
        odra_attrs.iter().any(OdraAttribute::is_payable)
    }

    pub fn is_non_reentrant(&self) -> bool {
        let (odra_attrs, _) = attr::partition_attributes(self.attrs()).unwrap_or_default();
        odra_attrs.iter().any(OdraAttribute::is_non_reentrant)
    }

    pub fn is_offchain(&self) -> bool {
        let (odra_attrs, _) = attr::partition_attributes(self.attrs()).unwrap_or_default();
        odra_attrs.iter().any(OdraAttribute::is_offchain)
    }

    pub fn arg_names(&self) -> Vec<Ident> {
        utils::syn::function_arg_names(self.sig())
    }

    pub fn has_args(&self) -> bool {
        !self.arg_names().is_empty()
    }

    pub fn named_args(&self) -> Vec<FnArgIR> {
        utils::syn::function_named_args(self.sig())
            .into_iter()
            .map(|arg| FnArgIR::new(arg.to_owned()))
            .collect()
    }

    pub fn return_type(&self) -> syn::ReturnType {
        utils::syn::function_return_type(self.sig())
    }

    pub fn try_return_type(&self) -> syn::ReturnType {
        let ty = match self.return_type() {
            syn::ReturnType::Default => parse_quote!(()),
            syn::ReturnType::Type(_, box ty) => parse_quote!(#ty)
        };
        let odra_result = utils::ty::odra_result(ty);
        utils::misc::ret_ty(&odra_result)
    }

    pub fn typed_args(&self) -> Vec<syn::PatType> {
        utils::syn::function_typed_args(self.sig())
    }

    pub fn generics(&self) -> &syn::Generics {
        &self.sig().generics
    }

    pub fn raw_typed_args(&self) -> Vec<syn::PatType> {
        self.typed_args()
            .into_iter()
            .map(|pat_ty| syn::PatType {
                ty: Box::new(utils::ty::unreferenced_ty(&pat_ty.ty)),
                ..pat_ty
            })
            .collect()
    }

    pub fn is_mut(&self) -> bool {
        let receiver = utils::syn::receiver_arg(self.sig());
        receiver.map(|r| r.mutability.is_some()).unwrap_or_default()
    }

    pub fn is_restricted(&self) -> bool {
        self.is_constructor() || self.is_upgrader()
    }

    pub fn is_constructor(&self) -> bool {
        self.name_str() == CONSTRUCTOR_NAME
    }

    pub fn is_upgrader(&self) -> bool {
        self.name_str() == UPGRADER_NAME
    }

    pub fn is_factory(&self) -> bool {
        self.name_str() == "factory"
    }

    /// The submodule field a `delegate!`-generated function with no docs of its own forwards
    /// to; its schema description is inherited from that submodule's entry point.
    pub fn inherits_description_from(&self) -> Option<&syn::ExprField> {
        match self {
            FnIR::Impl(ir) => ir
                .delegation
                .as_ref()
                .filter(|delegation| !delegation.has_own_docs)
                .map(|delegation| &delegation.target),
            FnIR::Def(_) => None
        }
    }

    fn attrs(&self) -> Vec<syn::Attribute> {
        match self {
            FnIR::Impl(ir) => ir.attrs(),
            FnIR::Def(ir) => ir.attrs()
        }
        .to_vec()
    }

    fn sig(&self) -> &syn::Signature {
        match self {
            FnIR::Impl(ir) => ir.sig(),
            FnIR::Def(ir) => ir.sig()
        }
    }
}

pub struct FnTraitIR {
    code: syn::TraitItemFn
}

impl FnTraitIR {
    pub fn new(code: syn::TraitItemFn) -> Self {
        Self { code }
    }

    pub fn sig(&self) -> &syn::Signature {
        &self.code.sig
    }

    fn attrs(&self) -> &[syn::Attribute] {
        &self.code.attrs
    }
}

/// Where a function generated by a `delegate!` block forwards its call.
pub struct Delegation {
    /// The submodule field, e.g. `self.metadata`.
    pub target: syn::ExprField,
    /// `false` if the docs were generated, so the schema description is inherited
    /// from the submodule's entry point of the same name.
    pub has_own_docs: bool
}

pub struct FnImplIR {
    code: syn::ImplItemFn,
    delegation: Option<Delegation>
}

impl FnImplIR {
    pub fn new(code: syn::ImplItemFn) -> Self {
        Self {
            code,
            delegation: None
        }
    }

    fn delegated_to(self, target: syn::ExprField, has_own_docs: bool) -> Self {
        Self {
            delegation: Some(Delegation {
                target,
                has_own_docs
            }),
            ..self
        }
    }

    pub fn sig(&self) -> &syn::Signature {
        &self.code.sig
    }

    fn attrs(&self) -> &[syn::Attribute] {
        &self.code.attrs
    }

    pub fn raw(&self) -> syn::ImplItemFn {
        self.code.clone()
    }
}

pub struct FnArgIR {
    code: syn::FnArg
}

impl FnArgIR {
    pub fn new(code: syn::FnArg) -> Self {
        FnArgIR { code }
    }

    pub fn raw(&self) -> syn::FnArg {
        self.code.clone()
    }

    pub fn name(&self) -> syn::Result<Ident> {
        match &self.code {
            syn::FnArg::Typed(syn::PatType {
                pat: box syn::Pat::Ident(pat),
                ..
            }) => Ok(pat.ident.clone()),
            _ => Err(errors::arg_without_name(&self.code))
        }
    }

    pub fn name_str(&self) -> syn::Result<String> {
        self.name().map(|i| i.to_string())
    }

    pub fn ty(&self) -> syn::Result<syn::Type> {
        match &self.code {
            syn::FnArg::Typed(syn::PatType { box ty, .. }) => Ok(ty.clone()),
            _ => Err(errors::arg_without_name(&self.code))
        }
    }
    pub fn name_and_ty(&self) -> syn::Result<(String, syn::Type)> {
        match &self.code {
            syn::FnArg::Typed(syn::PatType {
                box ty,
                pat: box syn::Pat::Ident(pat),
                ..
            }) => Ok((pat.ident.to_string(), ty.clone())),
            _ => Err(errors::arg_without_name(&self.code))
        }
    }

    pub fn is_ref(&self) -> bool {
        match &self.code {
            syn::FnArg::Typed(syn::PatType { box ty, .. }) => utils::syn::is_ref(ty),
            _ => false
        }
    }
}

pub struct TypeIR {
    code: syn::Item
}

impl TryFrom<&proc_macro2::TokenStream> for TypeIR {
    type Error = syn::Error;

    fn try_from(stream: &proc_macro2::TokenStream) -> Result<Self, Self::Error> {
        Ok(Self {
            code: syn::parse2::<syn::Item>(stream.clone())?
        })
    }
}

impl TypeIR {
    pub fn self_code(&self) -> &syn::Item {
        &self.code
    }

    pub fn kind(&self) -> syn::Result<TypeKind> {
        match &self.code {
            syn::Item::Enum(e) => {
                errors::check_all(e.variants.iter().map(validate_discriminant))?;
                let is_unit = e.variants.iter().all(|v| v.fields.is_empty());
                let variants = e.variants.iter().cloned().collect();
                if is_unit {
                    Ok(TypeKind::UnitEnum { variants })
                } else {
                    Ok(TypeKind::Enum { variants })
                }
            }
            syn::Item::Struct(syn::ItemStruct { fields, .. }) => {
                let fields = fields
                    .iter()
                    .map(|f| {
                        f.ident
                            .clone()
                            .map(|i| (i, f.ty.clone()))
                            .ok_or_else(|| errors::type_fields_unnamed(fields))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(TypeKind::Struct { fields })
            }
            _ => Err(errors::type_on_other_item(&self.code))
        }
    }
}

/// A variant's discriminant, when given, is a `u16` literal: it is the code of an error and the
/// tag of a serialized enum.
fn validate_discriminant(variant: &syn::Variant) -> syn::Result<()> {
    let Some((_, discriminant)) = &variant.discriminant else {
        return Ok(());
    };
    let is_u16 = match discriminant {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(int),
            ..
        }) => int.base10_parse::<u16>().is_ok(),
        _ => false
    };
    match is_u16 {
        true => Ok(()),
        false => Err(errors::discriminant_not_u16(discriminant))
    }
}

pub enum TypeKind {
    UnitEnum {
        variants: Vec<syn::Variant>
    },
    Enum {
        variants: Vec<syn::Variant>
    },
    Struct {
        fields: Vec<(syn::Ident, syn::Type)>
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_not_reserved_fn_arg() {
        let code: syn::ImplItemFn = syn::parse_quote!(
            pub fn abc(&self, arg_name: u64) {}
        );
        let result = FnIR::try_from(code);
        assert!(result.is_ok());
    }

    // The rejected arguments, attributes and names (reserved arguments, module arguments on an
    // impl or a trait, `payable` on `init`...) are covered by the compile-fail tests of the `odra`
    // crate (`tests/macros/fail`), which check the message and its span.

    #[test]
    fn test_module_args_factory_or_none_on_impl_are_allowed() {
        let item = quote::quote!(
            impl Token {
                pub fn transfer(&mut self) {}
            }
        );
        let attr = quote::quote!(factory = on);
        assert!(ModuleImplIR::try_from((&attr, &item)).is_ok());
        let attr = quote::quote!();
        assert!(ModuleImplIR::try_from((&attr, &item)).is_ok());
    }

    #[test]
    fn test_module_without_args_on_trait_is_allowed() {
        let item = quote::quote!(
            pub trait Token {
                fn transfer(&mut self);
            }
        );
        let attr = quote::quote!();
        assert!(ModuleImplIR::try_from((&attr, &item)).is_ok());
    }

    #[test]
    fn test_payable_regular_fn_is_allowed() {
        let code: syn::ImplItemFn = syn::parse_quote!(
            #[odra(payable)]
            pub fn deposit(&mut self) {}
        );
        assert!(FnIR::try_from(code).is_ok());
    }

    #[test]
    fn test_non_odra_attribute_on_constructor_is_allowed() {
        let code: syn::ImplItemFn = syn::parse_quote!(
            /// Initializes the contract.
            #[allow(dead_code)]
            pub fn init(&mut self) {}
        );
        assert!(FnIR::try_from(code).is_ok());
    }
}

#[cfg(test)]
mod offchain_tests {
    use super::ModuleImplIR;
    use quote::quote;

    fn parse(module: proc_macro2::TokenStream) -> syn::Result<ModuleImplIR> {
        ModuleImplIR::try_from((&quote!(), &module))
    }

    #[test]
    fn offchain_functions_are_split_from_the_entry_points() {
        let module = parse(quote! {
            impl Token {
                pub fn balance_of(&self, owner: &Address) -> U256 { U256::zero() }
                #[odra(offchain)]
                pub fn balances(&self, owners: Vec<Address>) -> Vec<U256> { vec![] }
            }
        })
        .unwrap();
        let names =
            |fns: Vec<super::FnIR>| fns.iter().map(super::FnIR::name_str).collect::<Vec<_>>();
        assert_eq!(
            names(module.functions().unwrap()),
            vec!["balance_of", "balances"]
        );
        assert_eq!(
            names(module.onchain_functions().unwrap()),
            vec!["balance_of"]
        );
        assert_eq!(
            names(module.offchain_functions().unwrap()),
            vec!["balances"]
        );
    }
}
