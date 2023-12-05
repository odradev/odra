use quote::{quote, ToTokens};

use crate::ir::{ModuleIR, StructIR};

pub fn mock_module() -> ModuleIR {
    let module = quote! {
        impl Erc20 {
            pub fn init(&mut self, total_supply: Option<U256>) {
                if let Some(total_supply) = total_supply {
                    self.total_supply.set(total_supply);
                    self.balances.set(self.env().caller(), total_supply);
                }
            }

            pub fn total_supply(&self) -> U256 {
                self.total_supply.get_or_default()
            }

            #[odra(payable)]
            pub fn pay_to_mint(&mut self) {
                let attached_value = self.env().attached_value();
                self.total_supply
                    .set(self.total_supply() + U256::from(attached_value.as_u64()));
            }

            #[odra(non_reentrant)]
            pub fn approve(&mut self, to: Address, amount: U256) {
                self.env.emit_event(Approval {
                    owner: self.env.caller(),
                    spender: to,
                    value: amount
                });
            }
        }
    };

    let attr = quote!();
    ModuleIR::try_from((&attr, &module)).unwrap()
}

pub fn mock_module_definition() -> StructIR {
    let module = quote!(
        pub struct CounterPack {
            env: Rc<ContractEnv>,
            counter0: ModuleWrapper<Counter>,
            counter1: ModuleWrapper<Counter>,
            counter2: ModuleWrapper<Counter>,
            counters: Variable<u32>,
            counters_map: Mapping<u8, Counter>
        }
    );
    let attr = quote!(events = [OnTransfer, OnApprove]);
    StructIR::try_from((&attr, &module)).unwrap()
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
