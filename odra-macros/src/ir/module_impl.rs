use proc_macro2::Ident;
use quote::ToTokens;
use syn::{parse_quote, ImplItem};

use super::attr::{self, OdraAttrs};
use super::config::ModuleConfig;
use super::delegate::{Delegate, DelegatedFn};
use super::errors;
use super::function::{FnArgIR, FnIR, ImplContext};
use super::{CONSTRUCTOR_NAME, UPGRADER_NAME};
use crate::utils;

/// The impl block or the trait `#[odra::module]` (or `#[odra::external_contract]`) is applied
/// to. Built with every function validated, see [FnIR].
pub enum ModuleImplIR {
    Impl(ModuleIR),
    Trait(ModuleTraitIR)
}

impl ModuleImplIR {
    /// The IR of an impl block, with `attr` the arguments of the macro.
    pub fn try_new_impl(attr: &proc_macro2::TokenStream, code: syn::ItemImpl) -> syn::Result<Self> {
        let config = syn::parse2::<ModuleConfig>(attr.clone())?;
        errors::check_all(
            config
                .struct_only_args()
                .map(|(name, span)| Err(errors::module_arg_on_impl(name, *span)))
        )?;
        ModuleIR::try_new(code, config).map(Self::Impl)
    }

    /// The IR of a trait, with `attr` the arguments of the macro.
    pub fn try_new_trait(
        attr: &proc_macro2::TokenStream,
        code: syn::ItemTrait
    ) -> syn::Result<Self> {
        let config = syn::parse2::<ModuleConfig>(attr.clone())?;
        errors::check_all(
            config
                .args()
                .map(|(name, span)| Err(errors::module_arg_on_trait(name, *span)))
        )?;
        // A trait function with a visibility (`pub fn`) is a `Verbatim` item to syn. Parsing it
        // as a function reports the `pub` here, instead of skipping the function and leaving the
        // error to rustc on the emitted trait.
        for item in code.items.iter() {
            if let syn::TraitItem::Verbatim(func) = item {
                syn::parse2::<syn::TraitItemFn>(func.clone())?;
            }
        }
        ModuleTraitIR::try_new(code).map(Self::Trait)
    }
}

/// The IR from the macro's token streams, for the test mocks.
#[cfg(test)]
impl TryFrom<(&proc_macro2::TokenStream, &proc_macro2::TokenStream)> for ModuleImplIR {
    type Error = syn::Error;

    fn try_from(
        (attr, item): (&proc_macro2::TokenStream, &proc_macro2::TokenStream)
    ) -> Result<Self, Self::Error> {
        match syn::parse2::<syn::Item>(item.clone())? {
            syn::Item::Impl(code) => Self::try_new_impl(attr, code),
            syn::Item::Trait(code) => Self::try_new_trait(attr, code),
            other => Err(errors::external_contract_on_other_item(&other))
        }
    }
}

impl ModuleImplIR {
    /// The item as the user wrote it, less the `#[odra(..)]` attributes, plus the functions the
    /// `delegate!` blocks generate in place of those blocks.
    pub fn self_code(&self) -> proc_macro2::TokenStream {
        match self {
            ModuleImplIR::Impl(ir) => ir.self_code().into_token_stream(),
            ModuleImplIR::Trait(ir) => ir.self_code().into_token_stream()
        }
    }

    pub fn module_ident(&self) -> Ident {
        match self {
            ModuleImplIR::Impl(ir) => ir.ident.clone(),
            ModuleImplIR::Trait(ir) => ir.code.ident.clone()
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
            ModuleImplIR::Impl(ir) => ir.config.factory,
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

    pub fn module_str(&self) -> String {
        self.module_ident().to_string()
    }

    /// `<Module><suffix>`, spanned as the module ident.
    fn suffixed(&self, suffix: &str) -> Ident {
        let module_ident = self.module_ident();
        Ident::new(&format!("{module_ident}{suffix}"), module_ident.span())
    }

    /// `__<module>_<part>`, the name of a generated module, spanned as the module ident.
    fn parts_mod_ident(&self, part: &str) -> Ident {
        let module_ident = self.module_ident();
        let snake_cased = utils::string::camel_to_snake(&module_ident);
        Ident::new(&format!("__{snake_cased}_{part}"), module_ident.span())
    }

    pub fn host_ref_ident(&self) -> Ident {
        self.suffixed("HostRef")
    }

    pub fn contract_ref_ident(&self) -> Ident {
        self.suffixed("ContractRef")
    }

    pub fn factory_module_ident(&self) -> Ident {
        self.suffixed("Factory")
    }

    /// The module a factory impl stands for. `#[odra::factory]` sees only the `<Module>Factory`
    /// ident that [factory_module_ident](Self::factory_module_ident) made, so the suffix is
    /// stripped back; for any other module it is the module ident itself.
    pub fn child_module_ident(&self) -> Ident {
        let module_ident = self.module_ident();
        let module_str = module_ident.to_string();
        let child = module_str.strip_suffix("Factory").unwrap_or(&module_str);
        Ident::new(child, module_ident.span())
    }

    pub fn init_args_ident(&self) -> Ident {
        self.suffixed("InitArgs")
    }

    pub fn upgrade_args_ident(&self) -> Ident {
        self.suffixed("UpgradeArgs")
    }

    pub fn schema_mod_ident(&self) -> Ident {
        self.parts_mod_ident("schema")
    }

    pub fn test_parts_mod_ident(&self) -> Ident {
        self.parts_mod_ident("test_parts")
    }

    pub fn wasm_parts_mod_ident(&self) -> Ident {
        self.parts_mod_ident("wasm_parts")
    }

    pub fn exec_parts_mod_ident(&self) -> Ident {
        self.parts_mod_ident("exec_parts")
    }

    /// The functions that become entry points of the deployed contract: everything but the
    /// `#[odra(offchain)]` ones.
    pub fn onchain_functions(&self) -> Vec<&FnIR> {
        self.functions()
            .iter()
            .filter(|f| !f.is_offchain())
            .collect()
    }

    /// The `#[odra(offchain)]` functions: run on the host against the contract's state, never
    /// deployed.
    pub fn offchain_functions(&self) -> Vec<&FnIR> {
        self.functions()
            .iter()
            .filter(|f| f.is_offchain())
            .collect()
    }

    pub fn host_mut_ret_functions(&self) -> Vec<&FnIR> {
        self.functions()
            .iter()
            .filter(|f| f.is_mut() && f.return_type() != syn::ReturnType::Default)
            .collect()
    }

    pub fn constructor(&self) -> Option<&FnIR> {
        self.functions()
            .iter()
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
        FnIR::unchecked(factory_fn)
    }

    pub fn factory_upgrade_fn(&self) -> FnIR {
        let args = self.upgrader_args();
        let ty_string = utils::ty::string();
        FnIR::unchecked(parse_quote! {
            pub fn upgrade_child_contract(&mut self, contract_name: #ty_string, #args) {
            }
        })
    }

    pub fn factory_batch_upgrade_fn(&self) -> FnIR {
        let rt_args = utils::ty::runtime_args();
        let btree_map = utils::ty::typed_btree_map(&utils::ty::string(), &parse_quote!(T));
        FnIR::unchecked(parse_quote! {
            pub fn batch_upgrade_child_contract<T: Into<#rt_args>>(&mut self, args: #btree_map) {
            }
        })
    }

    pub fn upgrader(&self) -> Option<&FnIR> {
        self.functions()
            .iter()
            .find(|f| f.name_str() == UPGRADER_NAME)
    }

    /// Every function of the item, validated when the IR was built: the public ones of a plain
    /// impl, all of a trait or trait impl, then the ones the `delegate!` blocks generate.
    pub fn functions(&self) -> &[FnIR] {
        match self {
            ModuleImplIR::Impl(ir) => &ir.functions,
            ModuleImplIR::Trait(ir) => &ir.functions
        }
    }

    fn constructor_args(&self) -> syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma> {
        self.constructor()
            .map(|fn_ir| fn_ir.args())
            .unwrap_or_default()
            .iter()
            .map(FnArgIR::raw)
            .collect::<syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>>()
    }

    fn upgrader_args(&self) -> syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma> {
        self.upgrader()
            .map(|fn_ir| fn_ir.args())
            .unwrap_or_default()
            .iter()
            .map(FnArgIR::raw)
            .collect::<syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>>()
    }
}

/// A plain or trait `impl` block of a module.
pub struct ModuleIR {
    code: syn::ItemImpl,
    /// The module: the last segment of the impl's self type, which is a path.
    ident: Ident,
    /// Boxed, as the impl variant would otherwise dwarf the trait one.
    config: Box<ModuleConfig>,
    functions: Vec<FnIR>
}

impl ModuleIR {
    fn try_new(code: syn::ItemImpl, config: ModuleConfig) -> syn::Result<Self> {
        let ctx = ImplContext {
            is_trait_impl: code.trait_.is_some(),
            is_factory: config.factory
        };
        let (ident, functions) = errors::both(
            Self::module_ident(&code.self_ty),
            Self::parse_functions(&code, ctx)
        )?;
        Ok(Self {
            code,
            ident,
            config: Box::new(config),
            functions
        })
    }

    fn module_ident(self_ty: &syn::Type) -> syn::Result<Ident> {
        match self_ty {
            syn::Type::Path(type_path) => type_path
                .path
                .segments
                .last()
                .map(|seg| seg.ident.clone())
                .ok_or_else(|| errors::impl_self_type_not_a_path(self_ty)),
            ty => Err(errors::impl_self_type_not_a_path(ty))
        }
    }

    pub(super) fn self_code(&self) -> syn::ItemImpl {
        let mut code = self.code.clone();
        // include delegated functions
        code.items.extend(
            self.functions
                .iter()
                .filter(|f| f.is_delegated())
                .filter_map(FnIR::raw_impl)
                .map(syn::ImplItem::Fn)
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
        code
    }

    /// Is the function an entry point: public, or any function of a trait impl.
    fn is_entry_point(func: &syn::ImplItemFn, is_trait_impl: bool) -> bool {
        is_trait_impl || matches!(func.vis, syn::Visibility::Public(_))
    }

    fn parse_functions(code: &syn::ItemImpl, ctx: ImplContext) -> syn::Result<Vec<FnIR>> {
        let own_functions = code
            .items
            .iter()
            .filter_map(|item| match item {
                syn::ImplItem::Fn(func) => Some(func),
                _ => None
            })
            .filter_map(|func| {
                // Private functions of a plain impl are not entry points: only their
                // `#[odra(..)]` attributes are checked, not their names nor arguments.
                if Self::is_entry_point(func, ctx.is_trait_impl) {
                    Some(FnIR::try_new_impl(func.clone(), ctx))
                } else {
                    OdraAttrs::parse(&func.attrs).err().map(Err)
                }
            });
        let delegated_functions = Self::delegated_functions(code)?
            .into_iter()
            .map(|delegated| {
                FnIR::try_new_impl(delegated.function, ctx)
                    .map(|fn_ir| fn_ir.delegated_to(delegated.target, delegated.has_own_docs))
            });
        errors::collect_all(own_functions.chain(delegated_functions))
    }

    fn delegated_functions(code: &syn::ItemImpl) -> syn::Result<Vec<DelegatedFn>> {
        let delegate_items = code
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
}

/// A trait describing a contract's entry points: `#[odra::module]` on a trait, or an
/// `#[odra::external_contract]`.
pub struct ModuleTraitIR {
    code: syn::ItemTrait,
    functions: Vec<FnIR>
}

impl ModuleTraitIR {
    fn try_new(code: syn::ItemTrait) -> syn::Result<Self> {
        let functions = errors::collect_all(code.items.iter().filter_map(|item| match item {
            syn::TraitItem::Fn(func) => Some(FnIR::try_from(func.clone())),
            _ => None
        }))?;
        Ok(Self { code, functions })
    }

    pub fn self_code(&self) -> syn::ItemTrait {
        let mut code = self.code.clone();
        code.items.iter_mut().for_each(|item| {
            if let syn::TraitItem::Fn(func) = item {
                func.attrs = attr::other_attributes(func.attrs.clone());
            }
        });
        code
    }
}
