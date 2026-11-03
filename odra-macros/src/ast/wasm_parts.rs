use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::parse_quote;

use crate::{
    ast::{
        factory::FactoryWasmFnsItem,
        parts_utils::{UsePreludeItem, UseSuperItem}
    },
    ir::{FnArgIR, FnIR, FnType, ModuleImplIR},
    utils
};

/// Which contract the wasm parts are built for: the module itself, or the factory that
/// deploys and upgrades instances of it.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WasmPartsKind {
    Module,
    Factory
}

/// Emits `mod __xxx_wasm_parts`, compiled only for wasm32 and only when the crate is built for
/// this module: the `entry_points()` listing, the `call()` installer, for a factory its
/// `new_contract` / `upgrade_child_contract` / `batch_upgrade_child_contract`, and one
/// `#[no_mangle]` function per entry point that runs it through the exec parts.
pub struct ModuleWasmPartsItem {
    module_name: String,
    mod_ident: Ident,
    entry_points_fn: EntryPointsFnItem,
    call_fn: CallFnItem,
    factory_fns: Option<FactoryWasmFnsItem>,
    entry_points: Vec<NoMangleFnItem>
}

impl ModuleWasmPartsItem {
    pub(crate) fn try_new(module: &ModuleImplIR, kind: WasmPartsKind) -> syn::Result<Self> {
        let functions = match kind {
            WasmPartsKind::Module => module.onchain_functions(),
            WasmPartsKind::Factory => module.functions().iter().collect()
        };
        let exec_parts_mod_ident = module.exec_parts_mod_ident()?;
        Ok(Self {
            module_name: module.module_str()?,
            mod_ident: module.wasm_parts_mod_ident()?,
            entry_points_fn: EntryPointsFnItem::new(module, kind, &functions),
            call_fn: CallFnItem::try_new(module, kind)?,
            factory_fns: match kind {
                WasmPartsKind::Module => None,
                WasmPartsKind::Factory => Some(FactoryWasmFnsItem::try_from(module)?)
            },
            entry_points: functions
                .iter()
                .map(|f| NoMangleFnItem::new(f, &exec_parts_mod_ident, kind))
                .collect()
        })
    }
}

impl TryFrom<&ModuleImplIR> for ModuleWasmPartsItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        Self::try_new(module, WasmPartsKind::Module)
    }
}

impl ToTokens for ModuleWasmPartsItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            module_name,
            mod_ident,
            entry_points_fn,
            call_fn,
            factory_fns,
            entry_points
        } = self;
        let wasm32_attr = utils::attr::wasm32();
        let odra_module_attr = utils::attr::odra_module(module_name);
        let use_super = UseSuperItem;
        let use_prelude = UsePreludeItem;

        tokens.extend(quote! {
            #wasm32_attr
            #odra_module_attr
            mod #mod_ident {
                #use_super
                #use_prelude

                #entry_points_fn
                #call_fn
                #factory_fns
                #(#entry_points)*
            }
        });
    }
}

/// Emits `fn entry_points() -> EntryPoints` listing the contract's entry points. A module lists
/// its entry points as installed; a factory lists them as templates (what a call to a child looks
/// like) plus its own three, and also emits `fn child_contract_entry_points()` with the installed
/// form, which `new_contract` and the upgrades hand to the host.
struct EntryPointsFnItem {
    entry_points: Vec<syn::Expr>,
    child_entry_points: Option<Vec<syn::Expr>>
}

impl EntryPointsFnItem {
    fn new(module: &ModuleImplIR, kind: WasmPartsKind, functions: &[&FnIR]) -> Self {
        match kind {
            WasmPartsKind::Module => Self {
                entry_points: functions.iter().copied().map(installed_entry_point).collect(),
                child_entry_points: None
            },
            WasmPartsKind::Factory => Self {
                entry_points: functions
                    .iter()
                    .copied()
                    .map(template_entry_point)
                    .chain([
                        factory_ep(parameters(&module.factory_fn())),
                        factory_upgrade_ep(parameters(&module.factory_upgrade_fn())),
                        factory_batch_upgrade_ep()
                    ])
                    .collect(),
                child_entry_points: Some(functions.iter().copied().map(installed_entry_point).collect())
            }
        }
    }
}

impl ToTokens for EntryPointsFnItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ty_entry_points = utils::ty::entry_points();
        let entry_points = utils::ident::entry_points();
        let child_entry_points = utils::ident::child_contract_entry_points();
        let add = utils::ident::add_entry_point();

        let listing = |fn_ident: &Ident, items: &[syn::Expr]| {
            quote! {
                #[inline]
                fn #fn_ident() -> #ty_entry_points {
                    use odra::entry_point::EntityEntryPointsExt;
                    let mut #entry_points = #ty_entry_points::new();
                    #(#entry_points.#add(#items);)*
                    #entry_points
                }
            }
        };

        tokens.extend(listing(&entry_points, &self.entry_points));
        if let Some(items) = &self.child_entry_points {
            tokens.extend(listing(&child_entry_points, items));
        }
    }
}

/// Emits `#[no_mangle] fn call()`, the installer the host runs on deploy and on upgrade. A module
/// reads its `init` (or `upgrade`) arguments from the call and forwards them; a factory has none.
struct CallFnItem {
    kind: WasmPartsKind,
    module_ident: Ident,
    init_args: Option<Vec<FnArgIR>>,
    upgrade_args: Option<Vec<FnArgIR>>
}

impl CallFnItem {
    fn try_new(module: &ModuleImplIR, kind: WasmPartsKind) -> syn::Result<Self> {
        Ok(Self {
            kind,
            module_ident: module.module_ident()?,
            init_args: module.constructor().map(|f| f.named_args()),
            upgrade_args: module.upgrader().map(|f| f.named_args())
        })
    }
}

impl ToTokens for CallFnItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let module_ident = &self.module_ident;
        let ty_schemas = utils::ty::schemas();
        let ty_has_events = utils::ty::has_events();
        let ty_runtime_args = utils::ty::runtime_args();
        let schemas = utils::ident::schemas();
        let entry_points = utils::ident::entry_points();
        let named_args = utils::ident::named_args();
        let new_env = new_wasm_env_expr();

        // The arguments of `init` or `upgrade` read from the call, if the module has that function.
        let forwarded_args = |args: &Option<Vec<FnArgIR>>| match args {
            Some(args) => {
                let inserts = args.iter().map(insert_arg_stmt);
                quote! {{
                    Some({
                        let mut #named_args = #ty_runtime_args::new();
                        #(#inserts)*
                        #named_args
                    })
                }}
            }
            None => quote!(Option::<#ty_runtime_args>::None)
        };
        let (read_args, install_args) = match self.kind {
            WasmPartsKind::Module => {
                let init_args = forwarded_args(&self.init_args);
                let upgrade_args = forwarded_args(&self.upgrade_args);
                (
                    quote! {
                        let is_upgrade = exec_env.get_named_arg::<bool>("odra_cfg_is_upgrade");
                        let #named_args = if is_upgrade { #upgrade_args } else { #init_args };
                    },
                    quote!(#named_args)
                )
            }
            WasmPartsKind::Factory => (quote!(), quote!(Option::<#ty_runtime_args>::None))
        };

        tokens.extend(quote! {
            #[no_mangle]
            fn call() {
                let #schemas = #ty_schemas(<#module_ident as #ty_has_events>::event_schemas());
                let exec_env = {
                    let env = #new_env;
                    let env_rc = Rc::new(env);
                    odra::ExecutionEnv::new(env_rc)
                };
                #read_args
                odra::odra_casper_wasm_env::host_functions::install_or_upgrade(
                    #entry_points(),
                    #schemas,
                    {
                        #[cfg(odra_contract_schema)]
                        let contract_schema = Some(include_str!(env!("ODRA_CONTRACT_SCHEMA_PATH")));
                        #[cfg(not(odra_contract_schema))]
                        let contract_schema: Option<&'static str> = None;
                        contract_schema
                    },
                    #install_args
                );
            }
        });
    }
}

/// Emits `#[no_mangle] fn xxx()`, the wasm export of one entry point: it runs the function
/// through the exec parts with a fresh wasm env and returns the result to the host.
struct NoMangleFnItem {
    fn_ident: Ident,
    exec_parts_mod_ident: Ident,
    execute_ident: Ident,
    returns_value: bool,
    /// A factory's child is installed by the factory, so `init` and `upgrade` see the factory as
    /// their caller unless the real caller is restored first.
    override_caller: bool
}

impl NoMangleFnItem {
    fn new(func: &FnIR, exec_parts_mod_ident: &Ident, kind: WasmPartsKind) -> Self {
        Self {
            fn_ident: func.name(),
            exec_parts_mod_ident: exec_parts_mod_ident.clone(),
            execute_ident: func.execute_name(),
            returns_value: !matches!(func.return_type(), syn::ReturnType::Default),
            override_caller: kind == WasmPartsKind::Factory
                && [FnType::Constructor, FnType::Upgrader].contains(&func.fn_type())
        }
    }
}

impl ToTokens for NoMangleFnItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            fn_ident,
            exec_parts_mod_ident,
            execute_ident,
            ..
        } = self;
        let result = utils::ident::result();
        let new_env = new_wasm_env_expr();

        let override_caller = self.override_caller.then(|| {
            quote!(odra::odra_casper_wasm_env::host_functions::override_factory_caller();)
        });
        let body = match self.returns_value {
            false => quote!(#exec_parts_mod_ident::#execute_ident(#new_env);),
            true => quote! {
                let #result = #exec_parts_mod_ident::#execute_ident(#new_env);
                odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::ret(
                    odra::odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert::unwrap_or_revert(
                        odra::casper_types::CLValue::from_t(#result)
                    )
                );
            }
        };

        tokens.extend(quote! {
            #[no_mangle]
            fn #fn_ident() {
                #override_caller
                #body
            }
        });
    }
}

/// The `EntryPoint` of a function as the installed contract declares it.
pub(crate) fn installed_entry_point(func: &FnIR) -> syn::Expr {
    let args = parameters(func);
    if func.is_constructor() {
        constructor_ep(args)
    } else if func.is_upgrader() {
        upgrader_ep(args)
    } else {
        regular_ep(
            func.name_str(),
            args,
            return_cl_type(func),
            func.is_payable(),
            func.is_non_reentrant()
        )
    }
}

/// The `EntryPoint` of a child function as its factory lists it: name, arguments and return
/// type only, enough for a host to build calls to the child without its code.
fn template_entry_point(func: &FnIR) -> syn::Expr {
    template_ep(func.name_str(), parameters(func), return_cl_type(func))
}

/// `vec![odra::args::parameter::<Ty>("name"), ..]` for the function's named arguments.
pub(crate) fn parameters(func: &FnIR) -> syn::Expr {
    let params = func
        .named_args()
        .iter()
        .map(|arg| new_parameter(arg.name_str(), arg.ty().clone()))
        .collect::<Vec<_>>();
    syn::parse_quote!(vec![#(#params),*])
}

/// The `CLType` of the function's return type, `()` when it has none.
pub(crate) fn return_cl_type(func: &FnIR) -> syn::Expr {
    match func.return_type() {
        syn::ReturnType::Default => unit_cl_type(),
        syn::ReturnType::Type(_, ty) => as_cl_type(&ty)
    }
}

/// Reads the argument from the call (`exec_env`) and puts it into `__odra_named_args`.
pub(crate) fn insert_arg_stmt(arg: &FnArgIR) -> syn::Stmt {
    let (name, ty) = arg.name_and_ty();
    let named_args = utils::ident::named_args();
    syn::parse_quote!(odra::args::EntrypointArgument::insert_runtime_arg(
        exec_env.get_named_arg::<#ty>(#name),
        #name,
        &mut #named_args
    );)
}

/// `WasmContractEnv::new_env()`, the env of one wasm call.
pub(crate) fn new_wasm_env_expr() -> syn::Expr {
    syn::parse_quote!(odra::odra_casper_wasm_env::WasmContractEnv::new_env())
}

// --- The `odra::entry_point::EntryPoint` variants -----------------------------------------------

pub(crate) fn constructor_ep(args_expr: syn::Expr) -> syn::Expr {
    parse_quote!(odra::entry_point::EntryPoint::Constructor {
        args: #args_expr
    })
}
pub(crate) fn upgrader_ep(args_expr: syn::Expr) -> syn::Expr {
    parse_quote!(odra::entry_point::EntryPoint::Upgrader {
        args: #args_expr
    })
}
pub(crate) fn regular_ep(
    name: String,
    args_expr: syn::Expr,
    ret_ty_expr: syn::Expr,
    is_payable: bool,
    is_non_reentrant: bool
) -> syn::Expr {
    parse_quote!(odra::entry_point::EntryPoint::Regular {
        name: #name,
        args: #args_expr,
        ret_ty: #ret_ty_expr,
        is_non_reentrant: #is_non_reentrant,
        is_payable: #is_payable
    })
}
pub(crate) fn factory_ep(args_expr: syn::Expr) -> syn::Expr {
    parse_quote!(odra::entry_point::EntryPoint::Factory {
        args: #args_expr,
    })
}
pub(crate) fn factory_upgrade_ep(args_expr: syn::Expr) -> syn::Expr {
    parse_quote!(odra::entry_point::EntryPoint::FactoryUpgrade {
        args: #args_expr,
    })
}
pub(crate) fn factory_batch_upgrade_ep() -> syn::Expr {
    parse_quote!(odra::entry_point::EntryPoint::FactoryBatchUpgrade)
}
pub(crate) fn template_ep(name: String, args_expr: syn::Expr, ret_ty_expr: syn::Expr) -> syn::Expr {
    parse_quote!(odra::entry_point::EntryPoint::Template {
        name: #name,
        args: #args_expr,
        ret_ty: #ret_ty_expr
    })
}

fn new_parameter(name: String, ty: syn::Type) -> syn::Expr {
    let ty = utils::ty::unreferenced_ty(&ty);

    parse_quote!(odra::args::parameter::<#ty>(#name))
}
fn as_cl_type(ty: &syn::Type) -> syn::Expr {
    let ty = utils::ty::unreferenced_ty(ty);
    let ty_cl_typed = utils::ty::cl_typed();
    let ty = utils::syn::as_casted_ty_stream(&ty, ty_cl_typed);
    parse_quote!(#ty::cl_type())
}
fn unit_cl_type() -> syn::Expr {
    let ty_cl_typed = utils::ty::cl_typed();
    parse_quote!(<() as #ty_cl_typed>::cl_type())
}

#[cfg(test)]
mod tests;
