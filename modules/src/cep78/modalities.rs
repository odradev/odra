use core::default;

use odra::prelude::*;
use super::error::CEP78Error;

#[repr(u8)]
#[derive(PartialEq, Eq, Clone)]
pub enum WhitelistMode {
    Unlocked = 0,
    Locked = 1,
}

impl TryFrom<u8> for WhitelistMode {
    type Error = CEP78Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(WhitelistMode::Unlocked),
            1 => Ok(WhitelistMode::Locked),
            _ => Err(CEP78Error::InvalidWhitelistMode),
        }
    }
}

#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum NFTHolderMode {
    Accounts = 0,
    Contracts = 1,
    Mixed = 2,
}

impl TryFrom<u8> for NFTHolderMode {
    type Error = CEP78Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(NFTHolderMode::Accounts),
            1 => Ok(NFTHolderMode::Contracts),
            2 => Ok(NFTHolderMode::Mixed),
            _ => Err(CEP78Error::InvalidHolderMode),
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
#[repr(u8)]
pub enum MintingMode {
    /// The ability to mint NFTs is restricted to the installing account only.
    Installer = 0,
    /// The ability to mint NFTs is not restricted.
    Public = 1,
    /// The ability to mint NFTs is restricted by an ACL.
    Acl = 2,
}

impl TryFrom<u8> for MintingMode {
    type Error = CEP78Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MintingMode::Installer),
            1 => Ok(MintingMode::Public),
            2 => Ok(MintingMode::Acl),
            _ => Err(CEP78Error::InvalidMintingMode),
        }
    }
}

#[repr(u8)]
#[derive(Default, Clone)]
pub enum NFTKind {
    /// The NFT represents a real-world physical
    /// like a house.
    #[default]
    Physical = 0,
    /// The NFT represents a digital asset like a unique
    /// JPEG or digital art.
    Digital = 1,
    /// The NFT is the virtual representation
    /// of a physical notion, e.g a patent
    /// or copyright.
    Virtual = 2,
}

impl TryFrom<u8> for NFTKind {
    type Error = CEP78Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(NFTKind::Physical),
            1 => Ok(NFTKind::Digital),
            2 => Ok(NFTKind::Virtual),
            _ => Err(CEP78Error::InvalidNftKind),
        }
    }
}

pub type MetadataRequirement = BTreeMap<NFTMetadataKind, Requirement>;

#[odra::odra_type]
#[repr(u8)]
pub enum Requirement {
    Required = 0,
    Optional = 1,
    Unneeded = 2,
}

impl TryFrom<u8> for Requirement {
    type Error = CEP78Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Requirement::Required),
            1 => Ok(Requirement::Optional),
            2 => Ok(Requirement::Unneeded),
            _ => Err(CEP78Error::InvalidRequirement),
        }
    }
}

#[repr(u8)]
#[derive(Default, PartialOrd, Ord)]
#[odra::odra_type]
pub enum NFTMetadataKind {
    #[default]
    CEP78 = 0,
    NFT721 = 1,
    Raw = 2,
    CustomValidated = 3,
}

impl TryFrom<u8> for NFTMetadataKind {
    type Error = CEP78Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(NFTMetadataKind::CEP78),
            1 => Ok(NFTMetadataKind::NFT721),
            2 => Ok(NFTMetadataKind::Raw),
            3 => Ok(NFTMetadataKind::CustomValidated),
            _ => Err(CEP78Error::InvalidNFTMetadataKind),
        }
    }
}

#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Default)]
pub enum OwnershipMode {
    /// The minter owns it and can never transfer it.
    #[default]
    Minter = 0,
    /// The minter assigns it to an address and can never be transferred.
    Assigned = 1,
    /// The NFT can be transferred even to an recipient that does not exist.
    Transferable = 2,
}

impl TryFrom<u8> for OwnershipMode {
    type Error = CEP78Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(OwnershipMode::Minter),
            1 => Ok(OwnershipMode::Assigned),
            2 => Ok(OwnershipMode::Transferable),
            _ => Err(CEP78Error::InvalidOwnershipMode),
        }
    }
}

#[repr(u8)]
#[odra::odra_type]
pub enum NFTIdentifierMode {
    Ordinal = 0,
    Hash = 1,
}

impl TryFrom<u8> for NFTIdentifierMode {
    type Error = CEP78Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(NFTIdentifierMode::Ordinal),
            1 => Ok(NFTIdentifierMode::Hash),
            _ => Err(CEP78Error::InvalidIdentifierMode),
        }
    }
}

#[repr(u8)]
#[derive(Default)]
#[odra::odra_type]
pub enum MetadataMutability {
    #[default]
    Immutable = 0,
    Mutable = 1,
}

impl TryFrom<u8> for MetadataMutability {
    type Error = CEP78Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MetadataMutability::Immutable),
            1 => Ok(MetadataMutability::Mutable),
            _ => Err(CEP78Error::InvalidMetadataMutability),
        }
    }
}

#[odra::odra_type]
pub enum TokenIdentifier {
    Index(u64),
    Hash(String),
}

impl TokenIdentifier {
    pub fn new_index(index: u64) -> Self {
        TokenIdentifier::Index(index)
    }

    pub fn new_hash(hash: String) -> Self {
        TokenIdentifier::Hash(hash)
    }

    pub fn get_index(&self) -> Option<u64> {
        if let Self::Index(index) = self {
            return Some(*index);
        }
        None
    }

    pub fn get_hash(self) -> Option<String> {
        if let Self::Hash(hash) = self {
            return Some(hash);
        }
        None
    }

    pub fn get_dictionary_item_key(&self) -> String {
        match self {
            TokenIdentifier::Index(token_index) => token_index.to_string(),
            TokenIdentifier::Hash(hash) => hash.clone(),
        }
    }
}

impl ToString for TokenIdentifier {
    fn to_string(&self) -> String {
        match self {
            TokenIdentifier::Index(index) => index.to_string(),
            TokenIdentifier::Hash(hash) => hash.to_string(),
        }
    }
}


#[repr(u8)]
#[odra::odra_type]
pub enum BurnMode {
    Burnable = 0,
    NonBurnable = 1,
}

impl TryFrom<u8> for BurnMode {
    type Error = CEP78Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(BurnMode::Burnable),
            1 => Ok(BurnMode::NonBurnable),
            _ => Err(CEP78Error::InvalidBurnMode),
        }
    }
}

#[repr(u8)]
#[derive(Clone, PartialEq, Eq)]
pub enum OwnerReverseLookupMode {
    NoLookUp = 0,
    Complete = 1,
    TransfersOnly = 2,
}

impl TryFrom<u8> for OwnerReverseLookupMode {
    type Error = CEP78Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(OwnerReverseLookupMode::NoLookUp),
            1 => Ok(OwnerReverseLookupMode::Complete),
            2 => Ok(OwnerReverseLookupMode::TransfersOnly),
            _ => Err(CEP78Error::InvalidReportingMode),
        }
    }
}

#[repr(u8)]
pub enum NamedKeyConventionMode {
    DerivedFromCollectionName = 0,
    V1_0Standard = 1,
    V1_0Custom = 2,
}

impl TryFrom<u8> for NamedKeyConventionMode {
    type Error = CEP78Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(NamedKeyConventionMode::DerivedFromCollectionName),
            1 => Ok(NamedKeyConventionMode::V1_0Standard),
            2 => Ok(NamedKeyConventionMode::V1_0Custom),
            _ => Err(CEP78Error::InvalidNamedKeyConvention),
        }
    }
}

#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy, Default)]
#[allow(clippy::upper_case_acronyms)]
pub enum EventsMode {
    NoEvents = 0,
    CEP47 = 1,
    #[default]
    CES = 2,
}

impl TryFrom<u8> for EventsMode {
    type Error = CEP78Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(EventsMode::NoEvents),
            1 => Ok(EventsMode::CEP47),
            2 => Ok(EventsMode::CES),
            _ => Err(CEP78Error::InvalidEventsMode),
        }
    }
}

#[repr(u8)]
#[non_exhaustive]
#[odra::odra_type]
pub enum TransferFilterContractResult {
    DenyTransfer = 0,
    ProceedTransfer,
}

impl From<u8> for TransferFilterContractResult {
    fn from(value: u8) -> Self {
        match value {
            0 => TransferFilterContractResult::DenyTransfer,
            _ => TransferFilterContractResult::ProceedTransfer,
        }
    }
}
