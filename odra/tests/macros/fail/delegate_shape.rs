//! A `delegate!` block is `to self.<field> { fn name(&self, ..) -> Ret; .. }`.
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
pub struct NoTo {
    owned: SubModule<Owned>
}

#[odra::module]
impl NoTo {
    delegate! {
        self.owned {
            fn get_owner(&self) -> Address;
        }
    }
}

#[odra::module]
pub struct NotAField {
    owned: SubModule<Owned>
}

#[odra::module]
impl NotAField {
    delegate! {
        to owned {
            fn get_owner(&self) -> Address;
        }
    }
}

#[odra::module]
pub struct Nested {
    owned: SubModule<Owned>
}

#[odra::module]
impl Nested {
    delegate! {
        to self.owned.owner {
            fn get_owner(&self) -> Address;
        }
    }
}

#[odra::module]
pub struct WithBody {
    owned: SubModule<Owned>
}

#[odra::module]
impl WithBody {
    delegate! {
        to self.owned {
            fn get_owner(&self) -> Address {
                self.owned.get_owner()
            }
        }
    }
}

#[odra::module]
pub struct OtherMacro {
    owned: SubModule<Owned>
}

#[odra::module]
impl OtherMacro {
    forward! {
        to self.owned {
            fn get_owner(&self) -> Address;
        }
    }
}

fn main() {}
