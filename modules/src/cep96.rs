#![allow(unused_variables, missing_docs)]

mod storage;

use odra::prelude::*;

use crate::cep96::storage::{
    Cep96DescriptionStorage, Cep96IconUriStorage, Cep96NameStorage, Cep96ProjectUriStorage
};
pub trait Cep96ContractMetadata {
    /// Contract's human-readable name.
    fn contract_name(&self) -> Option<String>;

    /// Brief description of the contract.
    fn contract_description(&self) -> Option<String>;

    /// URI pointing to the contract's icon image.
    fn contract_icon_uri(&self) -> Option<String>;

    /// URI pointing to the project's website or documentation.
    fn contract_project_uri(&self) -> Option<String>;
}

#[odra::module]
pub struct Cep96 {
    pub contract_name: SubModule<Cep96NameStorage>,
    pub contract_description: SubModule<Cep96DescriptionStorage>,
    pub contract_icon_uri: SubModule<Cep96IconUriStorage>,
    pub contract_project_uri: SubModule<Cep96ProjectUriStorage>
}

#[odra::module]
impl Cep96ContractMetadata for Cep96 {
    fn contract_name(&self) -> Option<String> {
        self.contract_name.get()
    }

    fn contract_description(&self) -> Option<String> {
        self.contract_description.get()
    }

    fn contract_icon_uri(&self) -> Option<String> {
        self.contract_icon_uri.get()
    }

    fn contract_project_uri(&self) -> Option<String> {
        self.contract_project_uri.get()
    }
}

impl Cep96 {
    pub fn init(
        &self,
        name: Option<String>,
        description: Option<String>,
        icon_uri: Option<String>,
        project_uri: Option<String>
    ) {
        if let Some(name) = name {
            self.contract_name.set(name);
        }
        if let Some(description) = description {
            self.contract_description.set(description);
        }
        if let Some(icon_uri) = icon_uri {
            self.contract_icon_uri.set(icon_uri);
        }
        if let Some(project_uri) = project_uri {
            self.contract_project_uri.set(project_uri);
        }
    }
}
