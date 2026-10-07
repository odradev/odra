//! The reader logic, independent of `wasm-bindgen` so it can be tested natively.

use casper_types::{bytesrepr::Bytes, CLType, CLTyped, CLValue, Key, StoredValue};
use odra_core::consts::STATE_KEY;
use odra_schema::casper_contract_schema::Type;
use odra_schema::{resolve_storage_with, StorageKind, StorageLocation};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{types, CustomTypeSet, Error};

/// The version of the layout file format this reader supports, see `odra-cli`'s
/// `LAYOUT_FORMAT_VERSION`.
pub const LAYOUT_FORMAT_VERSION: u8 = 1;

/// The layout file, the output of `odra-cli --json storage <Contract>`.
#[derive(Deserialize)]
struct LayoutFile {
    contract: String,
    layout: StorageKind,
    types: CustomTypeSet
}

/// The node's `CLValue` JSON. `casper_types::CLValue` rejects the `parsed` field the node sends
/// unless its `json-schema` feature is on, which would bloat the wasm.
#[derive(Deserialize)]
struct ClValueJson {
    cl_type: CLType,
    bytes: String
}

/// Where a value is stored and how to query it with the node RPC.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum Location {
    /// An item of a contract dictionary, read with `state_get_dictionary_item` (`ContractNamedKey`).
    Dictionary {
        /// The name of the dictionary, `state` for regular Odra storage.
        dictionary_name: String,
        /// The dictionary item key.
        dictionary_item_key: String,
        /// The type of the stored value.
        #[serde(rename = "type")]
        ty: Type
    },
    /// A named key of the contract, read with `query_global_state` (`[name]` as the path).
    NamedKey {
        /// The name of the named key.
        name: String,
        /// The type of the stored value.
        #[serde(rename = "type")]
        ty: Type
    }
}

impl Location {
    /// The type of the stored value.
    pub fn ty(&self) -> &Type {
        match self {
            Location::Dictionary { ty, .. } | Location::NamedKey { ty, .. } => ty
        }
    }
}

/// Reads the storage of a single contract, described by a layout file.
#[derive(Debug)]
pub struct StorageReader {
    contract: String,
    layout: StorageKind,
    types: CustomTypeSet
}

impl StorageReader {
    /// Creates a reader from the layout file (the output of `odra-cli --json storage <Contract>`).
    pub fn from_json(json: &Value) -> Result<Self, Error> {
        match json.get("version").map(|v| v.as_u64()) {
            None => return Err(Error::MissingVersion),
            Some(Some(version)) if version == LAYOUT_FORMAT_VERSION as u64 => {}
            Some(Some(version)) => return Err(Error::UnsupportedVersion(version)),
            Some(None) => return Err(Error::InvalidLayout("`version` is not a number".into()))
        }
        let file =
            LayoutFile::deserialize(json).map_err(|e| Error::InvalidLayout(e.to_string()))?;
        Ok(Self {
            contract: file.contract,
            layout: file.layout,
            types: file.types
        })
    }

    /// The name of the contract.
    pub fn contract(&self) -> &str {
        &self.contract
    }

    /// The storage layout of the contract.
    pub fn layout(&self) -> &StorageKind {
        &self.layout
    }

    /// Resolves a dotted field `path` to the location of the value.
    ///
    /// `keys` are the keys of the `Mapping`s, `List` items and dictionaries on the path, in path
    /// order, in the `odra-cli` text format (e.g. `account-hash-...`, `some:5`, `a:b` for a tuple).
    pub fn locate(&self, path: &str, keys: &[String]) -> Result<Location, Error> {
        let resolve_err = |reason: String| Error::Resolve {
            path: path.to_string(),
            reason
        };

        let mut pending_keys = keys.iter();
        let query = resolve_storage_with(&self.layout, path, |ty: &Type| {
            let Some(key) = pending_keys.next() else {
                return Ok(None);
            };
            types::into_bytes(&ty.0, key).map(Some).map_err(|e| {
                format!(
                    "cannot parse `{key}` as {}: {e}",
                    types::format_type_hint(&ty.0)
                )
            })
        })
        .map_err(|e| resolve_err(e.to_string()))?;
        let unused = pending_keys.count();
        if unused > 0 {
            return Err(resolve_err(format!(
                "{unused} key(s) were given but not used"
            )));
        }

        Ok(match query.location {
            StorageLocation::State { key } => Location::Dictionary {
                dictionary_name: STATE_KEY.to_string(),
                dictionary_item_key: key,
                ty: query.ty
            },
            StorageLocation::NamedKey { name } => Location::NamedKey { name, ty: query.ty },
            StorageLocation::Dictionary { name, key } => Location::Dictionary {
                dictionary_name: name,
                dictionary_item_key: key,
                ty: query.ty
            }
        })
    }

    /// Decodes the value read from `location`.
    ///
    /// `cl_value` is the node's `CLValue` JSON as is (`stored_value.CLValue` of the RPC result).
    /// With `raw`, returns the hex-encoded stored bytes instead of decoding them.
    pub fn decode(
        &self,
        cl_value: &Value,
        location: &Location,
        raw: bool
    ) -> Result<String, Error> {
        let bytes = stored_bytes(cl_value, location)?;
        if raw {
            return Ok(hex::encode(&bytes));
        }
        types::decode(&bytes, location.ty(), &self.types)
            .map(|(value, _)| value)
            .map_err(|e| Error::Decode {
                reason: e.to_string(),
                raw: hex::encode(&bytes)
            })
    }
}

/// The bytes of the stored value, unwrapped the same way `odra-casper-rpc-client` does.
fn stored_bytes(cl_value: &Value, location: &Location) -> Result<Vec<u8>, Error> {
    // Accept the whole `stored_value` too.
    let cl_value = cl_value.get("CLValue").unwrap_or(cl_value);
    let invalid = |e: String| Error::InvalidClValue(e);
    let json = ClValueJson::deserialize(cl_value).map_err(|e| invalid(e.to_string()))?;
    let bytes = hex::decode(&json.bytes).map_err(|e| invalid(e.to_string()))?;
    let cl_value = CLValue::from_components(json.cl_type, bytes);
    match location {
        // Odra stores `state` items as `Bytes`; CEP-18 dictionaries store plain values.
        Location::Dictionary { .. } if cl_value.cl_type() == &<Vec<u8> as CLTyped>::cl_type() => {
            cl_value
                .into_t::<Bytes>()
                .map(Into::into)
                .map_err(|e| Error::InvalidClValue(e.to_string()))
        }
        _ => Ok(cl_value.inner_bytes().to_vec())
    }
}

/// The hash (`hash-...`) of the current version of a contract package.
///
/// `stored_value` is the node's `stored_value` of `query_global_state` for the package hash.
/// Both the Casper 2.0 `SmartContract` and the legacy `ContractPackage` formats are supported.
pub fn current_contract_hash(stored_value: &Value) -> Result<String, Error> {
    // Accept the whole RPC result too.
    let stored_value = stored_value.get("stored_value").unwrap_or(stored_value);
    // Check the variant first, other variants may not deserialize (see [ClValueJson]).
    match stored_value.as_object().and_then(|o| o.keys().next()) {
        Some(variant) if variant == "SmartContract" || variant == "ContractPackage" => {}
        Some(variant) => {
            return Err(Error::InvalidPackage(format!(
                "expected a contract package, found {variant}"
            )))
        }
        None => return Err(Error::InvalidPackage("expected a stored value".into()))
    }
    let stored_value =
        StoredValue::deserialize(stored_value).map_err(|e| Error::InvalidPackage(e.to_string()))?;
    let hash = match stored_value {
        StoredValue::SmartContract(package) => package
            .current_entity_hash()
            .map(|hash| hash.value())
            .ok_or_else(|| Error::InvalidPackage("the package has no current entity".into()))?,
        StoredValue::ContractPackage(package) => package
            .current_contract_hash()
            .map(|hash| hash.value())
            .ok_or_else(|| Error::InvalidPackage("the package has no current contract".into()))?,
        other => {
            return Err(Error::InvalidPackage(format!(
                "expected a contract package, found {}",
                other.type_name()
            )))
        }
    };
    Ok(Key::Hash(hash).to_formatted_string())
}
