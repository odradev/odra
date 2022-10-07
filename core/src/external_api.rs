#[cfg(feature = "wasm-test")]
pub mod test_env;

#[cfg(feature = "wasm")]
pub mod contract_env;

// This mock here is required because when loading a code of a module
// in new contract repository, ContractEnv is required, but we semantically
// doesn't make sense for `wasm-test` feature.
#[cfg(feature = "wasm-test")]
pub mod contract_env {
    use odra_types::{
        bytesrepr::{FromBytes, ToBytes},
        event::Event,
        Address, CLTyped, CLValue, ExecutionError, RuntimeArgs, U512
    };

    pub struct ContractEnv;

    impl ContractEnv {
        pub fn self_address() -> Address {
            unimplemented!()
        }

        pub fn caller() -> Address {
            unimplemented!()
        }

        pub fn set_var<T: CLTyped + ToBytes>(_key: &str, _value: T) {
            unimplemented!()
        }

        pub fn get_var(_key: &str) -> Option<CLValue> {
            unimplemented!()
        }

        pub fn set_dict_value<K: ToBytes, V: ToBytes + FromBytes + CLTyped>(
            _dict: &str,
            _key: &K,
            _value: V,
        ) {
            unimplemented!()
        }

        pub fn get_dict_value<K: ToBytes>(_dict: &str, _key: &K) -> Option<CLValue> {
            unimplemented!()
        }

        pub fn emit_event<T>(_event: &T)
        where
            T: ToBytes + Event,
        {
            unimplemented!()
        }

        pub fn call_contract(
            _address: &Address,
            _entrypoint: &str,
            _args: &RuntimeArgs,
        ) -> Vec<u8> {
            unimplemented!()
        }

        pub fn revert<E>(_error: E) -> !
        where
            E: Into<ExecutionError>,
        {
            unimplemented!()
        }

        pub fn print(_message: &str) {
            unimplemented!()
        }

        pub fn attached_value() -> U512 {
            unimplemented!()
        }

        pub fn one_token() -> U512 {
            unimplemented!()
        }

        pub fn with_tokens(amount: U512) {
            unimplemented!()
        }

        pub fn token_balance(address: Address) -> U512 {
            unimplemented!()
        }

        pub fn transfer_tokens(to: Address, amount: U512) {
            unimplemented!()
        }

        pub fn self_balance() -> U512 {
            unimplemented!()
        }
    }
}
