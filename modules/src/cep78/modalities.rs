use super::error::CEP78Error;
use odra::prelude::*;

/// The WhitelistMode dictates if the ACL whitelist restricting access to
/// the mint entry point can be updated.
#[repr(u8)]
#[odra::odra_type]
#[derive(Default)]
pub enum WhitelistMode {
    /// The ACL whitelist is unlocked and can be updated via the `set_variables` endpoint.
    #[default]
    Unlocked = 0,
    /// The ACL whitelist is locked and cannot be updated further.
    Locked = 1
}

impl TryFrom<u8> for WhitelistMode {
    type Error = CEP78Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(WhitelistMode::Unlocked),
            1 => Ok(WhitelistMode::Locked),
            _ => Err(CEP78Error::InvalidWhitelistMode)
        }
    }
}

/// The modality dictates which entities on a Casper network can own and mint NFTs.
///
/// If the NFTHolderMode is set to Contracts a ContractHash whitelist must be provided.
/// This whitelist dictates which Contracts are allowed to mint NFTs in the restricted
/// Installer minting mode.
///
/// This modality is an optional installation parameter and will default to the Mixed mode
/// if not provided. However, this mode cannot be changed once the contract has been installed.
#[repr(u8)]
#[odra::odra_type]
#[derive(Copy, Default)]
pub enum NFTHolderMode {
    /// Only Accounts can own and mint NFTs.
    Accounts = 0,
    /// Only Contracts can own and mint NFTs.
    Contracts = 1,
    /// Both Accounts and Contracts can own and mint NFTs.
    #[default]
    Mixed = 2
}

impl TryFrom<u8> for NFTHolderMode {
    type Error = CEP78Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(NFTHolderMode::Accounts),
            1 => Ok(NFTHolderMode::Contracts),
            2 => Ok(NFTHolderMode::Mixed),
            _ => Err(CEP78Error::InvalidHolderMode)
        }
    }
}

/// The minting mode governs the behavior of contract when minting new tokens.
///
/// This modality is an optional installation parameter and will default
/// to the `Installer` mode if not provided. However, this mode cannot be changed
/// once the contract has been installed.
#[odra::odra_type]
#[repr(u8)]
#[derive(Default)]
pub enum MintingMode {
    /// The ability to mint NFTs is restricted to the installing account only.
    #[default]
    Installer = 0,
    /// The ability to mint NFTs is not restricted.
    Public = 1,
    /// The ability to mint NFTs is restricted by an ACL.
    Acl = 2
}

impl TryFrom<u8> for MintingMode {
    type Error = CEP78Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MintingMode::Installer),
            1 => Ok(MintingMode::Public),
            2 => Ok(MintingMode::Acl),
            _ => Err(CEP78Error::InvalidMintingMode)
        }
    }
}

#[repr(u8)]
#[odra::odra_type]
#[derive(Default)]
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
    Virtual = 2
}

impl TryFrom<u8> for NFTKind {
    type Error = CEP78Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(NFTKind::Physical),
            1 => Ok(NFTKind::Digital),
            2 => Ok(NFTKind::Virtual),
            _ => Err(CEP78Error::InvalidNftKind)
        }
    }
}

pub type MetadataRequirement = BTreeMap<NFTMetadataKind, Requirement>;

#[odra::odra_type]
#[repr(u8)]
pub enum Requirement {
    Required = 0,
    Optional = 1,
    Unneeded = 2
}

impl TryFrom<u8> for Requirement {
    type Error = CEP78Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Requirement::Required),
            1 => Ok(Requirement::Optional),
            2 => Ok(Requirement::Unneeded),
            _ => Err(CEP78Error::InvalidRequirement)
        }
    }
}

/// This modality dictates the schema for the metadata for NFTs minted
/// by a given instance of an NFT contract.
#[repr(u8)]
#[derive(Default, PartialOrd, Ord)]
#[odra::odra_type]
pub enum NFTMetadataKind {
    /// NFTs must have valid metadata conforming to the CEP-78 schema.
    #[default]
    CEP78 = 0,
    /// NFTs  must have valid metadata conforming to the NFT-721 metadata schema.
    NFT721 = 1,
    /// Metadata validation will not occur and raw strings can be passed to
    /// `token_metadata` runtime argument as part of the call to mint entrypoint.
    Raw = 2,
    /// Custom schema provided at the time of install will be used when validating
    /// the metadata as part of the call to mint entrypoint.
    CustomValidated = 3
}

impl TryFrom<u8> for NFTMetadataKind {
    type Error = CEP78Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(NFTMetadataKind::CEP78),
            1 => Ok(NFTMetadataKind::NFT721),
            2 => Ok(NFTMetadataKind::Raw),
            3 => Ok(NFTMetadataKind::CustomValidated),
            _ => Err(CEP78Error::InvalidNFTMetadataKind)
        }
    }
}

/// This modality specifies the behavior regarding ownership of NFTs and whether
/// the owner of the NFT can change over the contract's lifetime.
///
/// Ownership mode is a required installation parameter and cannot be changed
/// once the contract has been installed.
#[repr(u8)]
#[odra::odra_type]
#[derive(Default, PartialOrd, Ord, Copy)]
pub enum OwnershipMode {
    /// The minter owns it and can never transfer it.
    #[default]
    Minter = 0,
    /// The minter assigns it to an address and can never be transferred.
    Assigned = 1,
    /// The NFT can be transferred even to an recipient that does not exist.
    Transferable = 2
}

impl TryFrom<u8> for OwnershipMode {
    type Error = CEP78Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(OwnershipMode::Minter),
            1 => Ok(OwnershipMode::Assigned),
            2 => Ok(OwnershipMode::Transferable),
            _ => Err(CEP78Error::InvalidOwnershipMode)
        }
    }
}

/// The identifier mode governs the primary identifier for NFTs minted
/// for a given instance on an installed contract.
///
/// Since the default primary identifier in the `Hash` mode is custom or derived by
/// hashing over the metadata, making it a content-addressed identifier,
/// the metadata for the minted NFT cannot be updated after the mint.
///
/// Attempting to install the contract with the [MetadataMutability] modality set to
/// `Mutable` in the `Hash` identifier mode will raise an error.
///
/// This modality is a required installation parameter and cannot be changed
/// once the contract has been installed.
#[repr(u8)]
#[odra::odra_type]
#[derive(Default, PartialOrd, Ord, Copy)]
pub enum NFTIdentifierMode {
    /// NFTs minted in this modality are identified by a u64 value.
    /// This value is determined by the number of NFTs minted by
    /// the contract at the time the NFT is minted.
    #[default]
    Ordinal = 0,
    /// NFTs minted in this modality are identified by an optional custom
    /// string identifier or by default a base16 encoded representation of
    /// the blake2b hash of the metadata provided at the time of mint.
    Hash = 1
}

impl TryFrom<u8> for NFTIdentifierMode {
    type Error = CEP78Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(NFTIdentifierMode::Ordinal),
            1 => Ok(NFTIdentifierMode::Hash),
            _ => Err(CEP78Error::InvalidIdentifierMode)
        }
    }
}

/// The metadata mutability mode governs the behavior around updates to a given NFTs metadata.
///
/// The Mutable option cannot be used in conjunction with the Hash modality for the NFT identifier;
/// attempting to install the contract with this configuration raises
/// [super::error::CEP78Error::InvalidMetadataMutability] error.
///
/// This modality is a required installation parameter and cannot be changed
/// once the contract has been installed.
#[repr(u8)]
#[derive(Default, PartialOrd, Ord, Copy)]
#[odra::odra_type]
pub enum MetadataMutability {
    /// Metadata for NFTs minted in this mode cannot be updated once the NFT has been minted.
    #[default]
    Immutable = 0,
    /// Metadata for NFTs minted in this mode can update the metadata via the `set_token_metadata` entrypoint.
    Mutable = 1
}

impl TryFrom<u8> for MetadataMutability {
    type Error = CEP78Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MetadataMutability::Immutable),
            1 => Ok(MetadataMutability::Mutable),
            _ => Err(CEP78Error::InvalidMetadataMutability)
        }
    }
}

#[odra::odra_type]
pub enum TokenIdentifier {
    Index(u64),
    Hash(String)
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

    pub fn get_hash(&self) -> Option<String> {
        if let Self::Hash(hash) = self {
            return Some(hash.to_owned());
        }
        None
    }

    pub fn get_dictionary_item_key(&self) -> String {
        match self {
            TokenIdentifier::Index(token_index) => token_index.to_string(),
            TokenIdentifier::Hash(hash) => hash.clone()
        }
    }
}

impl ToString for TokenIdentifier {
    fn to_string(&self) -> String {
        match self {
            TokenIdentifier::Index(index) => index.to_string(),
            TokenIdentifier::Hash(hash) => hash.to_string()
        }
    }
}

/// The modality dictates whether tokens minted by a given instance of
/// an NFT contract can be burnt.
#[repr(u8)]
#[odra::odra_type]
#[derive(Default)]
pub enum BurnMode {
    /// Minted tokens can be burnt.
    #[default]
    Burnable = 0,
    /// Minted tokens cannot be burnt.
    NonBurnable = 1
}

impl TryFrom<u8> for BurnMode {
    type Error = CEP78Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(BurnMode::Burnable),
            1 => Ok(BurnMode::NonBurnable),
            _ => Err(CEP78Error::InvalidBurnMode)
        }
    }
}

/// This modality is set at install and determines if a given contract instance
/// writes necessary data to allow reverse lookup by owner in addition to by ID.
///
/// This modality provides the following options:
///
/// `NoLookup`: The reporting and receipt functionality is not supported.
/// In this option, the contract instance does not maintain a reverse lookup
/// database of ownership and therefore has more predictable gas costs and greater
/// scaling.
/// `Complete`: The reporting and receipt functionality is supported. Token
/// ownership will be tracked by the contract instance using the system described
/// [here](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/dev/docs/reverse-lookup.md#owner-reverse-lookup-functionality).
/// `TransfersOnly`: The reporting and receipt functionality is supported like
/// `Complete`. However, it does not begin tracking until the first transfer.
/// This modality is for use cases where the majority of NFTs are owned by
/// a private minter and only NFT's that have been transferred benefit from
/// reverse lookup tracking. Token ownership will also be tracked by the contract
/// instance using the system described [here](https://github.com/casper-ecosystem/cep-78-enhanced-nft/blob/dev/docs/reverse-lookup.md#owner-reverse-lookup-functionality).
///
/// Additionally, when set to Complete, causes a receipt to be returned by the mint
/// or transfer entrypoints, which the caller can store in their account or contract
/// context for later reference.
///
/// Further, two special entrypoints are enabled in Complete mode. First,
/// `register_owner` which when called will allocate the necessary tracking
/// record for the imputed entity. This allows isolation of the one time gas cost
/// to do this per owner, which is convenient for accounting purposes. Second,
/// updated_receipts, which allows an owner of one or more NFTs held by the contract
/// instance to attain up to date receipt information for the NFTs they currently own.
#[repr(u8)]
#[derive(Default, PartialOrd, Ord, Copy)]
#[odra::odra_type]
pub enum OwnerReverseLookupMode {
    /// The reporting and receipt functionality is not supported.
    #[default]
    NoLookUp = 0,
    /// The reporting and receipt functionality is supported.
    Complete = 1,
    /// The reporting and receipt functionality is supported, but the tracking
    /// does not start until the first transfer.
    TransfersOnly = 2
}

impl TryFrom<u8> for OwnerReverseLookupMode {
    type Error = CEP78Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(OwnerReverseLookupMode::NoLookUp),
            1 => Ok(OwnerReverseLookupMode::Complete),
            2 => Ok(OwnerReverseLookupMode::TransfersOnly),
            _ => Err(CEP78Error::InvalidReportingMode)
        }
    }
}

/// The `EventsMode` modality determines how the installed instance of CEP-78
/// will handle the recording of events that occur from interacting with
/// the contract.
///
/// Odra does not allow to set the `CEP47` event schema.
#[repr(u8)]
#[odra::odra_type]
#[derive(Copy, Default)]
#[allow(clippy::upper_case_acronyms)]
pub enum EventsMode {
    /// Signals the contract to not record events at all. This is the default mode.
    #[default]
    NoEvents = 0,
    /// Signals the contract to record events using the Casper Event Standard.
    CES = 2
}

impl TryFrom<u8> for EventsMode {
    type Error = CEP78Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(EventsMode::NoEvents),
            2 => Ok(EventsMode::CES),
            _ => Err(CEP78Error::InvalidEventsMode)
        }
    }
}

/// The transfer filter modality, if enabled, specifies a contract package hash
/// pointing to a contract that will be called when the transfer method is
/// invoked on the contract. CEP-78 will call the `can_transfer` method on the
/// specified callback contract, which is expected to return a value of
/// `TransferFilterContractResult`, represented as a u8.
#[repr(u8)]
#[non_exhaustive]
#[odra::odra_type]
pub enum TransferFilterContractResult {
    /// Blocks the transfer regardless of the outcome of other checks
    DenyTransfer = 0,
    /// Allows the transfer to proceed if other checks also pass
    ProceedTransfer
}

impl From<u8> for TransferFilterContractResult {
    fn from(value: u8) -> Self {
        match value {
            0 => TransferFilterContractResult::DenyTransfer,
            _ => TransferFilterContractResult::ProceedTransfer
        }
    }
}
