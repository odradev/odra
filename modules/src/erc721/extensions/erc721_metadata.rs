//! Erc721 metadata.

use crate::erc721::extensions::erc721_metadata::errors::Error::{
    BaseUriNotSet, NameNotSet, SymbolNotSet
};
use odra::prelude::*;
use odra::UnwrapOrRevert;
use odra::{module::Module, Var};

/// The ERC721 Metadata interface as defined in the standard.
pub trait Erc721Metadata {
    /// Returns the token collection name.
    fn name(&self) -> String;
    /// Returns the token collection symbol.
    fn symbol(&self) -> String;
    /// Returns the base URI for the token collection.
    fn base_uri(&self) -> String;
}

#[odra::module]
pub struct Erc721MetadataExtension {
    name: Var<String>,
    symbol: Var<String>,
    base_uri: Var<String>
}

impl Erc721Metadata for Erc721MetadataExtension {
    fn name(&self) -> String {
        self.name
            .get()
            .unwrap_or_revert_with(&self.env(), NameNotSet)
    }

    fn symbol(&self) -> String {
        self.symbol
            .get()
            .unwrap_or_revert_with(&self.env(), SymbolNotSet)
    }

    fn base_uri(&self) -> String {
        self.base_uri
            .get()
            .unwrap_or_revert_with(&self.env(), BaseUriNotSet)
    }
}

impl Erc721MetadataExtension {
    pub fn init(&mut self, name: String, symbol: String, base_uri: String) {
        self.name.set(name);
        self.symbol.set(symbol);
        self.base_uri.set(base_uri);
    }
}

pub mod errors {
    use odra::OdraError;

    #[derive(OdraError)]
    pub enum Error {
        NameNotSet = 31_000,
        SymbolNotSet = 31_001,
        BaseUriNotSet = 31_002
    }
}
