//! Dapp metadata.
use odra::prelude::*;

/// Descriptive information about a dapp.
#[odra::odra_type]
#[derive(Default)]
pub struct DappMetadata {
    /// The name of the dapp.
    pub name: String,
    /// A short description of the dapp.
    pub description: String,
    /// The URL of the dapp's website.
    pub website_url: String,
    /// The URL of the dapp's icon.
    pub icon_url: String
}
