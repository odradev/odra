use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};

use crate::{
    ast::wasm_parts::{insert_arg_stmt, new_wasm_env_expr, ModuleWasmPartsItem, WasmPartsKind},
    ir::{FnArgIR, ModuleImplIR},
    utils
};

/// Emits the wasm parts of a factory: the module's wasm parts built for the factory kind, see
/// [ModuleWasmPartsItem].
pub struct FactoryWasmPartsItem(ModuleWasmPartsItem);

impl TryFrom<&ModuleImplIR> for FactoryWasmPartsItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        ModuleWasmPartsItem::try_new(module, WasmPartsKind::Factory).map(Self)
    }
}

impl ToTokens for FactoryWasmPartsItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

/// Emits the three `#[no_mangle]` functions only a factory has: `new_contract`,
/// `upgrade_child_contract` and `batch_upgrade_child_contract`.
pub struct FactoryWasmFnsItem {
    new_contract_fn: NewContractFnItem,
    upgrade_fns: UpgradeChildContractFnsItem
}

impl TryFrom<&ModuleImplIR> for FactoryWasmFnsItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            new_contract_fn: NewContractFnItem::try_from(module)?,
            upgrade_fns: UpgradeChildContractFnsItem::try_from(module)?
        })
    }
}

impl ToTokens for FactoryWasmFnsItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.new_contract_fn.to_tokens(tokens);
        self.upgrade_fns.to_tokens(tokens);
    }
}

/// Emits `#[no_mangle] fn new_contract()`: installs a child with the child's entry points and
/// schemas, forwarding the `init` arguments of the call, emits `XxxContractDeployed` and returns
/// the child's address and access URef.
struct NewContractFnItem {
    child_ident: Ident,
    event_ident: Ident,
    init_args: Vec<FnArgIR>
}

impl TryFrom<&ModuleImplIR> for NewContractFnItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            child_ident: module.child_module_ident(),
            event_ident: format_ident!("{}ContractDeployed", module.module_ident()),
            init_args: module
                .constructor()
                .map(|f| f.named_args())
                .unwrap_or_default()
        })
    }
}

impl ToTokens for NewContractFnItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            child_ident,
            event_ident,
            init_args
        } = self;
        let child_entry_points = utils::ident::child_contract_entry_points();
        let schemas = utils::ident::schemas();
        let named_args = utils::ident::named_args();
        let ty_address = utils::ty::address();
        let ty_string = utils::ty::string();
        let ty_schemas = utils::ty::schemas();
        let ty_has_events = utils::ty::has_events();
        let ty_runtime_args = utils::ty::runtime_args();
        let new_env = new_wasm_env_expr();
        let insert_args = init_args.iter().map(insert_arg_stmt);

        tokens.extend(quote! {
            #[no_mangle]
            fn new_contract() {
                odra::odra_casper_wasm_env::host_functions::override_factory_caller();
                let #schemas = #ty_schemas(<#child_ident as #ty_has_events>::event_schemas());
                let exec_env = {
                    let env = #new_env;
                    let env_rc = Rc::new(env);
                    odra::ExecutionEnv::new(env_rc)
                };
                let mut #named_args = #ty_runtime_args::new();
                #(#insert_args)*

                let (contract_package_hash, access_uref) = odra::odra_casper_wasm_env::host_functions::install_new_contract(
                    #child_entry_points(),
                    #schemas,
                    None,
                    Some(#named_args)
                );
                let address: #ty_address = contract_package_hash.into();

                exec_env.emit_event(#event_ident {
                    contract_name: exec_env.get_named_arg::<#ty_string>("contract_name"),
                    contract_address: address
                });

                odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::ret(
                    odra::odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert::unwrap_or_revert(
                        odra::casper_types::CLValue::from_t((address, access_uref))
                    )
                );
            }
        });
    }
}

/// Emits `#[no_mangle] fn upgrade_child_contract()` and `fn batch_upgrade_child_contract()`:
/// each looks the child up by the name the factory registered it under, upgrades it with the
/// child's entry points, forwarding the `upgrade` arguments, and emits `XxxContractDeployed`.
struct UpgradeChildContractFnsItem {
    child_ident: Ident,
    event_ident: Ident,
    upgrade_args: Vec<(String, syn::Type)>
}

impl TryFrom<&ModuleImplIR> for UpgradeChildContractFnsItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        let upgrade_args = module
            .upgrader()
            .map(|f| f.named_args())
            .unwrap_or_default()
            .iter()
            .map(FnArgIR::name_and_ty)
            .collect();
        Ok(Self {
            child_ident: module.child_module_ident(),
            event_ident: format_ident!("{}ContractDeployed", module.module_ident()),
            upgrade_args
        })
    }
}

impl ToTokens for UpgradeChildContractFnsItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            child_ident,
            event_ident,
            upgrade_args
        } = self;
        let child_entry_points = utils::ident::child_contract_entry_points();
        let schemas = utils::ident::schemas();
        let ty_address = utils::ty::address();
        let ty_string = utils::ty::string();
        let ty_schemas = utils::ty::schemas();
        let ty_has_events = utils::ty::has_events();
        let ty_runtime_args = utils::ty::runtime_args();
        let ty_args_by_name = utils::ty::typed_btree_map(&ty_string, &utils::ty::bytes());
        let new_env = new_wasm_env_expr();
        let unwrap_or_revert =
            quote!(odra::odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert);
        let insert_args = upgrade_args.iter().map(|(name, ty)| {
            quote! { let _ = named_args.insert(#name, exec_env.get_named_arg::<#ty>(#name)); }
        });

        tokens.extend(quote! {
            #[no_mangle]
            fn upgrade_child_contract() {
                use #unwrap_or_revert;
                let #schemas = #ty_schemas(<#child_ident as #ty_has_events>::event_schemas());
                let exec_env = {
                    let env = #new_env;
                    let env_rc = Rc::new(env);
                    odra::ExecutionEnv::new(env_rc)
                };
                let name = exec_env.get_named_arg::<#ty_string>("contract_name");
                let mut named_args = {
                    let mut named_args = #ty_runtime_args::new();
                    #(#insert_args)*
                    named_args
                };
                let contract_key = UnwrapOrRevert::unwrap_or_revert(
                    odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::get_key(&name)
                );
                let package_hash = UnwrapOrRevert::unwrap_or_revert(
                    contract_key.into_package_hash()
                );
                let _ = named_args.insert("odra_cfg_package_hash_to_upgrade", package_hash.value());
                let _ = named_args.insert("odra_cfg_package_hash_key_name", name.clone());
                let contract_package_hash = odra::odra_casper_wasm_env::host_functions::upgrade_contract(
                    #child_entry_points(),
                    #schemas,
                    None,
                    Some(named_args)
                );

                let address: #ty_address = contract_package_hash.into();
                exec_env.emit_event(#event_ident {
                    contract_name: name,
                    contract_address: address
                });
            }

            #[no_mangle]
            fn batch_upgrade_child_contract() {
                use #unwrap_or_revert;
                use odra::casper_types::bytesrepr::FromBytes;

                let #schemas = #ty_schemas(<#child_ident as #ty_has_events>::event_schemas());
                let exec_env = {
                    let env = #new_env;
                    let env_rc = Rc::new(env);
                    odra::ExecutionEnv::new(env_rc)
                };
                let args = exec_env.get_named_arg::<#ty_args_by_name>("args");

                for (name, args_bytes) in args {
                    let contract_key = odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::get_key(&name);
                    if let Some(key) = contract_key {
                        let package_hash = UnwrapOrRevert::unwrap_or_revert(key.into_package_hash());
                        let mut named_args: #ty_runtime_args = UnwrapOrRevert::unwrap_or_revert(FromBytes::from_bytes(&args_bytes)).0;
                        let _ = named_args.insert("odra_cfg_package_hash_to_upgrade", package_hash.value());
                        let _ = named_args.insert("odra_cfg_package_hash_key_name", name.clone());
                        let contract_package_hash = odra::odra_casper_wasm_env::host_functions::upgrade_contract(
                            #child_entry_points(),
                            #schemas.clone(),
                            None,
                            Some(named_args)
                        );
                        let address: #ty_address = contract_package_hash.into();

                        exec_env.emit_event(#event_ident {
                            contract_name: name,
                            contract_address: address
                        });
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests;
