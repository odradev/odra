//! The JavaScript bindings.

use gloo_utils::format::JsValueSerdeExt;
use odra_schema::readable_fields;
use serde_json::Value;
use wasm_bindgen::prelude::*;

use crate::{reader, Error, Location};

/// Reads the storage of a deployed Odra contract, see the crate docs.
#[wasm_bindgen]
pub struct StorageReader(reader::StorageReader);

#[wasm_bindgen]
impl StorageReader {
    /// Creates a reader from the contract's schema file (the `*_schema.json` `odra-build`
    /// writes), given as a JSON string or an object.
    #[wasm_bindgen(constructor)]
    pub fn new(schema: JsValue) -> Result<StorageReader, JsError> {
        let json = match schema.as_string() {
            Some(json) => serde_json::from_str(&json),
            None => schema.into_serde::<Value>()
        }
        .map_err(|e| Error::InvalidSchema(e.to_string()))?;
        Ok(Self(reader::StorageReader::from_json(&json)?))
    }

    /// The name of the contract.
    #[wasm_bindgen(getter)]
    pub fn contract(&self) -> String {
        self.0.contract().to_string()
    }

    /// The storage layout tree of the contract.
    pub fn fields(&self) -> Result<JsValue, JsError> {
        Ok(JsValue::from_serde(self.0.layout())?)
    }

    /// Every value that can be read: `{ path, keys, type, storage }`, with the path and the key
    /// types as `locate` takes them.
    #[wasm_bindgen(js_name = "readableFields")]
    pub fn readable_fields(&self) -> Result<JsValue, JsError> {
        Ok(JsValue::from_serde(&readable_fields(self.0.layout()))?)
    }

    /// The hash (`hash-...`) of the current version of a contract package, from the
    /// `rawJSON.stored_value` of `queryLatestGlobalState(packageHash, [])`.
    #[wasm_bindgen(js_name = "currentContractHash")]
    pub fn current_contract_hash(&self, stored_value: JsValue) -> Result<String, JsError> {
        Ok(reader::current_contract_hash(&to_json(stored_value)?)?)
    }

    /// Resolves a dotted field path to the location of the value. `keys` are the keys of the
    /// mappings, list items and dictionaries on the path, in path order.
    pub fn locate(&self, path: &str, keys: Option<Vec<String>>) -> Result<JsValue, JsError> {
        let location = self.0.locate(path, &keys.unwrap_or_default())?;
        Ok(JsValue::from_serde(&location)?)
    }

    /// Decodes the value read from `location` (the result of `locate`). `clValue` is the
    /// `rawJSON.stored_value.CLValue` of the RPC result.
    pub fn decode(
        &self,
        #[wasm_bindgen(js_name = "clValue")] cl_value: JsValue,
        location: JsValue
    ) -> Result<String, JsError> {
        Ok(self
            .0
            .decode(&to_json(cl_value)?, &to_location(location)?)?)
    }

    /// The hex-encoded bytes of the value read from `location`, without decoding them, e.g. for a
    /// type that is not in the schema file. Takes the same arguments as `decode`.
    #[wasm_bindgen(js_name = "decodeRaw")]
    pub fn decode_raw(
        &self,
        #[wasm_bindgen(js_name = "clValue")] cl_value: JsValue,
        location: JsValue
    ) -> Result<String, JsError> {
        Ok(self
            .0
            .decode_raw(&to_json(cl_value)?, &to_location(location)?)?)
    }
}

fn to_json(value: JsValue) -> Result<Value, JsError> {
    Ok(value.into_serde::<Value>()?)
}

fn to_location(location: JsValue) -> Result<Location, Error> {
    location
        .into_serde()
        .map_err(|e| Error::InvalidLocation(e.to_string()))
}
