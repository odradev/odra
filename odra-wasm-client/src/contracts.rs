use gloo_utils::format::JsValueSerdeExt;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::{extensions::JsErrorContext, types::Address};

/// Container for multiple smart contract definitions with metadata.
///
/// This structure holds a collection of contract information along with
/// a timestamp indicating when the data was last updated. It can be
/// serialized/deserialized to/from JSON and is exposed to JavaScript.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, Eq)]
#[wasm_bindgen(inspectable)]
pub struct Contracts {
    #[serde(alias = "time")]
    last_updated: String,
    contracts: Vec<ContractInfo>
}

#[wasm_bindgen]
impl Contracts {
    /// Creates a new `Contracts` instance from a JavaScript value.
    ///
    /// # Arguments
    /// * `js` - JavaScript value containing contract data in JSON format
    ///
    /// # Returns
    /// * `Result<Self, JsError>` - New Contracts instance or error if parsing fails
    ///
    /// # Errors
    /// Returns `JsError` if the JavaScript value cannot be deserialized into a Contracts struct.
    #[wasm_bindgen(constructor)]
    pub fn new(js: JsValue) -> Result<Self, JsError> {
        js.into_serde::<Contracts>()
            .with_js_context("Failed to parse Contracts from JSON")
    }

    /// Asynchronously loads contract information from a remote JSON file.
    ///
    /// This method fetches contract data from the specified URL path using the browser's
    /// fetch API, parses the JSON response, and creates a new Contracts instance.
    ///
    /// # Arguments
    /// * `path` - URL path to the JSON file containing contract information
    ///
    /// # Returns
    /// * `Result<Self, JsError>` - New Contracts instance or error if loading fails
    ///
    /// # Errors
    /// Returns `JsError` if:
    /// - No window object is available (not in browser environment)
    /// - Network request fails
    /// - Response cannot be parsed as JSON
    /// - JSON structure doesn't match expected Contracts format
    #[wasm_bindgen(js_name = "fromPath")]
    pub async fn from_path(path: &str) -> Result<Self, JsError> {
        let window = web_sys::window().ok_or_else(|| JsError::new("No window object found"))?;
        let resp = JsFuture::from(window.fetch_with_str(path))
            .await
            .map_err(|err| {
                JsError::new(&format!("Failed to fetch contracts from {path}: {err:?}"))
            })?
            .dyn_into::<web_sys::Response>()
            .with_js_context("Failed to cast to Response")?;

        let promise = resp
            .json()
            .with_js_context("Failed to get JSON from response")?;
        let json = JsFuture::from(promise)
            .await
            .with_js_context("Failed to resolve JSON promise")?;

        let result = json
            .into_serde()
            .with_js_context("Failed to parse Contracts from JSON")?;
        crate::js::log(&format!("Loaded contracts: {:?}", result));
        Ok(result)
    }

    /// Retrieves contract information by name.
    ///
    /// Searches through the contracts collection for a contract with the specified name
    /// and returns its information if found.
    ///
    /// # Arguments
    /// * `name` - Name of the contract to find
    ///
    /// # Returns
    /// * `Result<ContractInfo, JsError>` - Contract information or error if not found
    ///
    /// # Errors
    /// Returns `JsError` if no contract with the specified name exists.
    #[wasm_bindgen]
    pub fn get(&self, name: &str) -> Result<ContractInfo, JsError> {
        let contract = self
            .contracts
            .iter()
            .find(|c| c.name == name)
            .ok_or_else(|| JsError::new(&format!("Contract with name {:?} not found", name)))?;
        Ok(contract.clone())
    }
}

/// Information about a specific smart contract.
///
/// Contains essential metadata for identifying and interacting with a deployed
/// smart contract, including its name and package hash used for addressing.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, Eq)]
#[wasm_bindgen(getter_with_clone, inspectable)]
pub struct ContractInfo {
    /// Human-readable name of the contract
    pub name: String,
    /// Unique hash identifier for the contract package
    pub package_hash: String
}

#[wasm_bindgen]
impl ContractInfo {
    /// Gets the contract's address derived from its package hash.
    ///
    /// Converts the package hash into an Address instance that can be used
    /// for contract interactions.
    ///
    /// # Returns
    /// * `Address` - Contract address derived from the package hash
    ///
    /// # Panics
    /// Panics if the package hash is invalid and cannot be converted to an Address.
    #[wasm_bindgen(getter)]
    pub fn address(&self) -> Address {
        Address::new(&self.package_hash).expect("Invalid package hash")
    }
}
