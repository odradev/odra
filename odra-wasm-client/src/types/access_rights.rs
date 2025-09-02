use casper_types::AccessRights as _AccessRights;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct AccessRights(_AccessRights);

#[wasm_bindgen]
impl AccessRights {
    #[wasm_bindgen(js_name = "NONE")]
    pub fn none() -> u8 {
        _AccessRights::NONE.bits()
    }

    #[wasm_bindgen(js_name = "READ")]
    pub fn read() -> u8 {
        _AccessRights::READ.bits()
    }

    #[wasm_bindgen(js_name = "WRITE")]
    pub fn write() -> u8 {
        _AccessRights::WRITE.bits()
    }

    #[wasm_bindgen(js_name = "ADD")]
    pub fn add() -> u8 {
        _AccessRights::ADD.bits()
    }

    #[wasm_bindgen(js_name = "READ_ADD")]
    pub fn read_add() -> u8 {
        _AccessRights::READ_ADD.bits()
    }

    #[wasm_bindgen(js_name = "READ_WRITE")]
    pub fn read_write() -> u8 {
        _AccessRights::READ_WRITE.bits()
    }

    #[wasm_bindgen(js_name = "ADD_WRITE")]
    pub fn add_write() -> u8 {
        _AccessRights::ADD_WRITE.bits()
    }

    #[wasm_bindgen(js_name = "READ_ADD_WRITE")]
    pub fn read_add_write() -> u8 {
        _AccessRights::READ_ADD_WRITE.bits()
    }

    // Utility method to create AccessRights with u8
    #[wasm_bindgen(constructor)]
    pub fn new(access_rights: u8) -> Result<AccessRights, JsError> {
        match _AccessRights::from_bits(access_rights) {
            Some(rights) => Ok(AccessRights(rights)),
            None => Err(JsError::new("Invalid URef access rights"))
        }
    }

    #[wasm_bindgen]
    pub fn from_bits(read: bool, write: bool, add: bool) -> Self {
        let mut access_rights = _AccessRights::NONE;
        if read {
            access_rights |= _AccessRights::READ;
        }
        if write {
            access_rights |= _AccessRights::WRITE;
        }
        if add {
            access_rights |= _AccessRights::ADD;
        }
        AccessRights(access_rights)
    }

    #[wasm_bindgen]
    // Utility method to check if the READ flag is set.
    pub fn is_readable(&self) -> bool {
        self.0.is_readable()
    }

    #[wasm_bindgen]
    // Utility method to check if the WRITE flag is set.
    pub fn is_writeable(&self) -> bool {
        self.0.is_writeable()
    }

    #[wasm_bindgen]
    // Utility method to check if the ADD flag is set.
    pub fn is_addable(&self) -> bool {
        self.0.is_addable()
    }

    #[wasm_bindgen]
    // Utility method to check if no flags are set.
    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

impl From<AccessRights> for _AccessRights {
    fn from(access_rights: AccessRights) -> Self {
        access_rights.0
    }
}

impl From<_AccessRights> for AccessRights {
    fn from(access_rights: _AccessRights) -> Self {
        AccessRights(access_rights)
    }
}
