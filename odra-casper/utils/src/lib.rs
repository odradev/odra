use casper_types::bytesrepr::FromBytes;
use casper_types::{CLTyped, CLValueError, StoredValue};

pub fn stored_value_into_t<T: CLTyped + FromBytes>(
    stored_value: StoredValue
) -> Result<T, CLValueError> {
    stored_value.into_cl_value().unwrap().into_t()
}
