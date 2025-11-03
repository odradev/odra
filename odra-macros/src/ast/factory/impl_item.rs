use derive_try_from_ref::TryFromRef;

use crate::{
    ast::{
        blueprint::BlueprintItem,
        factory::parts::{
            FactoryContractItem, FactoryExecPartsItem, FactoryHasEntrypointsImplItem,
            FactoryRefItem, FactoryTestPartsItem, FactoryWasmPartsItem
        },
        schema::{SchemaCustomTypesItem, FactorySchemaEntrypointsItem},
        test_parts::TestPartsReexportItem
    },
    ModuleImplIR
};

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleImplIR)]
#[err(syn::Error)]
pub struct FactoryModuleImplItem {
    #[expr(input.self_code()?)]
    self_code: proc_macro2::TokenStream,
    has_entrypoints_item: FactoryHasEntrypointsImplItem,
    ref_item: FactoryRefItem,
    test_parts: FactoryTestPartsItem,
    test_parts_reexport: TestPartsReexportItem,
    exec_parts: FactoryExecPartsItem,
    wasm_parts: FactoryWasmPartsItem,
    contract_item: FactoryContractItem,
    blueprint: BlueprintItem,
    schema_entrypoints: FactorySchemaEntrypointsItem,
    schema_custom_types: SchemaCustomTypesItem
}

#[cfg(test)]
mod test {
    use crate::{test_utils, FactoryModuleImplItem};

    #[test]
    fn test_factory_item() {
        let ir = test_utils::mock::module_factory_impl();
        let factory_item =
            FactoryModuleImplItem::try_from(&ir).expect("Failed to create FactoryModuleImplItem");

        let expected = quote::quote! {
            impl Erc20 {
                pub fn init(&mut self, value: u32) {
                    self.value.set(value);
                }

                /// Returns the total supply of the token.
                pub fn total_supply(&self) -> U256 {
                    self.total_supply.get_or_default()
                }

                /// Pay to mint.
                pub fn pay_to_mint(&mut self) {
                    let attached_value = self.env().attached_value();
                    self.total_supply
                        .set(self.total_supply() + U256::from(attached_value.as_u64()));
                }

                /// Approve.
                pub fn approve(&mut self, to: &Address, amount: &U256, msg: Maybe<String>) {
                    self.env.emit_event(Approval {
                        owner: self.env.caller(),
                        spender: to,
                        value: amount
                    });
                }
            }

            impl odra::contract_def::HasEntrypoints for Erc20 {
                fn entrypoints() -> odra::prelude::vec::Vec<odra::contract_def::Entrypoint> {
                    odra::prelude::vec![
                        odra::contract_def::Entrypoint {
                            name: odra::prelude::string::String::from("init"),
                            args: odra::prelude::vec![],
                            is_mutable: true,
                            return_ty: <() as odra::casper_types::CLTyped>::cl_type(),
                            ty: odra::contract_def::EntrypointType::Constructor,
                            attributes: odra::prelude::vec![]
                        },
                        odra::contract_def::Entrypoint {
                            name: odra::prelude::string::String::from("factory"),
                            args: odra::prelude::vec![
                                odra::args::odra_argument::<String>("contract_name"),
                                odra::args::odra_argument::<u32>("value")
                            ],
                            is_mutable: true,
                            return_ty: <(Address, odra::casper_types::URef) as odra::casper_types::CLTyped>::cl_type(),
                            ty: odra::contract_def::EntrypointType::Public,
                            attributes: odra::prelude::vec![]
                        }
                    ]
                }
            }

            /// [Erc20] Contract Ref.
            pub struct Erc20ContractRef {
                env: Rc<odra::ContractEnv>,
                address: Address,
                attached_value: odra::casper_types::U512,
            }

            impl odra::ContractRef for Erc20ContractRef {
                fn new(env: Rc<odra::ContractEnv>, address: Address) -> Self {
                    Self {
                        env,
                        address,
                        attached_value: odra::casper_types::U512::zero()
                    }
                }

                fn address(&self) -> &Address {
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

            impl Erc20ContractRef {
                pub fn factory(&mut self, contract_name: String, value: u32) -> (Address, odra::casper_types::URef) {
                    self.env.call_contract(
                        self.address,
                        odra::CallDef::new(
                            odra::prelude::string::String::from("factory"),
                            true,
                            {
                                let mut named_args = odra::casper_types::RuntimeArgs::new();
                                if self.attached_value > odra::casper_types::U512::zero() {
                                    let _ = named_args.insert("amount", self.attached_value);
                                }
                                odra::args::EntrypointArgument::insert_runtime_arg(contract_name.clone(), "contract_name", &mut named_args);
                                odra::args::EntrypointArgument::insert_runtime_arg(value.clone(), "value", &mut named_args);
                                named_args
                            }
                        )
                        .with_amount(self.attached_value),
                    )
                }
            }

            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaErrors for Erc20ContractRef {}

            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaEvents for Erc20ContractRef {}

            #[cfg(not(target_arch = "wasm32"))]
            mod __erc20_test_parts {
                use super::*;
                use odra::prelude::*;

                /// [Erc20] Host Ref.
                pub struct Erc20HostRef {
                    address: Address,
                    env: odra::host::HostEnv,
                    attached_value: odra::casper_types::U512
                }

                impl odra::host::HostRef for Erc20HostRef {
                    fn new(address: Address, env: odra::host::HostEnv) -> Self {
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

                    fn contract_address(&self) -> Address {
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

                impl Erc20HostRef {
                    pub fn factory(&mut self, contract_name: String, value: u32) -> (Address, odra::casper_types::URef) {
                        self.try_factory(contract_name, value).unwrap()
                    }
                }

                impl Erc20HostRef {
                    /// Does not fail in case of error, returns `odra::OdraResult` instead.
                    pub fn try_factory(&mut self, contract_name: String, value: u32) -> OdraResult<(Address, odra::casper_types::URef)> {
                        self.env
                            .call_contract(
                                self.address,
                                odra::CallDef::new(
                                    odra::prelude::string::String::from("factory"),
                                    true,
                                    {
                                        let mut named_args = odra::casper_types::RuntimeArgs::new();
                                        let _ = named_args.insert("contract_name", contract_name.clone());
                                        let _ = named_args.insert("value", value.clone());
                                        let _ = named_args.insert("odra_cfg_is_upgradable", true);
                                        let _ = named_args.insert("odra_cfg_is_upgrade", false);
                                        let _ = named_args.insert("odra_cfg_allow_key_override", true);
                                        let _ = named_args.insert("odra_cfg_package_hash_key_name", contract_name);
                                        named_args
                                    },
                                )
                            )
                    }
                }

                impl odra::contract_def::HasIdent for Erc20HostRef {
                    fn ident() -> odra::prelude::string::String {
                        Erc20::ident()
                    }
                }

                impl odra::host::EntryPointsCallerProvider for Erc20HostRef {
                    fn entry_points_caller(env: &odra::host::HostEnv) -> odra::entry_point_callback::EntryPointsCaller {
                        let entry_points = odra::prelude::vec![
                            odra::entry_point_callback::EntryPoint::new(
                                odra::prelude::string::String::from("factory"),
                                odra::prelude::vec![
                                    odra::entry_point_callback::Argument::new::<String>(
                                        odra::prelude::string::String::from("contract_name")
                                    ),
                                    odra::entry_point_callback::Argument::new::<u32>(
                                        odra::prelude::string::String::from("value")
                                    )
                                ]
                            )
                        ];
                        odra::entry_point_callback::EntryPointsCaller::new(env.clone(), entry_points, |contract_env, call_def| {
                            if call_def.entry_point() == "factory" {
                                return Err(
                                    OdraError::VmError(
                                        odra::VmError::Other(
                                            odra::prelude::String::from(
                                                "Factory is not supported for this configuration.",
                                            ),
                                        ),
                                    ),
                                );
                            }
                            Err(OdraError::VmError(
                                odra::VmError::NoSuchMethod(odra::prelude::String::from(call_def.entry_point()))
                            ))
                        })
                    }
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            pub use __erc20_test_parts::*;

            #[allow(missing_docs)]
            mod __erc20_exec_parts {
                use super::*;
                use odra::prelude::*;

                #[inline]
                pub fn execute_init(env: odra::ContractEnv) {
                    let env_rc = Rc::new(env);
                    let exec_env = odra::ExecutionEnv::new(env_rc.clone());
                    let value = exec_env.get_named_arg::<u32>("value");
                    let mut contract = <Erc20 as Module>::new(env_rc);
                    let result = contract.init(value);
                    return result;
                }

                #[inline]
                pub fn execute_total_supply(env: odra::ContractEnv) -> U256 {
                    let env_rc = Rc::new(env);
                    let contract = <Erc20 as Module>::new(env_rc);
                    let result = contract.total_supply();
                    return result;
                }

                #[inline]
                pub fn execute_pay_to_mint(env: odra::ContractEnv) {
                    let env_rc = Rc::new(env);
                    let exec_env = odra::ExecutionEnv::new(env_rc.clone());
                    exec_env.handle_attached_value();
                    let mut contract = <Erc20 as Module>::new(env_rc);
                    let result = contract.pay_to_mint();
                    exec_env.clear_attached_value();
                    return result;
                }

                #[inline]
                pub fn execute_approve(env: odra::ContractEnv) {
                    let env_rc = Rc::new(env);
                    let exec_env = odra::ExecutionEnv::new(env_rc.clone());
                    exec_env.non_reentrant_before();
                    let to = exec_env.get_named_arg::<Address>("to");
                    let amount = exec_env.get_named_arg::<U256>("amount");
                    let msg = exec_env.get_named_arg::<Maybe<String>>("msg");
                    let mut contract = <Erc20 as Module>::new(env_rc);
                    let result = contract.approve(&to, &amount, msg);
                    exec_env.non_reentrant_after();
                    return result;
                }
            }

            #[cfg(target_arch = "wasm32")]
            #[cfg(odra_module = "Erc20")]
            mod __erc20_wasm_parts {
                use super::*;
                use odra::prelude::*;

                #[inline]
                fn entry_points() -> odra::casper_types::EntryPoints {
                    let mut entry_points = odra::casper_types::EntryPoints::new();
                    entry_points
                        .add_entry_point(
                            odra::casper_types::EntityEntryPoint::new(
                                "init",
                                vec![odra::args::parameter::<u32>("value")]
                                    .into_iter()
                                    .filter_map(|x| x)
                                    .collect(),
                                <() as odra::casper_types::CLTyped>::cl_type(),
                                odra::casper_types::EntryPointAccess::Template,
                                odra::casper_types::EntryPointType::Called,
                                odra::casper_types::EntryPointPayment::Caller,
                            ),
                        );
                    entry_points
                        .add_entry_point(odra::casper_types::EntityEntryPoint::new(
                            "total_supply",
                            vec![],
                            <U256 as odra::casper_types::CLTyped>::cl_type(),
                            odra::casper_types::EntryPointAccess::Template,
                            odra::casper_types::EntryPointType::Called,
                            odra::casper_types::EntryPointPayment::Caller,
                        )
                    );
                    entry_points
                        .add_entry_point(
                            odra::casper_types::EntityEntryPoint::new(
                                "pay_to_mint",
                                vec![],
                                <() as odra::casper_types::CLTyped>::cl_type(),
                                odra::casper_types::EntryPointAccess::Template,
                                odra::casper_types::EntryPointType::Called,
                                odra::casper_types::EntryPointPayment::Caller,
                            ),
                        );
                    entry_points
                            .add_entry_point(
                            odra::casper_types::EntityEntryPoint::new(
                                "approve",
                                vec![
                                    odra::args::parameter:: < Address > ("to"),
                                    odra::args::parameter:: < U256 > ("amount"),
                                    odra::args::parameter:: < Maybe < String > > ("msg")
                                ].into_iter().filter_map(|x| x).collect(),
                                <() as odra::casper_types::CLTyped>::cl_type(),
                                odra::casper_types::EntryPointAccess::Template,
                                odra::casper_types::EntryPointType::Called,
                                odra::casper_types::EntryPointPayment::Caller,
                            ),
                        );
                    entry_points
                        .add_entry_point(
                            odra::casper_types::EntityEntryPoint::new(
                                "factory",
                                vec![odra::args::parameter::<u32>("value")]
                                    .into_iter()
                                    .filter_map(|x| x)
                                    .collect(),
                                <odra::casper_types::Key as odra::casper_types::CLTyped>::cl_type(),
                                odra::casper_types::EntryPointAccess::Public,
                                odra::casper_types::EntryPointType::Factory,
                                odra::casper_types::EntryPointPayment::Caller,
                            ),
                        );
                    entry_points
                }

                #[no_mangle]
                fn call() {
                    let schemas = odra::casper_event_standard::Schemas(
                        <Erc20 as odra::contract_def::HasEvents>::event_schemas()
                    );
                    let exec_env = {
                        let env = odra::odra_casper_wasm_env::WasmContractEnv::new_env();
                        let env_rc = Rc::new(env);
                        odra::ExecutionEnv::new(env_rc)
                    };

                    odra::odra_casper_wasm_env::host_functions::install_or_upgrade(
                        entry_points(),
                        schemas,
                        Option::<odra::casper_types::RuntimeArgs>::None
                    );
                }

                #[no_mangle]
                fn factory() {
                    let mut entry_points = odra::casper_types::EntryPoints::new();
                    entry_points
                        .add_entry_point(
                            odra::casper_types::EntityEntryPoint::new(
                                "init",
                                vec![odra::args::parameter::<u32>("value")]
                                    .into_iter()
                                    .filter_map(|x| x)
                                    .collect(),
                                <() as odra::casper_types::CLTyped>::cl_type(),
                                odra::casper_types::EntryPointAccess::Groups(vec![odra::casper_types::Group::new("constructor_group")]),
                                odra::casper_types::EntryPointType::Called,
                                odra::casper_types::EntryPointPayment::Caller,
                            ),
                        );
                    entry_points
                        .add_entry_point(odra::casper_types::EntityEntryPoint::new(
                            "total_supply",
                            vec![],
                            <U256 as odra::casper_types::CLTyped>::cl_type(),
                            odra::casper_types::EntryPointAccess::Public,
                            odra::casper_types::EntryPointType::Called,
                            odra::casper_types::EntryPointPayment::Caller,
                        )
                    );
                    entry_points
                        .add_entry_point(
                            odra::casper_types::EntityEntryPoint::new(
                                "pay_to_mint",
                                vec![],
                                <() as odra::casper_types::CLTyped>::cl_type(),
                                odra::casper_types::EntryPointAccess::Public,
                                odra::casper_types::EntryPointType::Called,
                                odra::casper_types::EntryPointPayment::Caller,
                            ),
                        );
                    entry_points
                            .add_entry_point(
                            odra::casper_types::EntityEntryPoint::new(
                                "approve",
                                vec![
                                    odra::args::parameter:: < Address > ("to"),
                                    odra::args::parameter:: < U256 > ("amount"),
                                    odra::args::parameter:: < Maybe < String > > ("msg")
                                ].into_iter().filter_map(|x| x).collect(),
                                <() as odra::casper_types::CLTyped>::cl_type(),
                                odra::casper_types::EntryPointAccess::Public,
                                odra::casper_types::EntryPointType::Called,
                                odra::casper_types::EntryPointPayment::Caller,
                            ),
                        );
                    let schemas = odra::casper_event_standard::Schemas(
                        <Erc20 as odra::contract_def::HasEvents>::event_schemas()
                    );
                    let exec_env = {
                        let env = odra::odra_casper_wasm_env::WasmContractEnv::new_env();
                        let env_rc = Rc::new(env);
                        odra::ExecutionEnv::new(env_rc)
                    };

                    let mut named_args = odra::casper_types::RuntimeArgs::new();
                    odra::args::EntrypointArgument::insert_runtime_arg(
                        exec_env.get_named_arg::<u32>("value"),
                        "value",
                        &mut named_args
                    );

                    let (contract_package_hash, access_uref) = odra::odra_casper_wasm_env::host_functions::install_new_contract(
                        entry_points,
                        schemas,
                        Some(named_args)
                    );
                    let address: Address = contract_package_hash.into();

                    exec_env.emit_event(Erc20ContractDeployed {
                        contract_name: exec_env.get_named_arg::<odra::prelude::string::String>("contract_name"),
                        contract_address: address
                    });

                    odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::ret(
                        odra::odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert::unwrap_or_revert(
                            odra::casper_types::CLValue::from_t((address, access_uref))
                        )
                    );
                }

                #[no_mangle]
                fn init() {
                    __erc20_exec_parts::execute_init(odra::odra_casper_wasm_env::WasmContractEnv::new_env());
                }

                #[no_mangle]
                fn total_supply() {
                    let result = __erc20_exec_parts::execute_total_supply(odra::odra_casper_wasm_env::WasmContractEnv::new_env());
                    odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::ret(
                        odra::odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert::unwrap_or_revert(
                            odra::casper_types::CLValue::from_t(result)
                        )
                    );
                }

                #[no_mangle]
                fn pay_to_mint() {
                    __erc20_exec_parts::execute_pay_to_mint(odra::odra_casper_wasm_env::WasmContractEnv::new_env());
                }

                #[no_mangle]
                fn approve() {
                    __erc20_exec_parts::execute_approve(odra::odra_casper_wasm_env::WasmContractEnv::new_env());
                }
            }

            impl odra::OdraContract for Erc20 {
                #[cfg(not(target_arch = "wasm32"))]
                type HostRef = Erc20HostRef;
                type ContractRef = Erc20ContractRef;
                #[cfg(not(target_arch = "wasm32"))]
                type InitArgs = odra::host::NoArgs;
                #[cfg(not(target_arch = "wasm32"))]
                type UpgradeArgs = odra::host::NoArgs;
            }

            #[cfg(odra_module = "Erc20")]
            mod __erc20_schema {
                use super::*;
                #[no_mangle]
                #[cfg(not(target_arch = "wasm32"))]
                fn module_schema() -> odra::contract_def::ContractBlueprint {
                    odra::contract_def::ContractBlueprint::new::<Erc20>()
                }
            }
        #[automatically_derived]
        #[cfg(not(target_arch = "wasm32"))]
        impl odra::schema::SchemaEntrypoints for Erc20 {
            fn schema_entrypoints() -> odra::prelude::vec::Vec<
                odra::schema::casper_contract_schema::Entrypoint,
            > {
                odra::prelude::vec![
                    odra::schema::entry_point::<()>(
                        "init",
                        "",
                        true,
                        odra::prelude::vec![]
                    ),
                    odra::schema::entry_point::<(Address, odra::casper_types::URef)>(
                        "factory", 
                        "", 
                        true, 
                        odra::prelude::vec![
                            odra::schema::argument::<String>("contract_name"),
                            odra::schema::argument::<u32>("value")
                        ]
                    )
                ]
            }
        }
        #[automatically_derived]
        #[cfg(not(target_arch = "wasm32"))]
        impl odra::schema::SchemaCustomTypes for Erc20 {
            fn schema_types() -> odra::prelude::vec::Vec<
                Option<odra::schema::casper_contract_schema::CustomType>,
            > {
                odra::prelude::BTreeSet::<
                    Option<odra::schema::casper_contract_schema::CustomType>,
                >::new()
                    .into_iter()
                    .chain(<u32 as odra::schema::SchemaCustomTypes>::schema_types())
                    .chain(<U256 as odra::schema::SchemaCustomTypes>::schema_types())
                    .chain(<Address as odra::schema::SchemaCustomTypes>::schema_types())
                    .chain(<Maybe<String> as odra::schema::SchemaCustomTypes>::schema_types())
                    .chain(<Self as odra::schema::SchemaEvents>::custom_types())
                    .collect()
            }
        }

        };

        test_utils::assert_eq(factory_item, expected);
    }
}
