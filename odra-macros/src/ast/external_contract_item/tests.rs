use crate::test_utils;

use super::ExternalContractImpl;

#[test]
fn external_contract_impl() {
    let ir = test_utils::mock::ext_contract();
    let item = ExternalContractImpl::try_from(&ir).unwrap();
    let expected = quote::quote! {
        #[allow(dead_code)]
        pub trait Token {
            fn balance_of(&self, owner: Address) -> U256;
        }

        /// [Token] Contract Ref.
        pub struct TokenContractRef {
            env: odra::prelude::Rc<odra::ContractEnv>,
            address: odra::prelude::Address,
            attached_value: odra::casper_types::U512,
        }

        impl odra::ContractRef for TokenContractRef {
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

        impl TokenContractRef {
            pub fn balance_of(&self, owner: Address) -> U256 {
                self.env.call_contract(
                    self.address,
                    odra::CallDef::new(
                        odra::prelude::string::String::from("balance_of"),
                        false,
                        {
                            let mut __odra_named_args = odra::casper_types::RuntimeArgs::new();
                            if self.attached_value > odra::casper_types::U512::zero() {
                                let _ = __odra_named_args.insert("amount", self.attached_value);
                            }
                            odra::args::EntrypointArgument::insert_runtime_arg(owner, "owner", &mut __odra_named_args);
                            __odra_named_args
                        }
                    )
                    .with_amount(self.attached_value),
                )
            }
        }

        #[automatically_derived]
        #[cfg(not(target_arch = "wasm32"))]
        impl odra::schema::SchemaErrors for TokenContractRef {}

        #[automatically_derived]
        #[cfg(not(target_arch = "wasm32"))]
        impl odra::schema::SchemaEvents for TokenContractRef {}

        impl Token for TokenContractRef {
            fn balance_of(&self, owner: Address) -> U256 {
                self.env.call_contract(
                    self.address,
                    odra::CallDef::new(
                        odra::prelude::string::String::from("balance_of"),
                        false,
                        {
                            let mut __odra_named_args = odra::casper_types::RuntimeArgs::new();
                            if self.attached_value > odra::casper_types::U512::zero() {
                                let _ = __odra_named_args.insert("amount", self.attached_value);
                            }
                            odra::args::EntrypointArgument::insert_runtime_arg(owner, "owner", &mut __odra_named_args);
                            __odra_named_args
                        }
                    )
                    .with_amount(self.attached_value),
                )
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        mod __token_test_parts {
            use super::*;
            use odra::prelude::*;

            /// [Token] Host Ref.
            pub struct TokenHostRef {
                address: odra::prelude::Address,
                env: odra::host::HostEnv,
                attached_value: odra::casper_types::U512
            }

            impl odra::host::HostRef for TokenHostRef {
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

            impl TokenHostRef {
                pub fn balance_of(&self, owner: Address) -> U256 {
                    self.try_balance_of(owner).unwrap()
                }
            }

            impl TokenHostRef {
                /// Does not fail in case of error, returns `odra::OdraResult` instead.
                pub fn try_balance_of(&self, owner: Address) -> OdraResult<U256> {
                    self.env.call_contract(
                        self.address,
                        odra::CallDef::new(
                            odra::prelude::string::String::from("balance_of"),
                            false,
                            {
                                let mut __odra_named_args = odra::casper_types::RuntimeArgs::new();
                                if self.attached_value > odra::casper_types::U512::zero() {
                                    let _ = __odra_named_args.insert("amount", self.attached_value);
                                }
                                odra::args::EntrypointArgument::insert_runtime_arg(owner, "owner", &mut __odra_named_args);
                                __odra_named_args
                            }
                        ).with_amount(self.attached_value),
                    )
                }
            }

            impl Token for TokenHostRef {
                fn balance_of(&self, owner: Address) -> U256 {
                    self.try_balance_of(owner).unwrap()
                }
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        pub use __token_test_parts::*;
    };

    test_utils::assert_eq(item, expected);
}
