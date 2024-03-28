//! Erc721 metadata.

use crate::erc721::extensions::erc721_metadata::errors::Error;
use odra::prelude::*;
use odra::UnwrapOrRevert;
use odra::Var;

/// The ERC721 Metadata interface as defined in the standard.
pub trait Erc721Metadata {
    /// Returns the token collection name.
    fn name(&self) -> String;
    /// Returns the token collection symbol.
    fn symbol(&self) -> String;
    /// Returns the base URI for the token collection.
    fn base_uri(&self) -> String;
}

/// The ERC721 Metadata extension.
#[odra::module(errors = Error)]
pub struct Erc721MetadataExtension {
    name: Var<String>,
    symbol: Var<String>,
    base_uri: Var<String>
}

impl Erc721Metadata for Erc721MetadataExtension {
    fn name(&self) -> String {
        self.name
            .get()
            .unwrap_or_revert_with(&self.env(), Error::NameNotSet)
    }

    fn symbol(&self) -> String {
        self.symbol
            .get()
            .unwrap_or_revert_with(&self.env(), Error::SymbolNotSet)
    }

    fn base_uri(&self) -> String {
        self.base_uri
            .get()
            .unwrap_or_revert_with(&self.env(), Error::BaseUriNotSet)
    }
}

impl Erc721MetadataExtension {
    /// Initializes the ERC721 metadata extension.
    pub fn init(&mut self, name: String, symbol: String, base_uri: String) {
        self.name.set(name);
        self.symbol.set(symbol);
        self.base_uri.set(base_uri);
    }
}

/// Erc721Metadata-related errors.
pub mod errors {
    /// Possible errors in the context of Erc721 metadata.
    #[odra::odra_error]
    pub enum Error {
        /// The name is not set.
        NameNotSet = 31_000,
        /// The symbol is not set.
        SymbolNotSet = 31_001,
        /// The base URI is not set.
        BaseUriNotSet = 31_002
    }
}
