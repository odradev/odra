use alloc::string::String;
use odra::types::U256;

pub trait Erc1155MetadataURI {
    fn uri(&self, token_id: U256) -> String;
}

#[odra::module]
pub struct Erc1155MetadataURIExtension {}
