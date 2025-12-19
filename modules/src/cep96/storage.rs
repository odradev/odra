use odra::prelude::*;

const KEY_CONTRACT_NAME: &str = "contract_name";
const KEY_CONTRACT_DESCRIPTION: &str = "contract_description";
const KEY_CONTRACT_ICON_URI: &str = "contract_icon_uri";
const KEY_CONTRACT_PROJECT_URI: &str = "contract_project_uri";

#[odra::module]
pub struct Cep96NameStorage;
impl Cep96NameStorage {
    pub fn set(&self, value: String) {
        if self.get().is_none() {
            self.env().set_named_value(KEY_CONTRACT_NAME, value);
        }
    }

    pub fn get(&self) -> Option<String> {
        self.env().get_named_value(KEY_CONTRACT_NAME)
    }
}

#[odra::module]
pub struct Cep96DescriptionStorage;
impl Cep96DescriptionStorage {
    pub fn set(&self, value: String) {
        if self.get().is_none() {
            self.env().set_named_value(KEY_CONTRACT_DESCRIPTION, value);
        }
    }

    pub fn get(&self) -> Option<String> {
        self.env().get_named_value(KEY_CONTRACT_DESCRIPTION)
    }
}

#[odra::module]
pub struct Cep96IconUriStorage;
impl Cep96IconUriStorage {
    pub fn set(&self, value: String) {
        if self.get().is_none() {
            self.env().set_named_value(KEY_CONTRACT_ICON_URI, value);
        }
    }

    pub fn get(&self) -> Option<String> {
        self.env().get_named_value(KEY_CONTRACT_ICON_URI)
    }
}

#[odra::module]
pub struct Cep96ProjectUriStorage;
impl Cep96ProjectUriStorage {
    pub fn set(&self, value: String) {
        if self.get().is_none() {
            self.env().set_named_value(KEY_CONTRACT_PROJECT_URI, value);
        }
    }

    pub fn get(&self) -> Option<String> {
        self.env().get_named_value(KEY_CONTRACT_PROJECT_URI)
    }
}
