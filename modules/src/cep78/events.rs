use odra::{prelude::*, Address};

#[odra::event]
pub struct Mint {
    recipient: Address,
    token_id: String,
    data: String
}

impl Mint {
    pub fn new(recipient: Address, token_id: String, data: String) -> Self {
        Self {
            recipient,
            token_id,
            data
        }
    }
}

#[odra::event]
pub struct Burn {
    owner: Address,
    token_id: String,
    burner: Address
}

impl Burn {
    pub fn new(owner: Address, token_id: String, burner: Address) -> Self {
        Self {
            owner,
            token_id,
            burner
        }
    }
}

#[odra::event]
pub struct Approval {
    owner: Address,
    spender: Address,
    token_id: String
}

impl Approval {
    pub fn new(owner: Address, spender: Address, token_id: String) -> Self {
        Self {
            owner,
            spender,
            token_id
        }
    }
}

#[odra::event]
pub struct ApprovalRevoked {
    owner: Address,
    token_id: String
}

impl ApprovalRevoked {
    pub fn new(owner: Address, token_id: String) -> Self {
        Self { owner, token_id }
    }
}

#[odra::event]
pub struct ApprovalForAll {
    owner: Address,
    operator: Address
}

impl ApprovalForAll {
    pub fn new(owner: Address, operator: Address) -> Self {
        Self { owner, operator }
    }
}

#[odra::event]
pub struct RevokedForAll {
    owner: Address,
    operator: Address
}

impl RevokedForAll {
    pub fn new(owner: Address, operator: Address) -> Self {
        Self { owner, operator }
    }
}

#[odra::event]
pub struct Transfer {
    owner: Address,
    spender: Option<Address>,
    recipient: Address,
    token_id: String
}

impl Transfer {
    pub fn new(
        owner: Address,
        spender: Option<Address>,
        recipient: Address,
        token_id: String
    ) -> Self {
        Self {
            owner,
            spender,
            recipient,
            token_id
        }
    }
}

#[odra::event]
pub struct MetadataUpdated {
    token_id: String,
    data: String
}

impl MetadataUpdated {
    pub fn new(token_id: String, data: String) -> Self {
        Self { token_id, data }
    }
}

#[odra::event]
pub struct VariablesSet {}

impl VariablesSet {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }
}

#[odra::event]
pub struct Migration {}

impl Migration {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }
}
