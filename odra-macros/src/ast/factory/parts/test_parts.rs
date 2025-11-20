use quote::{ToTokens, TokenStreamExt};

use crate::{
    ast::{
        factory::parts::{host_ref_item::FactoryHostRefItem, FactoryDeployImplItem},
        host_ref_item::HasIdentTraitImplItem,
        parts_utils::{UsePreludeItem, UseSuperItem},
        test_parts::PartsModuleItem
    },
    ModuleImplIR
};

pub struct FactoryTestPartsItem {
    module_item: PartsModuleItem,
    use_super: UseSuperItem,
    use_prelude: UsePreludeItem,
    host_ref: FactoryHostRefItem,
    trait_has_ident_impl_item: HasIdentTraitImplItem,
    deployer: FactoryDeployImplItem
}

impl TryFrom<&ModuleImplIR> for FactoryTestPartsItem {
    type Error = syn::Error;

    fn try_from(ir: &ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            module_item: PartsModuleItem::try_from(ir)?,
            use_super: UseSuperItem,
            use_prelude: UsePreludeItem,
            host_ref: FactoryHostRefItem::try_from(ir)?,
            trait_has_ident_impl_item: HasIdentTraitImplItem::try_from(ir)?,
            deployer: FactoryDeployImplItem::try_from(ir)?
        })
    }
}

impl ToTokens for FactoryTestPartsItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let module_item = &self.module_item;
        let use_super = &self.use_super;
        let use_prelude = &self.use_prelude;
        let host_ref = &self.host_ref;
        let trait_has_ident_impl_item = &self.trait_has_ident_impl_item;
        let deployer = &self.deployer;

        tokens.append_all(quote::quote! {
            #module_item {
                #use_super
                #use_prelude
                #host_ref
                #trait_has_ident_impl_item
                #deployer
            }
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::{self, mock};

    #[test]
    fn test_parts() {
        let module = mock::module_factory_impl();
        let actual = FactoryTestPartsItem::try_from(&module).unwrap();

        let expected = quote::quote! {
            #[cfg(not(target_arch = "wasm32"))]
            mod __erc20_factory_test_parts {
                use super::*;
                use odra::prelude::*;

                /// [Erc20Factory] Host Ref.
                pub struct Erc20FactoryHostRef {
                    address: odra::prelude::Address,
                    env: odra::host::HostEnv,
                    attached_value: odra::casper_types::U512
                }

                impl odra::host::HostRef for Erc20FactoryHostRef {
                    fn new(address: odra::prelude::Address, env: odra::host::HostEnv) -> Self {
                        Self {
                            address,
                            env,
                            attached_value: Default::default()
                        }
                    }

                    fn with_tokens(&self, tokens: odra::casper_types::U512) -> Self {
                        Self {
                            address: self.address,
                            env: self.env.clone(),
                            attached_value: tokens
                        }
                    }

                    fn contract_address(&self) -> odra::prelude::Address {
                        self.address
                    }

                    fn env(&self) -> &odra::host::HostEnv {
                        &self.env
                    }

                    fn get_event<T>(&self, index: i32) -> Result<T, odra::EventError>
                    where
                        T: odra::casper_types::bytesrepr::FromBytes + odra::casper_event_standard::EventInstance,
                    {
                        self.env.get_event(self, index)
                    }

                    fn last_call(&self) -> odra::ContractCallResult {
                        self.env.last_call_result(self.address)
                    }
                }

                impl Erc20FactoryHostRef {
                    pub fn new_contract(&mut self, contract_name: odra::prelude::string::String, value: u32) -> (odra::prelude::Address, odra::casper_types::URef) {
                        self.try_new_contract(contract_name, value).unwrap()
                    }
                    
                    pub fn upgrade_child_contract(&mut self, contract_name: odra::prelude::string::String) {
                        self.try_upgrade_child_contract(contract_name).unwrap()
                    }

                    pub fn batch_upgrade_child_contract<
                        S: Into<odra::args::BatchUpgradeArgs<T>>,
                        T: Into<odra::casper_types::RuntimeArgs>
                    >(
                        &mut self,
                        args: S
                    ) {
                        self.try_batch_upgrade_child_contract(args).unwrap()
                    }
                }

                impl Erc20FactoryHostRef {
                    /// Does not fail in case of error, returns `odra::OdraResult` instead.
                    pub fn try_new_contract(&mut self, contract_name: odra::prelude::string::String, value: u32) -> OdraResult<(odra::prelude::Address, odra::casper_types::URef)> {
                        self.env
                            .call_contract(
                                self.address,
                                odra::CallDef::new(
                                    odra::prelude::string::String::from("new_contract"),
                                    true,
                                    {
                                        let mut named_args = odra::casper_types::RuntimeArgs::new();
                                        odra::args::EntrypointArgument::insert_runtime_arg(true, "odra_cfg_is_upgradable", &mut named_args);
                                        odra::args::EntrypointArgument::insert_runtime_arg(false, "odra_cfg_is_upgrade", &mut named_args);
                                        odra::args::EntrypointArgument::insert_runtime_arg(true, "odra_cfg_allow_key_override", &mut named_args);
                                        odra::args::EntrypointArgument::insert_runtime_arg(contract_name.clone(), "odra_cfg_package_hash_key_name", &mut named_args);
                                        odra::args::EntrypointArgument::insert_runtime_arg(contract_name, "contract_name", &mut named_args);
                                        odra::args::EntrypointArgument::insert_runtime_arg(value, "value", &mut named_args);
                                        named_args
                                    },
                                )
                            )
                    }

                    /// Does not fail in case of error, returns `odra::OdraResult` instead.
                    pub fn try_upgrade_child_contract(
                        &mut self,
                        contract_name: odra::prelude::string::String,
                    ) -> OdraResult<()> {
                        self.env
                            .call_contract(
                                self.address,
                                odra::CallDef::new(
                                    odra::prelude::string::String::from("upgrade_child_contract"),
                                    true,
                                    {
                                        let mut named_args = odra::casper_types::RuntimeArgs::new();
                                        odra::args::EntrypointArgument::insert_runtime_arg(contract_name, "contract_name", &mut named_args);
                                        odra::args::EntrypointArgument::insert_runtime_arg(true, "odra_cfg_is_factory_upgrade", &mut named_args);
                                        odra::args::EntrypointArgument::insert_runtime_arg(true, "odra_cfg_allow_key_override", &mut named_args);
                                        odra::args::EntrypointArgument::insert_runtime_arg(false, "odra_cfg_create_upgrade_group", &mut named_args);
                                        named_args
                                    },
                                ),
                            )
                    }
                    /// Does not fail in case of error, returns `odra::OdraResult` instead.
                    pub fn try_batch_upgrade_child_contract<
                        S: Into<odra::args::BatchUpgradeArgs<T>>,
                        T: Into<odra::casper_types::RuntimeArgs>
                    >(
                        &mut self,
                        args: S
                    ) -> OdraResult<()> {
                        self.env
                            .call_contract(
                                self.address,
                                odra::CallDef::new(
                                    odra::prelude::string::String::from(
                                        "batch_upgrade_child_contract",
                                    ),
                                    true,
                                    {
                                        let mut named_args = odra::casper_types::RuntimeArgs::new();
                                        odra::args::EntrypointArgument::insert_runtime_arg(args.into(), "args", &mut named_args);
                                        odra::args::EntrypointArgument::insert_runtime_arg(true, "odra_cfg_is_factory_upgrade", &mut named_args);
                                        odra::args::EntrypointArgument::insert_runtime_arg(true, "odra_cfg_allow_key_override", &mut named_args);
                                        odra::args::EntrypointArgument::insert_runtime_arg(false, "odra_cfg_create_upgrade_group", &mut named_args);
                                        named_args
                                    },
                                ),
                            )
                    }
                }

                impl odra::contract_def::HasIdent for Erc20FactoryHostRef {
                    fn ident() -> odra::prelude::string::String {
                        Erc20Factory::ident()
                    }
                }

                impl odra::host::EntryPointsCallerProvider for Erc20FactoryHostRef {
                    fn entry_points_caller(env: &odra::host::HostEnv) -> odra::entry_point_callback::EntryPointsCaller {
                        let entry_points = odra::prelude::vec![
                            odra::entry_point_callback::EntryPoint::new(
                                odra::prelude::string::String::from("new_contract"),
                                odra::prelude::vec![
                                    odra::entry_point_callback::Argument::new::<odra::prelude::string::String>(
                                        odra::prelude::string::String::from("contract_name")
                                    ),
                                    odra::entry_point_callback::Argument::new::<u32>(
                                        odra::prelude::string::String::from("value")
                                    )
                                ]
                            )
                        ];
                        odra::entry_point_callback::EntryPointsCaller::new(env.clone(), entry_points, |contract_env, call_def| {
                            if call_def.entry_point() == "new_contract" {
                                return Err(OdraError::VmError(
                                    odra::VmError::Other(odra::prelude::String::from("Factory is not supported for this configuration."))
                                ));
                            }
                            Err(OdraError::VmError(
                                odra::VmError::NoSuchMethod(odra::prelude::String::from(call_def.entry_point()))
                            ))
                        })
                    }
                }
            }
        };

        test_utils::assert_eq(actual, expected);
    }
}
