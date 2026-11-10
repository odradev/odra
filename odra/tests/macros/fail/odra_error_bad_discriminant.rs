//! An error code is a `u16`, so a discriminant is a literal in that range.
#[odra::odra_error]
pub enum Error {
    TooBig = 70000,
    Negative = -1,
    Computed = 1 + 1,
    Fine = 3
}

fn main() {}
