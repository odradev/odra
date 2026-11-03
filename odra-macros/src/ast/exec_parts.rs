use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use crate::{
    ast::parts_utils::{UsePreludeItem, UseSuperItem},
    ir::{FnIR, ModuleImplIR},
    utils
};

/// Which contract the exec parts are built for: the module itself, or the factory that deploys
/// instances of it. A factory's exec parts instantiate the child module.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ExecPartsKind {
    Module,
    Factory
}

/// Emits `mod __xxx_exec_parts`: one `execute_<name>(env)` per function, the body every entry
/// point runs on both the host and in wasm. It reads the arguments from the call, instantiates
/// the module, calls the function and returns its result, with the reentrancy guard and the
/// attached value handled around the call when the function asks for them.
pub struct ModuleExecPartsItem {
    mod_ident: Ident,
    exec_functions: Vec<ExecFunctionItem>
}

impl ModuleExecPartsItem {
    pub(crate) fn try_new(module: &ModuleImplIR, kind: ExecPartsKind) -> syn::Result<Self> {
        let module_ident = match kind {
            ExecPartsKind::Module => module.module_ident()?,
            ExecPartsKind::Factory => module.child_module_ident()?
        };
        Ok(Self {
            mod_ident: module.exec_parts_mod_ident()?,
            exec_functions: module
                .functions()
                .iter()
                .map(|f| ExecFunctionItem::try_new(f, &module_ident, kind))
                .collect::<syn::Result<_>>()?
        })
    }
}

impl TryFrom<&ModuleImplIR> for ModuleExecPartsItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        Self::try_new(module, ExecPartsKind::Module)
    }
}

impl ToTokens for ModuleExecPartsItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mod_ident = &self.mod_ident;
        let exec_functions = &self.exec_functions;
        let missing_docs_attr = utils::attr::missing_docs();
        let use_super = UseSuperItem;
        let use_prelude = UsePreludeItem;

        tokens.extend(quote! {
            #missing_docs_attr
            mod #mod_ident {
                #use_super
                #use_prelude

                #(#exec_functions)*
            }
        });
    }
}

/// Emits `pub fn execute_<name>(__odra_env: ContractEnv) -> Ret { .. }`.
struct ExecFunctionItem {
    execute_ident: Ident,
    return_type: syn::ReturnType,
    /// `None` for a factory's function named `factory`, which is emitted with an empty body and
    /// no return type.
    body: Option<ExecFnBody>
}

impl ExecFunctionItem {
    fn try_new(func: &FnIR, module_ident: &Ident, kind: ExecPartsKind) -> syn::Result<Self> {
        let is_factory_placeholder = kind == ExecPartsKind::Factory && func.is_factory();
        Ok(Self {
            execute_ident: func.execute_name(),
            return_type: match is_factory_placeholder {
                true => syn::ReturnType::Default,
                false => func.return_type()
            },
            body: match is_factory_placeholder {
                true => None,
                false => Some(ExecFnBody::try_new(func, module_ident)?)
            }
        })
    }
}

impl ToTokens for ExecFunctionItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            execute_ident,
            return_type,
            body
        } = self;
        let env = utils::ident::prefixed_env();
        let ty_contract_env = utils::ty::contract_env();

        tokens.extend(quote! {
            #[inline]
            pub fn #execute_ident(#env: #ty_contract_env) #return_type {
                #body
            }
        });
    }
}

/// The body of an `execute_` function: what it reads from the call and how it calls the module.
struct ExecFnBody {
    module_ident: Ident,
    fn_ident: Ident,
    args: Vec<ExecArg>,
    is_mut: bool,
    is_payable: bool,
    is_non_reentrant: bool,
    /// Whether the body needs the `ExecutionEnv`: to read arguments, to guard reentrancy, to
    /// handle the attached value, or because `upgrade` always gets one.
    uses_exec_env: bool
}

/// An argument read from the call with `get_named_arg` and passed to the function, by reference
/// when the function takes a reference.
struct ExecArg {
    name: Ident,
    ty: syn::Type,
    by_ref: bool
}

impl ExecFnBody {
    fn try_new(func: &FnIR, module_ident: &Ident) -> syn::Result<Self> {
        let args = func
            .named_args()
            .iter()
            .map(|arg| ExecArg {
                name: arg.name(),
                ty: utils::ty::unreferenced_ty(arg.ty()),
                by_ref: arg.is_ref()
            })
            .collect();
        Ok(Self {
            module_ident: module_ident.clone(),
            fn_ident: func.name(),
            uses_exec_env: func.is_payable()
                || func.is_non_reentrant()
                || func.has_args()
                || func.is_upgrader(),
            args,
            is_mut: func.is_mut(),
            is_payable: func.is_payable(),
            is_non_reentrant: func.is_non_reentrant()
        })
    }
}

impl ToTokens for ExecFnBody {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            module_ident,
            fn_ident,
            args,
            ..
        } = self;
        let env = utils::ident::prefixed_env();
        let env_rc = utils::ident::env_rc();
        let exec_env = utils::ident::exec_env();
        let contract = utils::ident::contract();
        let result = utils::ident::result();
        let ty_module = utils::ty::module();

        let new_exec_env = self
            .uses_exec_env
            .then(|| quote!(let #exec_env = odra::ExecutionEnv::new(#env_rc.clone());));
        let non_reentrant_before = self
            .is_non_reentrant
            .then(|| quote!(#exec_env.non_reentrant_before();));
        let non_reentrant_after = self
            .is_non_reentrant
            .then(|| quote!(#exec_env.non_reentrant_after();));
        let handle_attached_value = self
            .is_payable
            .then(|| quote!(#exec_env.handle_attached_value();));
        let clear_attached_value = self
            .is_payable
            .then(|| quote!(#exec_env.clear_attached_value();));
        let read_args = args.iter().map(|ExecArg { name, ty, .. }| {
            let arg_name = name.to_string();
            quote!(let #name = #exec_env.get_named_arg::<#ty>(#arg_name);)
        });
        let pass_args = args.iter().map(|ExecArg { name, by_ref, .. }| {
            let ref_token = by_ref.then(|| quote!(&));
            quote!(#ref_token #name)
        });
        let mut_token = self.is_mut.then(|| quote!(mut));

        tokens.extend(quote! {
            let #env_rc = Rc::new(#env);
            #new_exec_env
            #non_reentrant_before
            #handle_attached_value
            #(#read_args)*
            let #mut_token #contract = <#module_ident as #ty_module>::new(#env_rc);
            let #result = #contract.#fn_ident(#(#pass_args),*);
            #clear_attached_value
            #non_reentrant_after
            return #result;
        });
    }
}

#[cfg(test)]
mod tests;
