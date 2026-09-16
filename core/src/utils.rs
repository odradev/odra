//! General purpose utilities.

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

/// Encodes the storage path (the chain of module field indices) into bytes.
///
/// Two encoding modes exist to support backward compatibility:
///
/// **Legacy encoding** (default, when all path indices fit in 4 bits):
/// Packs indices into a `u32` using 4-bit left shifts, identical to the original
/// `(parent << 4) + child` formula. Produces 4 big-endian bytes. This ensures
/// deployed contracts with ≤15 fields per module get the same storage keys.
///
/// **Path encoding** (indices > 15):
/// Emits `[0xFF, path_len, path[0], ..., path[n]]`. The `0xFF` prefix cannot
/// collide with legacy keys (whose first byte never exceeds `0x0F`). The
/// `path_len` byte makes the boundary with appended mapping data unambiguous,
/// preventing collisions between e.g. a `Var` at a deeper path and a `Mapping`
/// at a shallower path with matching key bytes.
///
/// **Why `path_len` is necessary — collision example:**
///
/// Consider two fields whose path bytes and mapping data concatenate identically:
/// - Field A: `Var` at path `[3, 5]` (depth 2), no mapping data.
/// - Field B: `Mapping` at path `[3]` (depth 1), mapping key serializes to `[5]`.
///
/// The final hash input is `index_bytes ++ mapping_data`.
///
/// Without `path_len` (hypothetical `[0xFF, path..., mapping_data...]`):
/// - A → `[0xFF, 3, 5]`, B → `[0xFF, 3] ++ [5]` = `[0xFF, 3, 5]` — **collision!**
///
/// With `path_len` (actual `[0xFF, path_len, path..., mapping_data...]`):
/// - A → `[0xFF, 2, 3, 5]`, B → `[0xFF, 1, 3] ++ [5]` = `[0xFF, 1, 3, 5]` — **distinct.**
pub fn storage_index_bytes(path: &[u8]) -> Vec<u8> {
    // Legacy: pack indices into u32 via 4-bit shifts (e.g. path [3, 15] → 0x3F).
    // Only used when all indices fit in a nibble, preserving old storage keys.
    if path.iter().all(|&idx| idx <= 15) {
        let index: u32 = path.iter().fold(0u32, |acc, &idx| (acc << 4) + idx as u32);
        index.to_be_bytes().to_vec()
    } else {
        // Path encoding: [0xFF, len, idx_0, idx_1, ...]. Used for fields 16+.
        let mut bytes = Vec::with_capacity(2 + path.len());
        bytes.push(0xFF);
        bytes.push(path.len() as u8);
        bytes.extend_from_slice(path);
        bytes
    }
}

/// Returns the bytes that are hashed to produce a storage key of a module element.
///
/// `path` is the chain of field indices from the contract root to the element and
/// `mapping_data` is the concatenation of serialized [`Mapping`](crate::mapping::Mapping)
/// keys used along the way. The storage key is the hex-encoded `blake2b` hash of the result.
pub fn storage_key_preimage(path: &[u8], mapping_data: &[u8]) -> Vec<u8> {
    let mut key = storage_index_bytes(path);
    key.extend_from_slice(mapping_data);
    key
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
