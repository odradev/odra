use super::RefItem;
use crate::test_utils;
use quote::quote;

#[test]
fn contract_ref() {
    let module = test_utils::mock::module_impl();
    let expected = quote! {
        /// [Erc20] Contract Ref.
        pub struct Erc20ContractRef {
            env: odra::prelude::Rc<odra::ContractEnv>,
            address: odra::prelude::Address,
            attached_value: odra::casper_types::U512,
        }

        impl odra::ContractRef for Erc20ContractRef {
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

        impl Erc20ContractRef {
            /// Initializes the contract with the given parameters.
            pub fn init(&mut self, total_supply: Option<U256>) {
                self.env.call_contract(
                    self.address,
                    odra::CallDef::new(
                        odra::prelude::string::String::from("init"),
                        true,
                        {
                            let mut __odra_named_args = odra::casper_types::RuntimeArgs::new();
                            if self.attached_value > odra::casper_types::U512::zero() {
                                let _ = __odra_named_args.insert("amount", self.attached_value);
                            }
                            odra::args::EntrypointArgument::insert_runtime_arg(total_supply, "total_supply", &mut __odra_named_args);
                            __odra_named_args
                        }
                    )
                    .with_amount(self.attached_value),
                )
            }

            /// Upgrades the contract with the given parameters.
            pub fn upgrade(&mut self, total_supply: Option<U256>) {
                self.env.call_contract(
                    self.address,
                    odra::CallDef::new(
                        odra::prelude::string::String::from("upgrade"),
                        true,
                        {
                            let mut __odra_named_args = odra::casper_types::RuntimeArgs::new();
                            if self.attached_value > odra::casper_types::U512::zero() {
                                let _ = __odra_named_args.insert("amount", self.attached_value);
                            }
                            odra::args::EntrypointArgument::insert_runtime_arg(total_supply, "total_supply", &mut __odra_named_args);
                            __odra_named_args
                        }
                    )
                    .with_amount(self.attached_value),
                )
            }

            /// Returns the total supply of the token.
            pub fn total_supply(&self) -> U256 {
                self.env.call_contract(
                    self.address,
                    odra::CallDef::new(
                        odra::prelude::string::String::from("total_supply"),
                        false,
                        {
                            let mut __odra_named_args = odra::casper_types::RuntimeArgs::new();
                            if self.attached_value > odra::casper_types::U512::zero() {
                                let _ = __odra_named_args.insert("amount", self.attached_value);
                            }
                            __odra_named_args
                        }
                    )
                    .with_amount(self.attached_value),
                )
            }

            /// Pay to mint.
            pub fn pay_to_mint(&mut self) {
                self.env
                    .call_contract(
                        self.address,
                        odra::CallDef::new(
                            odra::prelude::string::String::from("pay_to_mint"),
                            true,
                            {
                                let mut __odra_named_args = odra::casper_types::RuntimeArgs::new();
                                if self.attached_value > odra::casper_types::U512::zero() {
                                    let _ = __odra_named_args.insert("amount", self.attached_value);
                                }
                                __odra_named_args
                            }
                        )
                        .with_amount(self.attached_value),
                    )
            }

            /// Approve.
            pub fn approve(&mut self, to: &Address, amount: &U256, msg: Maybe<String>) {
                self.env
                    .call_contract(
                        self.address,
                        odra::CallDef::new(
                            odra::prelude::string::String::from("approve"),
                            true,
                            {
                                let mut __odra_named_args = odra::casper_types::RuntimeArgs::new();
                                if self.attached_value > odra::casper_types::U512::zero() {
                                    let _ = __odra_named_args.insert("amount", self.attached_value);
                                }
                                odra::args::EntrypointArgument::insert_runtime_arg(to, "to", &mut __odra_named_args);
                                odra::args::EntrypointArgument::insert_runtime_arg(amount, "amount", &mut __odra_named_args);
                                odra::args::EntrypointArgument::insert_runtime_arg(msg, "msg", &mut __odra_named_args);
                                __odra_named_args
                            }
                        )
                        .with_amount(self.attached_value),
                    )
            }

            /// Airdrops the given amount to the given addresses.
            pub fn airdrop(&self, to: &[Address], amount: &U256) {
                self.env
                    .call_contract(
                        self.address,
                        odra::CallDef::new(
                            odra::prelude::string::String::from("airdrop"),
                            false,
                            {
                                let mut __odra_named_args = odra::casper_types::RuntimeArgs::new();
                                if self.attached_value > odra::casper_types::U512::zero() {
                                    let _ = __odra_named_args.insert("amount", self.attached_value);
                                }
                                odra::args::EntrypointArgument::insert_runtime_arg(to, "to", &mut __odra_named_args);
                                odra::args::EntrypointArgument::insert_runtime_arg(amount, "amount", &mut __odra_named_args);
                                __odra_named_args
                            }
                        )
                        .with_amount(self.attached_value),
                    )
            }

            /// Swaps the given amount to the given addresses.
            pub fn swap(&mut self, to: Address, amount: U256) -> U256 {
                self.env
                    .call_contract(
                        self.address,
                        odra::CallDef::new(
                            odra::prelude::string::String::from("swap"),
                            true,
                            {
                                let mut __odra_named_args = odra::casper_types::RuntimeArgs::new();
                                if self.attached_value > odra::casper_types::U512::zero() {
                                    let _ = __odra_named_args.insert("amount", self.attached_value);
                                }
                                odra::args::EntrypointArgument::insert_runtime_arg(to, "to", &mut __odra_named_args);
                                odra::args::EntrypointArgument::insert_runtime_arg(amount, "amount", &mut __odra_named_args);
                                __odra_named_args
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
    };
    let actual = RefItem::try_from(&module).unwrap();
    test_utils::assert_eq(actual, expected);
}

#[test]
fn contract_trait_impl_ref() {
    let module = test_utils::mock::module_trait_impl();
    let expected = quote! {
        /// [Erc20] Contract Ref.
        pub struct Erc20ContractRef {
            env: odra::prelude::Rc<odra::ContractEnv>,
            address: odra::prelude::Address,
            attached_value: odra::casper_types::U512,
        }

        impl odra::ContractRef for Erc20ContractRef {
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

        impl IErc20 for Erc20ContractRef {
            fn total_supply(&self) -> U256 {
                self.env.call_contract(
                    self.address,
                    odra::CallDef::new(
                        odra::prelude::string::String::from("total_supply"),
                        false,
                        {
                            let mut __odra_named_args = odra::casper_types::RuntimeArgs::new();
                            if self.attached_value > odra::casper_types::U512::zero() {
                                let _ = __odra_named_args.insert("amount", self.attached_value);
                            }
                            __odra_named_args
                        }
                    )
                    .with_amount(self.attached_value),
                )
            }

            fn set_total_supply(&mut self) -> U256 {
                self.env
                    .call_contract(
                        self.address,
                        odra::CallDef::new(
                            odra::prelude::string::String::from("set_total_supply"),
                            true,
                            {
                                let mut __odra_named_args = odra::casper_types::RuntimeArgs::new();
                                if self.attached_value > odra::casper_types::U512::zero() {
                                    let _ = __odra_named_args.insert("amount", self.attached_value);
                                }
                                __odra_named_args
                            }
                        )
                        .with_amount(self.attached_value),
                    )
            }

            fn pay_to_mint(&mut self) {
                self.env
                    .call_contract(
                        self.address,
                        odra::CallDef::new(
                            odra::prelude::string::String::from("pay_to_mint"),
                            true,
                            {
                                let mut __odra_named_args = odra::casper_types::RuntimeArgs::new();
                                if self.attached_value > odra::casper_types::U512::zero() {
                                    let _ = __odra_named_args.insert("amount", self.attached_value);
                                }
                                __odra_named_args
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
    };
    let actual = RefItem::try_from(&module).unwrap();
    test_utils::assert_eq(actual, expected);
}

#[test]
fn contract_ref_delegate() {
    let module = test_utils::mock::module_delegation();
    let expected = quote! {
        /// [Erc20] Contract Ref.
        pub struct Erc20ContractRef {
            env: odra::prelude::Rc<odra::ContractEnv>,
            address: odra::prelude::Address,
            attached_value: odra::casper_types::U512,
        }

        impl odra::ContractRef for Erc20ContractRef {
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

        impl Erc20ContractRef {
            /// Returns the total supply of the token.
            pub fn total_supply(&self) -> U256 {
                self.env.call_contract(
                    self.address,
                    odra::CallDef::new(
                        odra::prelude::string::String::from("total_supply"),
                        false,
                        {
                            let mut __odra_named_args = odra::casper_types::RuntimeArgs::new();
                            if self.attached_value > odra::casper_types::U512::zero() {
                                let _ = __odra_named_args.insert("amount", self.attached_value);
                            }
                            __odra_named_args
                        }
                    )
                    .with_amount(self.attached_value),
                )
            }

            /// Returns the owner of the contract.
            pub fn get_owner(&self) -> Address {
                self.env
                    .call_contract(
                        self.address,
                        odra::CallDef::new(
                            odra::prelude::string::String::from("get_owner"),
                            false,
                            {
                                let mut __odra_named_args = odra::casper_types::RuntimeArgs::new();
                                if self.attached_value > odra::casper_types::U512::zero() {
                                    let _ = __odra_named_args.insert("amount", self.attached_value);
                                }
                                __odra_named_args
                            }
                        )
                        .with_amount(self.attached_value),
                    )
            }

            /// Sets the owner of the contract.
            pub fn set_owner(&mut self, new_owner: Address) {
                self.env
                    .call_contract(
                        self.address,
                        odra::CallDef::new(
                            odra::prelude::string::String::from("set_owner"),
                            true,
                            {
                                let mut __odra_named_args = odra::casper_types::RuntimeArgs::new();
                                if self.attached_value > odra::casper_types::U512::zero() {
                                    let _ = __odra_named_args.insert("amount", self.attached_value);
                                }
                                odra::args::EntrypointArgument::insert_runtime_arg(new_owner, "new_owner", &mut __odra_named_args);
                                __odra_named_args
                            }
                        )
                        .with_amount(self.attached_value),
                    )
            }

            /// Returns the name of the token.
            pub fn name(&self) -> String {
                self.env
                    .call_contract(
                        self.address,
                        odra::CallDef::new(
                            odra::prelude::string::String::from("name"),
                            false,
                            {
                                let mut __odra_named_args = odra::casper_types::RuntimeArgs::new();
                                if self.attached_value > odra::casper_types::U512::zero() {
                                    let _ = __odra_named_args.insert("amount", self.attached_value);
                                }
                                __odra_named_args
                            }
                        )
                        .with_amount(self.attached_value),
                    )
            }

            /// Delegated. See `self.metadata.symbol()` for details.
            pub fn symbol(&self) -> String {
                self.env
                    .call_contract(
                        self.address,
                        odra::CallDef::new(
                            odra::prelude::string::String::from("symbol"),
                            false,
                            {
                                let mut __odra_named_args = odra::casper_types::RuntimeArgs::new();
                                if self.attached_value > odra::casper_types::U512::zero() {
                                    let _ = __odra_named_args.insert("amount", self.attached_value);
                                }
                                __odra_named_args
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
    };
    let actual = RefItem::try_from(&module).unwrap();
    test_utils::assert_eq(actual, expected);
}

#[test]
fn contract_ref_invalid_delegate() {
    // A `delegate!` that does not parse is rejected when the module is read, before any
    // item is built from it.
    assert!(test_utils::mock::module_invalid_delegation().is_err());
}
