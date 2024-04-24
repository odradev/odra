use odra::prelude::*;
use odra::Var;

use super::modalities::{BurnMode, EventsMode, MintingMode, NFTHolderMode, NFTKind, OwnershipMode};

#[odra::module]
pub struct Settings {
    allow_minting: Var<bool>,
    minting_mode: Var<MintingMode>,
    ownership_mode: Var<OwnershipMode>,
    nft_kind: Var<NFTKind>,
    holder_mode: Var<NFTHolderMode>,
    burn_mode: Var<BurnMode>,
    events_mode: Var<EventsMode>,
    operator_burn_mode: Var<bool>
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

    #[inline]
    pub fn holder_mode(&self) -> NFTHolderMode {
        self.holder_mode.get_or_default()
    }

    #[inline]
    pub fn operator_burn_mode(&self) -> bool {
        self.operator_burn_mode.get_or_default()
    }

    #[inline]
    pub fn set_operator_burn_mode(&mut self, value: bool) {
        self.operator_burn_mode.set(value)
    }
}
