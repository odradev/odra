use derive_try_from_ref::TryFromRef;
use quote::{ToTokens, TokenStreamExt};

use crate::{
    ast::{
        contract_ref_item::{ContractRefStructItem, ContractRefTraitImplItem},
        ref_utils::{self, SchemaErrorsItem, SchemaEventsItem}
    },
    ModuleImplIR
};

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleImplIR)]
#[err(syn::Error)]
pub struct FactoryRefItem {
    struct_item: ContractRefStructItem,
    trait_impl_item: ContractRefTraitImplItem,
    impl_item: ContractRefImplItem,
    schema_errors_item: SchemaErrorsItem,
    schema_events_item: SchemaEventsItem
}
struct ContractRefImplItem {
    ref_ident: syn::Ident,
    factory_fns: Vec<syn::ItemFn>
}

impl TryFrom<&'_ ModuleImplIR> for ContractRefImplItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        let fns = vec![module.factory_fn(), module.factory_upgrade_fn(), module.factory_batch_upgrade_fn()];
        Ok(Self {
            ref_ident: module.contract_ref_ident()?,
            factory_fns: fns.iter().map(|fun| ref_utils::contract_function_item(fun, false)).collect()
        })
    }
}

impl ToTokens for ContractRefImplItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ref_ident = &self.ref_ident;
        let fns = &self.factory_fns;
        tokens.append_all(quote::quote! {
            impl #ref_ident {
                #(#fns)*
            }
        });
    }
}

#[cfg(test)]
mod test {
    use quote::quote;

    use crate::{ast::factory::parts::FactoryRefItem, test_utils};

    #[test]
    fn contract_ref() {
        let module = test_utils::mock::module_factory_impl();
        let expected = quote! {
            /// [Erc20Factory] Contract Ref.
            pub struct Erc20FactoryContractRef {
                env: Rc<odra::ContractEnv>,
                address: Address,
                attached_value: odra::casper_types::U512,
            }

            impl odra::ContractRef for Erc20FactoryContractRef {
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

            impl Erc20FactoryContractRef {
                pub fn new_contract(&mut self, contract_name: odra::prelude::string::String, value: u32) -> (Address, odra::casper_types::URef) {
                    self.env.call_contract(
                        self.address,
                        odra::CallDef::new(
                            odra::prelude::string::String::from("new_contract"),
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
                                        let mut named_args = odra::casper_types::RuntimeArgs::new();
                                        if self.attached_value > odra::casper_types::U512::zero() {
                                            let _ = named_args.insert("amount", self.attached_value);
                                        }
                                        odra::args::EntrypointArgument::insert_runtime_arg(
                                            contract_name.clone(),
                                            "contract_name",
                                            &mut named_args,
                                        );
                                        named_args
                                    },
                                )
                                .with_amount(self.attached_value),
                        )
                }
                
                pub fn batch_upgrade_child_contract(
                    &mut self,
                    default_args: odra::casper_types::bytesrepr::Bytes,
                    names_to_upgrade: odra::prelude::vec::Vec<odra::prelude::string::String>,
                    specific_args: odra::prelude::BTreeMap<
                        odra::prelude::string::String,
                        odra::casper_types::bytesrepr::Bytes,
                    >,
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
                                        let mut named_args = odra::casper_types::RuntimeArgs::new();
                                        if self.attached_value > odra::casper_types::U512::zero() {
                                            let _ = named_args.insert("amount", self.attached_value);
                                        }
                                        odra::args::EntrypointArgument::insert_runtime_arg(
                                            default_args.clone(),
                                            "default_args",
                                            &mut named_args,
                                        );
                                        odra::args::EntrypointArgument::insert_runtime_arg(
                                            names_to_upgrade.clone(),
                                            "names_to_upgrade",
                                            &mut named_args,
                                        );
                                        odra::args::EntrypointArgument::insert_runtime_arg(
                                            specific_args.clone(),
                                            "specific_args",
                                            &mut named_args,
                                        );
                                        named_args
                                    },
                                )
                                .with_amount(self.attached_value),
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
}
