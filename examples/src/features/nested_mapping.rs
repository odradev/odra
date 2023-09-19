use odra::{map, prelude::string::String, types::casper_types::U256, Mapping, UnwrapOrRevert};

use crate::contracts::owned_token::OwnedToken;

#[odra::module]
pub struct NestedMapping {
    strings: Mapping<String, Mapping<u32, Mapping<String, String>>>,
    tokens: Mapping<String, Mapping<u32, Mapping<String, OwnedToken>>>
}

#[odra::module]
impl NestedMapping {
    pub fn set_string(&mut self, key1: String, key2: u32, key3: String, value: String) {
        map!(self.strings[key1][key2][key3] = value);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_token(
        &mut self,
        key1: String,
        key2: u32,
        key3: String,
        token_name: String,
        decimals: u8,
        symbol: String,
        initial_supply: &U256
    ) {
        self.tokens
            .get_instance(&key1)
            .get_instance(&key2)
            .get_instance(&key3)
            .init(token_name, symbol, decimals, initial_supply);
    }

    pub fn get_string_macro(&self, key1: String, key2: u32, key3: String) -> String {
        map!(self.strings[key1][key2][key3])
    }

    pub fn get_string_api(&self, key1: String, key2: u32, key3: String) -> String {
        let mapping = self.strings.get_instance(&key1).get_instance(&key2);
        mapping.get(&key3).unwrap_or_revert()
    }

    pub fn total_supply(&self, key1: String, key2: u32, key3: String) -> U256 {
        self.tokens
            .get_instance(&key1)
            .get_instance(&key2)
            .get_instance(&key3)
            .total_supply()
    }
}

#[cfg(test)]
mod test {
    use crate::features::nested_mapping::NestedMappingDeployer;
    use odra::prelude::string::String;

    #[test]
    fn nested_mapping_works() {
        // given a nested mapping contract
        let mut contract = NestedMappingDeployer::default();
        let (key1, key2, key3) = (String::from("a"), 1, String::from("b"));
        // when set a value
        let value = String::from("value");
        contract.set_string(key1.clone(), key2, key3.clone(), value.clone());
        // then the value can be retrieved using both get_string_macro and get_string_api
        assert_eq!(
            contract.get_string_macro(key1.clone(), key2, key3.clone()),
            value
        );
        assert_eq!(
            contract.get_string_api(key1.clone(), key2, key3.clone()),
            value
        );

        // when create a token
        let token_name = String::from("token");
        let decimals = 10;
        let symbol = String::from("SYM");
        let initial_supply = 100.into();
        contract.set_token(
            key1.clone(),
            key2,
            key3.clone(),
            token_name,
            decimals,
            symbol,
            &initial_supply
        );
        // then the total supply is set
        assert_eq!(contract.total_supply(key1, key2, key3), initial_supply);
    }
}
