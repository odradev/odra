use quote::quote;

use crate::{ast::factory::parts::FactoryRefItem, test_utils};

#[test]
fn contract_ref() {
    let module = test_utils::mock::module_factory_impl();
    let expected = quote! {
        /// [Erc20Factory] Contract Ref.
        pub struct Erc20FactoryContractRef {
            env: odra::prelude::Rc<odra::ContractEnv>,
            address: odra::prelude::Address,
            attached_value: odra::casper_types::U512,
        }

        impl odra::ContractRef for Erc20FactoryContractRef {
            fn new(env: odra::prelude::Rc<odra::ContractEnv>, address: odra::prelude::Address) -> Self {
                Self {
                    env,
                    address,
                    attached_value: odra::casper_types::U512::zero()
                }
            }

            fn address(&self) -> &odra::prelude::Address {
                &self.address
            }

            fn with_tokens(&self, tokens: odra::casper_types::U512) -> Self {
                Self {
                    address: self.address,
                    env: self.env.clone(),
                    attached_value: tokens,
                }
            }
        }

        impl Erc20FactoryContractRef {
            pub fn new_contract(&mut self, contract_name: odra::prelude::string::String, value: u32) -> (odra::prelude::Address, odra::casper_types::URef) {
                self.env.call_contract(
                    self.address,
                    odra::CallDef::new(
                        odra::prelude::string::String::from("new_contract"),
                        true,
                        {
                            let mut __odra_named_args = odra::casper_types::RuntimeArgs::new();
                            odra::args::EntrypointArgument::insert_runtime_arg(true, "odra_cfg_is_upgradable", &mut __odra_named_args);
                            odra::args::EntrypointArgument::insert_runtime_arg(false, "odra_cfg_is_upgrade", &mut __odra_named_args);
                            odra::args::EntrypointArgument::insert_runtime_arg(true, "odra_cfg_allow_key_override", &mut __odra_named_args);
                            odra::args::EntrypointArgument::insert_runtime_arg(contract_name.clone(), "odra_cfg_package_hash_key_name", &mut __odra_named_args);
                            odra::args::EntrypointArgument::insert_runtime_arg(contract_name, "contract_name", &mut __odra_named_args);
                            odra::args::EntrypointArgument::insert_runtime_arg(value, "value", &mut __odra_named_args);
                            __odra_named_args
                        }
                    )
                )
            }

            pub fn upgrade_child_contract(
                &mut self,
                contract_name: odra::prelude::string::String,
            ) {
                self.env
                    .call_contract(
                        self.address,
                        odra::CallDef::new(
                                odra::prelude::string::String::from("upgrade_child_contract"),
                                true,
                                {
                                    let mut __odra_named_args = odra::casper_types::RuntimeArgs::new();
                                    odra::args::EntrypointArgument::insert_runtime_arg(contract_name, "contract_name", &mut __odra_named_args);
                                    odra::args::EntrypointArgument::insert_runtime_arg(true, "odra_cfg_is_factory_upgrade", &mut __odra_named_args);
                                    odra::args::EntrypointArgument::insert_runtime_arg(true, "odra_cfg_allow_key_override", &mut __odra_named_args);
                                    odra::args::EntrypointArgument::insert_runtime_arg(false, "odra_cfg_create_upgrade_group", &mut __odra_named_args);
                                    __odra_named_args
                                },
                            )
                    )
            }

            pub fn batch_upgrade_child_contract<
                T: Into<odra::casper_types::RuntimeArgs>
            >(
                &mut self,
                args: odra::prelude::BTreeMap<odra::prelude::string::String, T>
            ) {
                self.env
                    .call_contract(
                        self.address,
                        odra::CallDef::new(
                                odra::prelude::string::String::from(
                                    "batch_upgrade_child_contract",
                                ),
                                true,
                                {
                                    let mut __odra_named_args = odra::casper_types::RuntimeArgs::new();
                                    odra::args::EntrypointArgument::insert_runtime_arg(odra::args::BatchUpgradeArgs::from(args), "args", &mut __odra_named_args);
                                    odra::args::EntrypointArgument::insert_runtime_arg(true, "odra_cfg_is_factory_upgrade", &mut __odra_named_args);
                                    odra::args::EntrypointArgument::insert_runtime_arg(true, "odra_cfg_allow_key_override", &mut __odra_named_args);
                                    odra::args::EntrypointArgument::insert_runtime_arg(false, "odra_cfg_create_upgrade_group", &mut __odra_named_args);
                                    __odra_named_args
                                },
                            )
                    )
            }
        }

        #[automatically_derived]
        #[cfg(not(target_arch = "wasm32"))]
        impl odra::schema::SchemaErrors for Erc20FactoryContractRef {}

        #[automatically_derived]
        #[cfg(not(target_arch = "wasm32"))]
        impl odra::schema::SchemaEvents for Erc20FactoryContractRef {}
    };
    let actual = FactoryRefItem::try_from(&module).unwrap();
    test_utils::assert_eq(actual, expected);
}
