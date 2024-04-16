use odra::prelude::*;
use odra::Var;

use super::modalities::BurnMode;
use super::modalities::EventsMode;
use super::modalities::MintingMode;
use super::modalities::NFTHolderMode;
use super::modalities::NFTKind;
use super::modalities::OwnershipMode;

#[odra::module]
pub struct Settings {
    allow_minting: Var<bool>,
    minting_mode: Var<MintingMode>,
    ownership_mode: Var<OwnershipMode>,
    nft_kind: Var<NFTKind>,
    holder_mode: Var<NFTHolderMode>,
    burn_mode: Var<BurnMode>,
    events_mode: Var<EventsMode>
}

impl Settings {
    pub fn init(
        &mut self,
        allow_minting: bool,
        minting_mode: MintingMode,
        ownership_mode: OwnershipMode,
        nft_kind: NFTKind,
        holder_mode: NFTHolderMode,
        burn_mode: BurnMode,
        events_mode: EventsMode
    ) {
        self.allow_minting.set(allow_minting);
        self.minting_mode.set(minting_mode);
        self.ownership_mode.set(ownership_mode);
        self.nft_kind.set(nft_kind);
        self.holder_mode.set(holder_mode);
        self.burn_mode.set(burn_mode);
        self.events_mode.set(events_mode);
    }

    #[inline]
    pub fn allow_minting(&self) -> bool {
        self.allow_minting.get_or_default()
    }

    #[inline]
    pub fn set_allow_minting(&mut self, value: bool) {
        self.allow_minting.set(value)
    }

    #[inline]
    pub fn events_mode(&self) -> EventsMode {
        self.events_mode.get_or_default()
    }

    #[inline]
    pub fn burn_mode(&self) -> BurnMode {
        self.burn_mode.get_or_default()
    }

    #[inline]
    pub fn ownership_mode(&self) -> OwnershipMode {
        self.ownership_mode.get_or_default()
    }

    #[inline]
    pub fn minting_mode(&self) -> MintingMode {
        self.minting_mode.get_or_default()
    }
}
