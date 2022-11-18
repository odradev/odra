use odra::types::{arithmetic::ArithmeticsError, Balance, U256};

#[cfg(feature = "mock-vm")]
pub fn balance_to_u256(balance: Balance) -> Result<U256, ArithmeticsError> {
    Ok(balance)
}

#[cfg(feature = "mock-vm")]
pub fn u256_to_balance(value: U256) -> Balance {
    value
}

#[cfg(feature = "casper")]
pub fn balance_to_u256(balance: Balance) -> Result<U256, ArithmeticsError> {
    if exceeds_u256(balance) {
        return Err(ArithmeticsError::AdditionOverflow);
    }

    let src = balance.0;
    let mut words = [0u64; 4];
    words.copy_from_slice(&src[0..4]);

    Ok(U256(words))
}

#[cfg(feature = "casper")]
pub fn u256_to_balance(value: U256) -> Balance {
    let mut bytes = [0u8; 32];
    value.to_little_endian(&mut bytes);
    Balance::from_little_endian(&bytes)
}

#[cfg(feature = "casper")]
fn exceeds_u256(balance: Balance) -> bool {
    match balance.0.len() {
        0..=3 => false,
        _ => balance.0[4] > 0
    }
}
