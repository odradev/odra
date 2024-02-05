use odra::casper_types::U256;
use odra::prelude::*;

/// An optional ERC1155MetadataExtension.
pub trait Erc1155MetadataURI {
    /// Returns the URI for token type `token_id`.
    fn uri(&self, token_id: U256) -> String;
}
