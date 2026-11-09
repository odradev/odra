use super::HostRefItem;
use crate::test_utils;
use quote::quote;

#[test]
fn host_ref() {
    let module = test_utils::mock::module_impl();
    let expected = quote! {
        /// [Erc20] Host Ref.
        pub struct Erc20HostRef {
            address: odra::prelude::Address,
            env: odra::host::HostEnv,
            attached_value: odra::casper_types::U512
        }

        impl odra::host::HostRef for Erc20HostRef {
            fn new(address: odra::prelude::Address, env: odra::host::HostEnv) -> Self {
                env.register_contract_ref(
                    address,
                    <Self as odra::contract_def::HasIdent>::ident(),
                    <Self as odra::host::EntryPointsCallerProvider>::entry_points_caller
                );
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

        impl Erc20HostRef {
            /// Initializes the contract with the given parameters.
            pub fn init(&mut self, total_supply: Option<U256>) {
                self.try_init(total_supply).unwrap()
            }

            /// Upgrades the contract with the given parameters.
            pub fn upgrade(&mut self, total_supply: Option<U256>) {
                self.try_upgrade(total_supply).unwrap()
            }

            /// Returns the total supply of the token.
            pub fn total_supply(&self) -> U256 {
                self.try_total_supply().unwrap()
            }

            /// Pay to mint.
            pub fn pay_to_mint(&mut self) {
                self.try_pay_to_mint().unwrap()
            }

            /// Approve.
            pub fn approve(&mut self, to: &Address, amount: &U256, msg: Maybe<String>) {
                self.try_approve(to, amount, msg).unwrap()
            }

            /// Airdrops the given amount to the given addresses.
            pub fn airdrop(&self, to: &[Address], amount: &U256) {
                self.try_airdrop(to, amount).unwrap()
            }

            /// Swaps the given amount to the given addresses.
            pub fn swap(&mut self, to: Address, amount: U256) -> U256 {
                self.try_swap(to, amount).unwrap()
            }
        }

        impl Erc20HostRef {
            /// Initializes the contract with the given parameters.
            /// Does not fail in case of error, returns `odra::OdraResult` instead.
            pub fn try_init(&mut self, total_supply: Option<U256>) -> OdraResult<()> {
                self.env
                    .call_contract(
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
                            },
                        )
                        .with_amount(self.attached_value),
                    )
            }

            /// Upgrades the contract with the given parameters.
            /// Does not fail in case of error, returns `odra::OdraResult` instead.
            pub fn try_upgrade(&mut self, total_supply: Option<U256>) -> OdraResult<()> {
                self.env
                    .call_contract(
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
                            },
                        )
                        .with_amount(self.attached_value),
                    )
            }

            /// Returns the total supply of the token.
            /// Does not fail in case of error, returns `odra::OdraResult` instead.
            pub fn try_total_supply(&self) -> OdraResult<U256> {
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
                    ).with_amount(self.attached_value),
                )
            }
            /// Pay to mint.
            /// Does not fail in case of error, returns `odra::OdraResult` instead.
            pub fn try_pay_to_mint(&mut self) -> OdraResult<()> {
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
                                },
                            )
                            .with_amount(self.attached_value),
                    )
            }
            /// Approve.
            /// Does not fail in case of error, returns `odra::OdraResult` instead.
            pub fn try_approve(
                &mut self,
                to: &Address,
                amount: &U256,
                msg: Maybe<String>,
            ) -> OdraResult<()> {
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
                                },
                            )
                            .with_amount(self.attached_value),
                    )
            }
            /// Airdrops the given amount to the given addresses.
            /// Does not fail in case of error, returns `odra::OdraResult` instead.
            pub fn try_airdrop(&self, to: &[Address], amount: &U256) -> OdraResult<()> {
                self.env.call_contract(
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
                    ).with_amount(self.attached_value),
                )
            }

            /// Swaps the given amount to the given addresses.
            /// Does not fail in case of error, returns `odra::OdraResult` instead.
            pub fn try_swap(&mut self, to: Address, amount: U256) -> OdraResult<U256> {
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
                                odra::args::EntrypointArgument::insert_runtime_arg(
                                    to,
                                    "to",
                                    &mut __odra_named_args,
                                );
                                odra::args::EntrypointArgument::insert_runtime_arg(
                                    amount,
                                    "amount",
                                    &mut __odra_named_args,
                                );
                                __odra_named_args
                            },
                        )
                        .with_amount(self.attached_value),
                    )
            }
        }

        impl Erc20HostRef {
            /// Swaps the given amount to the given addresses.
            /// Ignores the result of the call.
            pub fn swap_no_ret(&mut self, to: Address, amount: U256) {
                self.try_swap_no_ret(to, amount).unwrap()
            }

            /// Swaps the given amount to the given addresses.
            /// Ignores the result of the call.
            pub fn try_swap_no_ret(&mut self, to: Address, amount: U256) -> OdraResult<()> {
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
                                odra::args::EntrypointArgument::insert_runtime_arg(
                                    to,
                                    "to",
                                    &mut __odra_named_args,
                                );
                                odra::args::EntrypointArgument::insert_runtime_arg(
                                    amount,
                                    "amount",
                                    &mut __odra_named_args,
                                );
                                __odra_named_args
                            },
                        )
                        .with_amount(self.attached_value),
                    )
            }
        }
    };
    let actual = HostRefItem::try_from(&module).unwrap();
    test_utils::assert_eq(actual, expected);
}

#[test]
fn host_trait_impl_ref() {
    let module = test_utils::mock::module_trait_impl();
    let expected = quote! {
        /// [Erc20] Host Ref.
        pub struct Erc20HostRef {
            address: odra::prelude::Address,
            env: odra::host::HostEnv,
            attached_value: odra::casper_types::U512
        }

        impl odra::host::HostRef for Erc20HostRef {
            fn new(address: odra::prelude::Address, env: odra::host::HostEnv) -> Self {
                env.register_contract_ref(
                    address,
                    <Self as odra::contract_def::HasIdent>::ident(),
                    <Self as odra::host::EntryPointsCallerProvider>::entry_points_caller
                );
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

        impl IErc20 for Erc20HostRef {
            fn total_supply(&self) -> U256 {
                self.try_total_supply().unwrap()
            }
            fn set_total_supply(&mut self) -> U256 {
                self.try_set_total_supply().unwrap()
            }
            fn pay_to_mint(&mut self) {
                self.try_pay_to_mint().unwrap()
            }
        }

        impl Erc20HostRef {
            /// Does not fail in case of error, returns `odra::OdraResult` instead.
            pub fn try_total_supply(&self) -> OdraResult<U256> {
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
                    ).with_amount(self.attached_value),
                )
            }

            /// Does not fail in case of error, returns `odra::OdraResult` instead.
            pub fn try_set_total_supply(&mut self) -> OdraResult<U256> {
                self.env.call_contract(
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
                    ).with_amount(self.attached_value),
                )
            }

            /// Does not fail in case of error, returns `odra::OdraResult` instead.
            pub fn try_pay_to_mint(&mut self) -> OdraResult<()>  {
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
                                },
                            )
                            .with_amount(self.attached_value),
                    )
            }
        }

        impl Erc20HostRef {
            /// Ignores the result of the call.
            pub fn set_total_supply_no_ret(&mut self) {
                self.try_set_total_supply_no_ret().unwrap()
            }

            /// Ignores the result of the call.
            pub fn try_set_total_supply_no_ret(&mut self) -> OdraResult<()> {
                self.env.call_contract(
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
                    ).with_amount(self.attached_value),
                )
            }
        }
    };
    let actual = HostRefItem::try_from(&module).unwrap();
    test_utils::assert_eq(actual, expected);
}

#[test]
fn host_ref_delegation() {
    let module = test_utils::mock::module_delegation();
    let expected = quote! {
        /// [Erc20] Host Ref.
        pub struct Erc20HostRef {
            address: odra::prelude::Address,
            env: odra::host::HostEnv,
            attached_value: odra::casper_types::U512
        }

        impl odra::host::HostRef for Erc20HostRef {
            fn new(address: odra::prelude::Address, env: odra::host::HostEnv) -> Self {
                env.register_contract_ref(
                    address,
                    <Self as odra::contract_def::HasIdent>::ident(),
                    <Self as odra::host::EntryPointsCallerProvider>::entry_points_caller
                );
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

        impl Erc20HostRef {
            /// Returns the total supply of the token.
            pub fn total_supply(&self) -> U256 {
                self.try_total_supply().unwrap()
            }

            /// Returns the owner of the contract.
            pub fn get_owner(&self) -> Address {
                self.try_get_owner().unwrap()
            }

            /// Sets the owner of the contract.
            pub fn set_owner(&mut self, new_owner: Address) {
                self.try_set_owner(new_owner).unwrap()
            }

            /// Returns the name of the token.
            pub fn name(&self) -> String {
                self.try_name().unwrap()
            }

            /// Delegated. See `self.metadata.symbol()` for details.
            pub fn symbol(&self) -> String {
                self.try_symbol().unwrap()
             }
        }

        impl Erc20HostRef {
            /// Returns the total supply of the token.
            /// Does not fail in case of error, returns `odra::OdraResult` instead.
            pub fn try_total_supply(&self) -> OdraResult<U256> {
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
                    ).with_amount(self.attached_value),
                )
            }

            /// Returns the owner of the contract.
            /// Does not fail in case of error, returns `odra::OdraResult` instead.
            pub fn try_get_owner(&self) -> OdraResult<Address> {
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
                                },
                            )
                            .with_amount(self.attached_value),
                    )
            }

            /// Sets the owner of the contract.
            /// Does not fail in case of error, returns `odra::OdraResult` instead.
            pub fn try_set_owner(&mut self, new_owner: Address) -> OdraResult<()> {
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
                                },
                            )
                            .with_amount(self.attached_value),
                    )
            }

            /// Returns the name of the token.
            /// Does not fail in case of error, returns `odra::OdraResult` instead.
            pub fn try_name(&self) -> OdraResult<String> {
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
                                },
                            )
                            .with_amount(self.attached_value),
                    )
            }

            /// Delegated. See `self.metadata.symbol()` for details.
            /// Does not fail in case of error, returns `odra::OdraResult` instead.
            pub fn try_symbol(&self) -> OdraResult<String> {
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
                                },
                            )
                            .with_amount(self.attached_value),
                    )
            }

        }
    };
    let actual = HostRefItem::try_from(&module).unwrap();
    test_utils::assert_eq(actual, expected);
}
