//! A delegated function is written like a trait method, without a visibility.
use odra::prelude::*;

#[odra::module]
pub struct Owned {
    owner: Var<Address>
}

#[odra::module]
impl Owned {
    pub fn get_owner(&self) -> Address {
        self.owner.get_or_revert_with(ExecutionError::UnwrapError)
    }
}

#[odra::module]
pub struct Contract {
    owned: SubModule<Owned>
}

#[odra::module]
impl Contract {
    delegate! {
        to self.owned {
            pub fn get_owner(&self) -> Address;
        }
    }
}

fn main() {}
