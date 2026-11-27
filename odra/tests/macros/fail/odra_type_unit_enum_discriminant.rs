//! A unit enum is serialized as one byte, so its discriminants are from 0 to 255: `Big::A` would
//! be written as `44`, the same byte as `Big::B`. `Implicit::B` is 256.
#[odra::odra_type]
pub enum Big {
    A = 300,
    B = 44
}

#[odra::odra_type]
pub enum Implicit {
    A = 255,
    B
}

fn main() {}
