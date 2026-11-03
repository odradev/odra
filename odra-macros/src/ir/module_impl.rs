use proc_macro2::Ident;
use quote::ToTokens;
use syn::{parse_quote, ImplItem};

use super::attr::{self, OdraAttribute};
use super::config::ConfigItem;
use super::delegate::{Delegate, DelegatedFn};
use super::errors::{self, OffchainMisuse};
use super::function::{FnArgIR, FnIR, FnImplIR};
use super::{CONSTRUCTOR_NAME, UPGRADER_NAME};
use crate::utils;

/// The impl block or the trait `#[odra::module]` (or `#[odra::external_contract]`) is applied
/// to. Built with every function validated, see [FnIR].
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

    /// `factory = on` on the impl block.
    pub fn is_factory(&self) -> bool {
        match self {
            ModuleImplIR::Impl(ir) => match &ir.config {
                ConfigItem::Module(cfg) => *cfg.factory,
                ConfigItem::Empty => false
            },
            ModuleImplIR::Trait(_) => false
        }
    }

    /// The trait of `impl Trait for Xxx`, as written (a path may be qualified).
    pub fn impl_trait_path(&self) -> Option<syn::Path> {
        match self {
            ModuleImplIR::Impl(ir) => ir.code.trait_.as_ref().map(|(_, path, _)| path.clone()),
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

/// A plain or trait `impl` block of a module.
pub struct ModuleIR {
    code: syn::ItemImpl,
    config: ConfigItem
}

impl ModuleIR {
    pub(super) fn self_code(&self) -> syn::Result<syn::ItemImpl> {
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
    pub(super) fn unchecked_functions(&self) -> Vec<FnIR> {
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

    pub(super) fn functions(&self) -> syn::Result<Vec<FnIR>> {
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

    pub(super) fn is_trait_impl(&self) -> bool {
        self.code.trait_.is_some()
    }
}

/// A trait describing a contract's entry points: `#[odra::module]` on a trait, or an
/// `#[odra::external_contract]`.
pub struct ModuleTraitIR {
    code: syn::ItemTrait,
    #[allow(dead_code)]
    config: ConfigItem
}

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

    pub(super) fn module_ident(&self) -> syn::Ident {
        self.code.ident.clone()
    }

    pub(super) fn functions(&self) -> syn::Result<Vec<FnIR>> {
        errors::collect_all(self.code.items.iter().filter_map(|item| match item {
            syn::TraitItem::Fn(func) => Some(FnIR::try_from(func.clone())),
            _ => None
        }))
    }
}
