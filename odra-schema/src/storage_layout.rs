//! Storage layout of Odra modules.
//!
//! Every `#[odra::module]` gets a generated [SchemaStorageLayout] implementation that describes
//! where each field lives in the contract storage: the index of the field, the kind of storage
//! primitive and the types involved. The layout is a tree, because a module can be composed of
//! other modules.
//!
//! [resolve_storage] walks the tree for a dotted field path (e.g. `erc20.balances`) and produces
//! the exact location the value is stored under, so the state of a deployed contract can be read
//! without calling any entry point. That is what `odra-cli storage` does under the hood.

use std::fmt::{self, Display, Formatter};
use std::io::Write;

use base64::prelude::{Engine, BASE64_STANDARD};
use blake2::digest::VariableOutput;
use blake2::Blake2bVar;
use casper_contract_schema::{NamedCLType, Type};
use casper_types::bytesrepr::{FromBytes, ToBytes};
use casper_types::CLTyped;
use num_traits::{Num, One};
use odra_core::prelude::*;
use odra_core::{utils, ContractRef};
use serde::Serialize;

use crate::NamedCLTyped;

/// A single field of a module together with its index in the module.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct StorageField {
    /// The field name as written in the module struct.
    pub name: String,
    /// The index of the field in the module (1-based, `env` excluded).
    pub index: u8,
    /// What is stored under the field.
    #[serde(flatten)]
    pub kind: StorageKind
}

impl StorageField {
    /// Creates a new field description.
    pub fn new(name: &str, index: u8, kind: StorageKind) -> Self {
        Self {
            name: name.to_string(),
            index,
            kind
        }
    }
}

/// How a dictionary item key is derived from the key value.
#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KeyEncoding {
    /// The key is a string used as-is.
    Utf8,
    /// The serialized key is base64-encoded.
    Base64,
    /// The serialized key is hashed with `blake2b` and hex-encoded.
    HexHash
}

/// Describes what is stored under a module element.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StorageKind {
    /// A single value (`Var<T>`, `External<T>`).
    Value {
        /// The type of the value.
        ty: Type
    },
    /// A key-value store (`Mapping<K, V>`).
    Mapping {
        /// The type of the key.
        key: Type,
        /// What is stored under a key - a value or a whole module.
        value: Box<StorageKind>
    },
    /// An indexed collection (`List<T>`).
    List {
        /// The type of the items.
        item: Type
    },
    /// A counter (`Sequence<T>`).
    Sequence {
        /// The type of the counter.
        ty: Type
    },
    /// A composition of other modules (`SubModule<M>` or the contract itself).
    Module {
        /// The fields of the module.
        fields: Vec<StorageField>
    },
    /// A value stored under a Casper named key.
    NamedKey {
        /// The name of the named key.
        name: String,
        /// The type of the value.
        ty: Type
    },
    /// A value stored in a Casper dictionary.
    Dictionary {
        /// The name of the dictionary.
        name: String,
        /// The type of the key.
        key: Type,
        /// How the dictionary item key is derived from the key value.
        key_encoding: KeyEncoding,
        /// The type of the value.
        value: Type
    }
}

impl StorageKind {
    /// A single value of type `T`.
    pub fn value<T: NamedCLTyped>() -> Self {
        Self::Value { ty: Type(T::ty()) }
    }

    /// A value of type `T` stored under a named key.
    pub fn named_key<T: NamedCLTyped>(name: &str) -> Self {
        Self::NamedKey {
            name: name.to_string(),
            ty: Type(T::ty())
        }
    }

    /// A value of type `V` stored in a dictionary under a key of type `K`.
    pub fn dictionary<K: NamedCLTyped, V: NamedCLTyped>(
        name: &str,
        key_encoding: KeyEncoding
    ) -> Self {
        Self::Dictionary {
            name: name.to_string(),
            key: Type(K::ty()),
            key_encoding,
            value: Type(V::ty())
        }
    }

    /// The fields of a module, empty for any other kind.
    pub fn fields(&self) -> &[StorageField] {
        match self {
            Self::Module { fields } => fields,
            _ => &[]
        }
    }
}

/// Describes the storage layout of a module element.
///
/// Implemented by every storage primitive, every `#[odra::module]` (generated)
/// and, as a plain value, every type that has a [NamedCLTyped] representation.
pub trait SchemaStorageLayout {
    /// Returns the layout of the element.
    fn storage_kind() -> StorageKind;
}

impl<T: NamedCLTyped> SchemaStorageLayout for T {
    fn storage_kind() -> StorageKind {
        StorageKind::value::<T>()
    }
}

impl<T: NamedCLTyped> SchemaStorageLayout for Var<T> {
    fn storage_kind() -> StorageKind {
        StorageKind::value::<T>()
    }
}

impl<K: NamedCLTyped + ToBytes, V: SchemaStorageLayout> SchemaStorageLayout for Mapping<K, V> {
    fn storage_kind() -> StorageKind {
        StorageKind::Mapping {
            key: Type(K::ty()),
            value: Box::new(V::storage_kind())
        }
    }
}

impl<T: NamedCLTyped> SchemaStorageLayout for List<T> {
    fn storage_kind() -> StorageKind {
        StorageKind::List {
            item: Type(T::ty())
        }
    }
}

impl<T> SchemaStorageLayout for Sequence<T>
where
    T: NamedCLTyped + Num + One + ToBytes + FromBytes + CLTyped
{
    fn storage_kind() -> StorageKind {
        StorageKind::Sequence { ty: Type(T::ty()) }
    }
}

impl<T: ContractRef> SchemaStorageLayout for External<T> {
    fn storage_kind() -> StorageKind {
        StorageKind::value::<Address>()
    }
}

impl<M: SchemaStorageLayout> SchemaStorageLayout for SubModule<M> {
    fn storage_kind() -> StorageKind {
        M::storage_kind()
    }
}

/// Where a value is stored in the contract storage.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "location", rename_all = "snake_case")]
pub enum StorageLocation {
    /// An item of the Odra `state` dictionary.
    State {
        /// The dictionary item key (hex-encoded hash).
        key: String
    },
    /// A named key of the contract.
    NamedKey {
        /// The name of the named key.
        name: String
    },
    /// An item of a named dictionary of the contract.
    Dictionary {
        /// The name of the dictionary.
        name: String,
        /// The dictionary item key.
        key: String
    }
}

/// The result of resolving a field path against a storage layout.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct StorageQuery {
    /// The type of the stored value.
    pub ty: Type,
    /// Where the value is stored.
    #[serde(flatten)]
    pub location: StorageLocation
}

/// An error that can occur while resolving a field path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageLayoutError {
    /// A path segment does not name a field of the module.
    UnknownField {
        /// The unknown segment.
        name: String,
        /// The fields of the module.
        available: Vec<String>
    },
    /// The element requires a key (a `Mapping`, a `List` item, a dictionary) but none was given.
    MissingKey {
        /// The path of the element.
        path: String,
        /// The type of the expected key.
        ty: Type
    },
    /// More keys were given than the path consumes.
    UnusedKeys {
        /// The number of keys that were not used.
        count: usize
    },
    /// The path continues past a leaf value.
    NotAModule {
        /// The path of the leaf.
        path: String
    },
    /// The path ends on a module, which has no single value.
    NotAValue {
        /// The path of the module.
        path: String,
        /// The fields of the module.
        available: Vec<String>
    },
    /// A key could not be encoded as required by the storage.
    InvalidKey {
        /// The path of the element.
        path: String,
        /// The reason.
        reason: String
    },
    /// The path is deeper than the storage supports.
    PathTooDeep
}

impl Display for StorageLayoutError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownField { name, available } => write!(
                f,
                "Unknown field `{name}`. Available fields: {}",
                available.join(", ")
            ),
            Self::MissingKey { path, ty } => {
                write!(f, "`{path}` requires a key of type {:?}", ty.0)
            }
            Self::UnusedKeys { count } => write!(f, "{count} key(s) were given but not used"),
            Self::NotAModule { path } => write!(f, "`{path}` is a value, it has no fields"),
            Self::NotAValue { path, available } => write!(
                f,
                "`{path}` is a module, choose one of its fields: {}",
                available.join(", ")
            ),
            Self::InvalidKey { path, reason } => write!(f, "Invalid key for `{path}`: {reason}"),
            Self::PathTooDeep => write!(f, "The path is deeper than the storage supports")
        }
    }
}

impl std::error::Error for StorageLayoutError {}

/// The length of the `List` sub-path that stores the items.
const LIST_ITEMS_INDEX: u8 = 0;
/// The length of the `List` sub-path that stores the number of items.
const LIST_LENGTH_INDEX: u8 = 1;
/// The sub-path of the `Sequence` value.
const SEQUENCE_VALUE_INDEX: u8 = 0;
/// The field name that resolves to the number of items of a `List`.
const LIST_LENGTH_FIELD: &str = "len";
/// The maximum depth of a storage path, mirrors `ContractEnv`.
const MAX_PATH_LEN: usize = 8;

/// Resolves a dotted field `path` (e.g. `erc20.balances`) against the `layout` of a contract
/// and returns the storage location and the type of the value.
///
/// `keys` are consumed in order every time the path goes through a `Mapping`, a `List` item or a
/// dictionary. They must be serialized with `ToBytes` (the same way a contract serializes them).
/// The number of items of a `List` is available under the `len` pseudo-field.
pub fn resolve_storage(
    layout: &StorageKind,
    path: &str,
    keys: &[Vec<u8>]
) -> Result<StorageQuery, StorageLayoutError> {
    let mut keys = keys.iter();
    let query = resolve_storage_with(layout, path, |_| Ok(keys.next().cloned()))?;
    let unused = keys.len();
    if unused > 0 {
        return Err(StorageLayoutError::UnusedKeys { count: unused });
    }
    Ok(query)
}

/// Resolves a dotted field `path` like [resolve_storage], but the keys are produced on demand.
///
/// `next_key` is called with the expected key type every time the path goes through a
/// `Mapping`, a `List` item or a dictionary. It returns the serialized key, `None` if there are no
/// more keys, or an error message if the key cannot be produced for the given type.
pub fn resolve_storage_with<F>(
    layout: &StorageKind,
    path: &str,
    next_key: F
) -> Result<StorageQuery, StorageLayoutError>
where
    F: FnMut(&Type) -> Result<Option<Vec<u8>>, String>
{
    let mut resolver = Resolver {
        next_key,
        path: Vec::new(),
        mapping_data: Vec::new(),
        visited: Vec::new()
    };
    let segments = path.split('.').filter(|s| !s.is_empty());
    let mut current = layout.clone();
    for segment in segments {
        current = resolver.descend(current, segment)?;
    }
    resolver.finish(current)
}

struct Resolver<F> {
    next_key: F,
    path: Vec<u8>,
    mapping_data: Vec<u8>,
    visited: Vec<String>
}

impl<F> Resolver<F>
where
    F: FnMut(&Type) -> Result<Option<Vec<u8>>, String>
{
    fn location(&self) -> String {
        self.visited.join(".")
    }

    fn push_index(&mut self, index: u8) -> Result<(), StorageLayoutError> {
        if self.path.len() >= MAX_PATH_LEN {
            return Err(StorageLayoutError::PathTooDeep);
        }
        self.path.push(index);
        Ok(())
    }

    fn next_key(&mut self, ty: &Type) -> Result<Vec<u8>, StorageLayoutError> {
        let key = (self.next_key)(ty).map_err(|reason| StorageLayoutError::InvalidKey {
            path: self.location(),
            reason
        })?;
        key.ok_or_else(|| StorageLayoutError::MissingKey {
            path: self.location(),
            ty: ty.clone()
        })
    }

    /// Unwraps keyed containers until a module or a leaf value is reached.
    fn unwrap(&mut self, kind: StorageKind) -> Result<StorageKind, StorageLayoutError> {
        match kind {
            StorageKind::Mapping { key, value } => {
                let key_bytes = self.next_key(&key)?;
                self.mapping_data.extend_from_slice(&key_bytes);
                self.unwrap(*value)
            }
            StorageKind::List { item } => {
                let key_bytes = self.next_key(&Type(NamedCLType::U32))?;
                self.push_index(LIST_ITEMS_INDEX)?;
                self.mapping_data.extend_from_slice(&key_bytes);
                Ok(StorageKind::Value { ty: item })
            }
            StorageKind::Sequence { ty } => {
                self.push_index(SEQUENCE_VALUE_INDEX)?;
                Ok(StorageKind::Value { ty })
            }
            other => Ok(other)
        }
    }

    fn descend(
        &mut self,
        kind: StorageKind,
        segment: &str
    ) -> Result<StorageKind, StorageLayoutError> {
        if let StorageKind::List { .. } = &kind {
            if segment == LIST_LENGTH_FIELD {
                self.push_index(LIST_LENGTH_INDEX)?;
                self.visited.push(segment.to_string());
                return Ok(StorageKind::Value {
                    ty: Type(NamedCLType::U32)
                });
            }
        }
        let kind = self.unwrap(kind)?;
        let StorageKind::Module { fields } = kind else {
            return Err(StorageLayoutError::NotAModule {
                path: self.location()
            });
        };
        let field = fields.iter().find(|f| f.name == segment).ok_or_else(|| {
            StorageLayoutError::UnknownField {
                name: segment.to_string(),
                available: fields.iter().map(|f| f.name.clone()).collect()
            }
        })?;
        self.push_index(field.index)?;
        self.visited.push(segment.to_string());
        Ok(field.kind.clone())
    }

    fn finish(&mut self, kind: StorageKind) -> Result<StorageQuery, StorageLayoutError> {
        let kind = self.unwrap(kind)?;
        match kind {
            StorageKind::Value { ty } => Ok(StorageQuery {
                ty,
                location: StorageLocation::State {
                    key: state_key(&self.path, &self.mapping_data)
                }
            }),
            StorageKind::NamedKey { name, ty } => Ok(StorageQuery {
                ty,
                location: StorageLocation::NamedKey { name }
            }),
            StorageKind::Dictionary {
                name,
                key,
                key_encoding,
                value
            } => {
                let key_bytes = self.next_key(&key)?;
                let key = encode_dictionary_key(&key_bytes, key_encoding).map_err(|reason| {
                    StorageLayoutError::InvalidKey {
                        path: self.location(),
                        reason
                    }
                })?;
                Ok(StorageQuery {
                    ty: value,
                    location: StorageLocation::Dictionary { name, key }
                })
            }
            StorageKind::Module { fields } => Err(StorageLayoutError::NotAValue {
                path: self.location(),
                available: fields.iter().map(|f| f.name.clone()).collect()
            }),
            StorageKind::Mapping { .. }
            | StorageKind::List { .. }
            | StorageKind::Sequence { .. } => {
                unreachable!("keyed containers are unwrapped before")
            }
        }
    }
}

/// Computes the `state` dictionary item key for a storage path and mapping data,
/// exactly the way `ContractEnv` does.
pub fn state_key(path: &[u8], mapping_data: &[u8]) -> String {
    let preimage = utils::storage_key_preimage(path, mapping_data);
    hex::encode(blake2b(&preimage))
}

fn encode_dictionary_key(key: &[u8], encoding: KeyEncoding) -> Result<String, String> {
    match encoding {
        KeyEncoding::Utf8 => String::from_bytes(key)
            .map(|(s, _)| s)
            .map_err(|_| "expected a string".to_string()),
        KeyEncoding::Base64 => Ok(BASE64_STANDARD.encode(key)),
        KeyEncoding::HexHash => Ok(hex::encode(blake2b(key)))
    }
}

fn blake2b(bytes: &[u8]) -> [u8; 32] {
    let mut result = [0u8; 32];
    let mut hasher = <Blake2bVar as VariableOutput>::new(32).expect("should create hasher");
    let _ = hasher.write(bytes);
    hasher
        .finalize_variable(&mut result)
        .expect("should copy hash to the result array");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use casper_types::U256;

    fn token() -> StorageKind {
        StorageKind::Module {
            fields: vec![
                StorageField::new("name", 1, StorageKind::value::<String>()),
                StorageField::new(
                    "balances",
                    2,
                    <Mapping<Address, U256> as SchemaStorageLayout>::storage_kind()
                ),
                StorageField::new(
                    "holders",
                    3,
                    <List<Address> as SchemaStorageLayout>::storage_kind()
                ),
                StorageField::new(
                    "ids",
                    4,
                    <Sequence<u32> as SchemaStorageLayout>::storage_kind()
                ),
            ]
        }
    }

    fn loans() -> StorageKind {
        StorageKind::Module {
            fields: vec![
                StorageField::new("lenders", 1, token()),
                StorageField::new("borrowers", 2, token()),
                StorageField::new("decimals", 3, StorageKind::named_key::<u8>("decimals")),
                StorageField::new(
                    "allowances",
                    4,
                    StorageKind::dictionary::<String, U256>("allowances", KeyEncoding::Utf8)
                ),
            ]
        }
    }

    fn state(path: &[u8], mapping_data: &[u8]) -> StorageLocation {
        StorageLocation::State {
            key: state_key(path, mapping_data)
        }
    }

    #[test]
    fn resolves_var_in_submodule() {
        let query = resolve_storage(&loans(), "borrowers.name", &[]).unwrap();
        assert_eq!(query.ty, Type(NamedCLType::String));
        assert_eq!(query.location, state(&[2, 1], &[]));
    }

    #[test]
    fn resolves_mapping_with_key() {
        let key = 7u8.to_bytes().unwrap();
        let query =
            resolve_storage(&loans(), "lenders.balances", std::slice::from_ref(&key)).unwrap();
        assert_eq!(query.ty, Type(NamedCLType::U256));
        assert_eq!(query.location, state(&[1, 2], &key));
    }

    #[test]
    fn resolves_list_item_and_length() {
        let key = 3u32.to_bytes().unwrap();
        let item =
            resolve_storage(&loans(), "lenders.holders", std::slice::from_ref(&key)).unwrap();
        assert_eq!(item.ty, Type(NamedCLType::Key));
        assert_eq!(item.location, state(&[1, 3, 0], &key));

        let len = resolve_storage(&loans(), "lenders.holders.len", &[]).unwrap();
        assert_eq!(len.ty, Type(NamedCLType::U32));
        assert_eq!(len.location, state(&[1, 3, 1], &[]));
    }

    #[test]
    fn resolves_sequence() {
        let query = resolve_storage(&loans(), "lenders.ids", &[]).unwrap();
        assert_eq!(query.ty, Type(NamedCLType::U32));
        assert_eq!(query.location, state(&[1, 4, 0], &[]));
    }

    #[test]
    fn resolves_named_key_and_dictionary() {
        let query = resolve_storage(&loans(), "decimals", &[]).unwrap();
        assert_eq!(query.ty, Type(NamedCLType::U8));
        assert_eq!(
            query.location,
            StorageLocation::NamedKey {
                name: "decimals".to_string()
            }
        );

        let key = "alice".to_string().to_bytes().unwrap();
        let query = resolve_storage(&loans(), "allowances", &[key]).unwrap();
        assert_eq!(
            query.location,
            StorageLocation::Dictionary {
                name: "allowances".to_string(),
                key: "alice".to_string()
            }
        );
    }

    #[test]
    fn reports_errors() {
        assert!(matches!(
            resolve_storage(&loans(), "lenders.missing", &[]),
            Err(StorageLayoutError::UnknownField { .. })
        ));
        assert!(matches!(
            resolve_storage(&loans(), "lenders.balances", &[]),
            Err(StorageLayoutError::MissingKey { .. })
        ));
        assert!(matches!(
            resolve_storage(&loans(), "lenders", &[]),
            Err(StorageLayoutError::NotAValue { .. })
        ));
        assert!(matches!(
            resolve_storage(&loans(), "lenders.name.x", &[]),
            Err(StorageLayoutError::NotAModule { .. })
        ));
        assert!(matches!(
            resolve_storage(&loans(), "lenders.name", &[vec![1]]),
            Err(StorageLayoutError::UnusedKeys { count: 1 })
        ));
    }

    #[test]
    fn state_key_matches_legacy_encoding() {
        // Path [1, 2] packs into 0x00000012 in the legacy (nibble) encoding.
        let preimage = utils::storage_key_preimage(&[1, 2], &[9]);
        assert_eq!(preimage, vec![0, 0, 0, 0x12, 9]);
        assert_eq!(state_key(&[1, 2], &[9]).len(), 64);
    }

    #[test]
    fn serializes_layout() {
        let json = serde_json::to_value(loans()).unwrap();
        assert_eq!(json["kind"], "module");
        assert_eq!(json["fields"][0]["name"], "lenders");
        assert_eq!(json["fields"][0]["fields"][1]["kind"], "mapping");
        assert_eq!(json["fields"][0]["fields"][1]["key"], "Key");
        assert_eq!(json["fields"][3]["key_encoding"], "utf8");
    }
}
