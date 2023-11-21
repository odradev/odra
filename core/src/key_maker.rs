use crate::prelude::*;

static TABLE: &[u8] = b"0123456789abcdef";

#[inline]
fn hex(byte: u8) -> u8 {
    TABLE[byte as usize]
}

pub fn u32_to_hex(value: u32) -> [u8; 8] {
    let mut result = [0u8; 8];
    let bytes = value.to_be_bytes();
    for i in 0..4 {
        result[2 * i] = hex(bytes[i] >> 4);
        result[2 * i + 1] = hex(bytes[i] & 0xf);
    }
    result
}

pub fn bytes_to_hex(bytes: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    for byte in bytes {
        result.push(hex(byte >> 4));
        result.push(hex(byte & 0xf));
    }
    result
}

#[cfg(test)]
mod tests {
    use crate::key_maker::u32_to_hex;

    #[test]
    fn test_u32_to_hex() {
        assert_eq!(&u32_to_hex(0), b"00000000");
        assert_eq!(&u32_to_hex(255), b"000000ff");
    }
}
