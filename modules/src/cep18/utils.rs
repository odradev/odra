/// Security badge that can be assigned to an account to grant it certain permissions.
#[odra::odra_type]
pub enum SecurityBadge {
    /// The account is an admin.
    Admin = 0,
    /// The account is a minter.
    Minter = 1,
    /// The account has no special permissions.
    None = 2
}

impl SecurityBadge {
    /// Returns true if the account has admin permissions.
    pub fn can_admin(&self) -> bool {
        matches!(self, SecurityBadge::Admin)
    }

    /// Returns true if the account has minter or admin permissions.
    pub fn can_mint(&self) -> bool {
        matches!(self, SecurityBadge::Minter | SecurityBadge::Admin)
    }
}
/// Modality of the CEP-18 contract.
#[derive(Default)]
#[odra::odra_type]
pub enum Cep18Modality {
    /// No modailities are set.
    #[default]
    None = 0,
    /// The contract can mint and burn tokens.
    MintAndBurn = 1
}

impl Cep18Modality {
    /// Returns true if the mint and burn functionality is enabled.
    pub fn mint_and_burn_enabled(&self) -> bool {
        matches!(self, Cep18Modality::MintAndBurn)
    }
}

// implement conversion from modality into u8
impl From<Cep18Modality> for u8 {
    fn from(modality: Cep18Modality) -> u8 {
        modality as u8
    }
}
