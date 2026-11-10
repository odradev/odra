//! An error is only its code, so a variant carries no data.
#[odra::odra_error]
pub enum Error {
    Overflow(u32),
    Underflow { by: u32 },
    Empty
}

fn main() {}
