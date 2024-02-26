use quote::ToTokens;

pub mod mock {
    use crate::ir::{ModuleImplIR, ModuleStructIR, TypeIR};
    use quote::quote;

    pub fn module_impl() -> ModuleImplIR {
        let module = quote! {
            impl Erc20 {
                /// Initializes the contract with the given parameters.
                pub fn init(&mut self, total_supply: Option<U256>) {
                    if let Some(total_supply) = total_supply {
                        self.total_supply.set(total_supply);
                        self.balances.set(self.env().caller(), total_supply);
                    }
                }

                /// Returns the total supply of the token.
                pub fn total_supply(&self) -> U256 {
                    self.total_supply.get_or_default()
                }

                /// Pay to mint.
                #[odra(payable)]
                pub fn pay_to_mint(&mut self) {
                    let attached_value = self.env().attached_value();
                    self.total_supply
                        .set(self.total_supply() + U256::from(attached_value.as_u64()));
                }

                /// Approve.
                #[odra(non_reentrant)]
                pub fn approve(&mut self, to: &Address, amount: &U256) {
                    self.env.emit_event(Approval {
                        owner: self.env.caller(),
                        spender: to,
                        value: amount
                    });
                }

                /// Airdrops the given amount to the given addresses.
                pub fn airdrop(to: &[Address], amount: &U256) {
                }

                fn private_function() {

                }
            }
        };

        let attr = quote!();
        ModuleImplIR::try_from((&attr, &module)).unwrap()
    }

    pub fn module_trait_impl() -> ModuleImplIR {
        let module = quote! {
            impl IErc20 for Erc20 {
                fn total_supply(&self) -> U256 {
                    self.total_supply.get_or_default()
                }

                #[odra(payable)]
                fn pay_to_mint(&mut self) {
                    let attached_value = self.env().attached_value();
                    self.total_supply
                        .set(self.total_supply() + U256::from(attached_value.as_u64()));
                }
            }
        };

        let attr = quote!();
        ModuleImplIR::try_from((&attr, &module)).unwrap()
    }

    pub fn module_delegation() -> ModuleImplIR {
        let module = quote! {
            impl Erc20 {
                /// Returns the total supply of the token.
                pub fn total_supply(&self) -> U256 {
                    self.total_supply.get_or_default()
                }

                delegate! {
                    to self.ownable {
                        fn get_owner(&self) -> Address;
                        fn set_owner(&mut self, new_owner: Address);
                    }

                    to self.metadata {
                        fn name(&self) -> String;
                        fn symbol(&self) -> String;
                    }
                }
            }
        };

        let attr = quote!();
        ModuleImplIR::try_from((&attr, &module)).unwrap()
    }

    pub fn module_definition() -> ModuleStructIR {
        let module = quote!(
            pub struct CounterPack {
                counter0: SubModule<Counter>,
                counter1: SubModule<Counter>,
                counter2: SubModule<Counter>,
                counters: Var<u32>,
                counters_map: Mapping<u8, Counter>
            }
        );
        let attr = quote!(events = [OnTransfer, OnApprove]);
        ModuleStructIR::try_from((&attr, &module)).unwrap()
    }

    pub fn empty_module_definition() -> ModuleStructIR {
        let module = quote!(
            pub struct CounterPack;
        );
        let attr = quote!();
        ModuleStructIR::try_from((&attr, &module)).unwrap()
    }

    pub fn custom_struct() -> TypeIR {
        let ty = quote!(
            struct MyType {
                a: String,
                b: u32
            }
        );
        TypeIR::try_from(&ty).unwrap()
    }

    pub fn custom_enum() -> TypeIR {
        let ty = quote!(
            enum MyType {
                A,
                B
            }
        );
        TypeIR::try_from(&ty).unwrap()
    }

    pub fn ext_contract() -> ModuleImplIR {
        let ext = quote!(
            pub trait Token {
                fn balance_of(&self, owner: Address) -> U256;
            }
        );
        let attr = quote!();
        ModuleImplIR::try_from((&attr, &ext)).unwrap()
    }
}

#[track_caller]
pub fn assert_eq<A: ToTokens, B: ToTokens>(a: A, b: B) {
    fn parse<T: ToTokens>(e: T) -> String {
        let e = e.to_token_stream().to_string();
        let e = syn::parse_file(&e).unwrap();
        prettyplease::unparse(&e)
    }
    pretty_assertions::assert_eq!(parse(a), parse(b));
}
