use odra_types::{
    bytesrepr::{FromBytes, ToBytes},
    event::Event,
    Address, CLTyped, CLValue, ExecutionError, U512,
};

pub fn self_address() -> Address {
    unimplemented!()
}

pub fn caller() -> Address {
    unimplemented!()
}

pub fn set_var<T: CLTyped + ToBytes>(_: &str, _: T) {
    unimplemented!()
}

pub fn get_var(_: &str) -> Option<CLValue> {
    unimplemented!()
}

pub fn set_dict_value<K: ToBytes, V: ToBytes + FromBytes + CLTyped>(_: &str, _: &K, _: V) {
    unimplemented!()
}

pub fn get_dict_value<K: ToBytes>(_: &str, _: &K) -> Option<CLValue> {
    unimplemented!()
}

pub fn emit_event<T>(_: &T)
where
    T: ToBytes + Event,
{
    unimplemented!()
}

pub fn revert<E>(_: E) -> !
where
    E: Into<ExecutionError>,
{
    unimplemented!()
}

pub fn get_block_time() -> u64 {
    unimplemented!()
}

pub fn attached_value() -> U512 {
    unimplemented!()
}

pub fn one_token() -> U512 {
    unimplemented!()
}

pub fn transfer_tokens(_: Address, _: U512) -> bool {
    unimplemented!()
}

pub fn self_balance() -> U512 {
    unimplemented!()
}
