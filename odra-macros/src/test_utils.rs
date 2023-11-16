use quote::{quote, ToTokens};

use crate::ir::ModuleIR;

pub fn mock_module() -> ModuleIR {
    let module = quote! {
        impl Erc20 {
            fn init(env: &mut Env, name: String) -> Void {
                env.set_value(String::from("name"), name)
            }

            fn name(env: &Env) -> OdraResult<String> {
                Ok(env.get_value(String::from("name"))?.unwrap())
            }

            fn balance_of(env: &Env, address: Address) -> OdraResult<U256> {
                let key = (String::from("balances"), address);
                Ok(env.get_value(key)?.unwrap())
            }

            fn mint(env: &mut Env, address: Address, amount: U256) -> Void {
                let key = (String::from("balances"), address);
                let current_balance = Self::balance_of(env, address.clone())?;
                env.set_value(key, current_balance + amount)
            }
        }
    };
    ModuleIR::new(&module)
}

pub fn assert_eq<A: ToTokens, B: ToTokens>(a: A, b: B) {
    fn parse<T: ToTokens>(e: T) -> String {
        let e = e.to_token_stream().to_string();
        let e = syn::parse_file(&e).unwrap();
        prettyplease::unparse(&e)
    }
    pretty_assertions::assert_eq!(parse(a), parse(b));
}