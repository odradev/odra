use odra::{named_keys::single_value_storage, prelude::*};

use super::constants::*;
use super::error::CEP78Error;
use super::modalities::{BurnMode, EventsMode, MintingMode, NFTHolderMode, NFTKind, OwnershipMode};

single_value_storage!(Cep78AllowMinting, bool, ALLOW_MINTING);
single_value_storage!(
    Cep78MintingMode,
    MintingMode,
    MINTING_MODE,
    CEP78Error::MissingMintingMode
);
single_value_storage!(
    Cep78OwnershipMode,
    OwnershipMode,
    OWNERSHIP_MODE,
    CEP78Error::MissingOwnershipMode
);
single_value_storage!(Cep78NFTKind, NFTKind, NFT_KIND, CEP78Error::MissingNftKind);
single_value_storage!(
    Cep78HolderMode,
    NFTHolderMode,
    HOLDER_MODE,
    CEP78Error::MissingHolderMode
);
single_value_storage!(
    Cep78BurnMode,
    BurnMode,
    BURN_MODE,
    CEP78Error::MissingBurnMode
);
single_value_storage!(
    Cep78EventsMode,
    EventsMode,
    EVENTS_MODE,
    CEP78Error::MissingEventsMode
);
single_value_storage!(
    Cep78OperatorBurnMode,
    bool,
    OPERATOR_BURN_MODE,
    CEP78Error::MissingOperatorBurnMode
);

#[odra::module]
pub struct Settings {
    allow_minting: SubModule<Cep78AllowMinting>,
    minting_mode: SubModule<Cep78MintingMode>,
    ownership_mode: SubModule<Cep78OwnershipMode>,
    nft_kind: SubModule<Cep78NFTKind>,
    holder_mode: SubModule<Cep78HolderMode>,
    burn_mode: SubModule<Cep78BurnMode>,
    events_mode: SubModule<Cep78EventsMode>,
    operator_burn_mode: SubModule<Cep78OperatorBurnMode>
}

impl Settings {
    #[allow(clippy::too_many_arguments)]
    pub fn init(
        &mut self,
        allow_minting: bool,
        minting_mode: MintingMode,
        ownership_mode: OwnershipMode,
        nft_kind: NFTKind,
        holder_mode: NFTHolderMode,
        burn_mode: BurnMode,
        events_mode: EventsMode,
        operator_burn_mode: bool
    ) {
        self.allow_minting.set(allow_minting);
        self.minting_mode.set(minting_mode);
        self.ownership_mode.set(ownership_mode);
        self.nft_kind.set(nft_kind);
        self.holder_mode.set(holder_mode);
        self.burn_mode.set(burn_mode);
        self.events_mode.set(events_mode);
        self.operator_burn_mode.set(operator_burn_mode);
    }

    #[inline]
    pub fn allow_minting(&self) -> bool {
        self.allow_minting.get().unwrap_or_default()
    }

    #[inline]
    pub fn set_allow_minting(&mut self, value: bool) {
        self.allow_minting.set(value)
    }

    #[inline]
    pub fn events_mode(&self) -> EventsMode {
        self.events_mode.get()
    }

    #[inline]
    pub fn burn_mode(&self) -> BurnMode {
        self.burn_mode.get()
    }

    #[inline]
    pub fn ownership_mode(&self) -> OwnershipMode {
        self.ownership_mode.get()
    }

    #[inline]
    pub fn minting_mode(&self) -> MintingMode {
        self.minting_mode.get()
    }

    #[inline]
    pub fn holder_mode(&self) -> NFTHolderMode {
        self.holder_mode.get()
    }

    #[inline]
    pub fn operator_burn_mode(&self) -> bool {
        self.operator_burn_mode.get()
    }

    #[inline]
    pub fn nft_kind(&self) -> NFTKind {
        self.nft_kind.get()
    }

    #[inline]
    pub fn set_operator_burn_mode(&mut self, value: bool) {
        self.operator_burn_mode.set(value)
    }
}
