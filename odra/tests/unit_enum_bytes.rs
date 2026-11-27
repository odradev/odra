//! An `#[odra::odra_type]` unit enum is one byte on the wire, its discriminant, and reads back
//! as the same variant up to 255.
#![cfg(not(target_arch = "wasm32"))]
#![allow(missing_docs)]

use odra::casper_types::bytesrepr::{FromBytes, ToBytes};

#[odra::odra_type]
pub enum Edge {
    Max = 255,
    Zero = 0,
    One
}

#[test]
fn unit_enum_roundtrip_up_to_255() {
    for (variant, byte) in [(Edge::Max, 255u8), (Edge::Zero, 0), (Edge::One, 1)] {
        let bytes = variant.to_bytes().unwrap();
        assert_eq!(bytes, vec![byte]);
        assert_eq!(Edge::from_bytes(&bytes).unwrap(), (variant, &[][..]));
    }
}
