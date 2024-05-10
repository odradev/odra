use odra::{prelude::*, Address};

#[odra::event]
pub struct Mint {
    recipient: Address,
    token_id: String,
    data: String
}

#[odra::event]
pub struct Burn {
    owner: Address,
    token_id: String,
    burner: Address
}

#[odra::event]
pub struct Approval {
    owner: Address,
    spender: Address,
    token_id: String
}

#[odra::event]
pub struct ApprovalRevoked {
    owner: Address,
    token_id: String
}

#[odra::event]
pub struct ApprovalForAll {
    owner: Address,
    operator: Address
}

#[odra::event]
pub struct RevokedForAll {
    owner: Address,
    operator: Address
}

#[odra::event]
pub struct Transfer {
    owner: Address,
    spender: Option<Address>,
    recipient: Address,
    token_id: String
}

#[odra::event]
pub struct MetadataUpdated {
    token_id: String,
    data: String
}

#[odra::event]
pub struct VariablesSet {}

#[odra::event]
pub struct Migration {}
