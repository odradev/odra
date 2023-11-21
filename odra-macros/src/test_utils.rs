use quote::{quote, ToTokens};

use crate::ir::ModuleIR;

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
        }
    };
    ModuleIR::try_from(&module).unwrap()
}

pub fn assert_eq<A: ToTokens, B: ToTokens>(a: A, b: B) {
    fn parse<T: ToTokens>(e: T) -> String {
        let e = e.to_token_stream().to_string();
        let e = syn::parse_file(&e).unwrap();
        prettyplease::unparse(&e)
    }
    pretty_assertions::assert_eq!(parse(a), parse(b));
}