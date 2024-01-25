use crate::casper_types::bytesrepr::{Bytes, FromBytes};
use crate::error::EventError;
use crate::prelude::*;
use casper_event_standard::casper_types::bytesrepr::ToBytes;

/// Serializes a value implementing the `ToBytes` trait into a `Bytes` object.
///
/// # Arguments
///
/// * `value` - The value to be serialized.
///
/// # Returns
///
/// Returns a `Bytes` object containing the serialized representation of the value.
///
/// # Panics
///
/// Panics if serialization fails.
pub fn serialize<T: ToBytes>(value: &T) -> Bytes {
    Bytes::from(value.to_bytes().expect("Couldn't serialize"))
}

/// Returns the name of the passed event.
///
/// # Arguments
///
/// * `bytes` - The byte slice containing the event.
///
/// # Returns
///
/// Returns the name of the event as a `String`.
///
/// # Errors
///
/// Returns an `EventError` if the name extraction fails or the event name is unexpected.
pub(crate) fn extract_event_name(bytes: &[u8]) -> Result<String, EventError> {
    let name: String = FromBytes::from_bytes(bytes)
        .map_err(|_| EventError::CouldntExtractName)?
        .0;
    name.strip_prefix("event_")
        .map(|s| s.to_string())
        .ok_or(EventError::UnexpectedType(name))
}

/// Calculates the absolute position of the event. Accepts both positive and negative indexing.
///
/// # Examples
///
/// ```
/// # use odra_core::utils::event_absolute_position;
///
/// assert_eq!(event_absolute_position(10, 0), Some(0));
/// assert_eq!(event_absolute_position(10, -1), Some(9));
/// assert_eq!(event_absolute_position(10, 10), None);
/// ```
pub fn event_absolute_position(len: u32, index: i32) -> Option<u32> {
    if index.is_negative() {
        let abs_idx = index.wrapping_abs();
        if abs_idx > len as i32 {
            return None;
        }
        Some(
            len.checked_sub(abs_idx as u32)
                .expect("Checked sub failed, it shouldn't happen")
        )
    } else {
        if index >= len as i32 {
            return None;
        }
        Some(index as u32)
    }
}

pub static KEY_DELIMITER: &str = "#";

static TABLE: &[u8] = b"0123456789abcdef";

#[inline]
fn hex(byte: u8) -> u8 {
    TABLE[byte as usize]
}

/// Converts the hexadecimal values from the source byte slice into a more readable form,
/// representing each byte in hexadecimal form, in the destination byte slice.
///
/// * It iterates over the source slice `src` and the destination slice `dst` concurrently.
/// * For each byte in the source, it calculates the hexadecimal representation.
/// * It splits the byte into two nibbles (4-bit groups): the higher order 4 bits and the lower order 4 bits.
/// * It converts each nibble into its corresponding hexadecimal representation.
/// * It stores the two hexadecimal representations in two consecutive slots of the destination slice.
///
/// # Example
///
/// ```
/// # use odra_core::utils::hex_to_slice;
///
/// let mut dst = vec![0; 10];
/// let src = [255, 254, 253, 252, 251];
/// hex_to_slice(&src, &mut dst);
/// assert_eq!(&dst, &[102, 102, 102, 101, 102, 100, 102, 99, 102, 98]);
/// ```
pub fn hex_to_slice(src: &[u8], dst: &mut [u8]) {
    for (byte, slots) in src.iter().zip(dst.chunks_exact_mut(2)) {
        slots[0] = hex((*byte >> 4) & 0xf);
        slots[1] = hex(*byte & 0xf);
    }
}

/// Joins two parts of a key with the [`KEY_DELIMITER`].
#[inline]
pub fn create_key(left: &str, right: &str) -> String {
    crate::prelude::format!("{}{}{}", left, KEY_DELIMITER, right)
}

#[cfg(test)]
mod tests {
    use super::event_absolute_position;

    #[test]
    fn event_absolute_position_works() {
        assert_eq!(event_absolute_position(0, 1), None);
        assert_eq!(event_absolute_position(10, 10), None);
        assert_eq!(event_absolute_position(10, -11), None);
        assert_eq!(event_absolute_position(10, 0), Some(0));
        assert_eq!(event_absolute_position(10, 1), Some(1));
        assert_eq!(event_absolute_position(10, -1), Some(9));
        assert_eq!(event_absolute_position(10, -2), Some(8));
        assert_eq!(event_absolute_position(10, -10), Some(0));
    }
}
