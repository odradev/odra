use odra::prelude::*;

#[odra::odra_type]
pub enum SecurityBadge {
    Admin = 0,
    Minter = 1,
    None = 2
}
