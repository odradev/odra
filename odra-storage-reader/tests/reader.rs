//! Tests against layouts exported with `odra-cli --json storage <Contract>` and node responses
//! captured from NCTL (`tests/fixtures`), so the expected values come from a real chain.

use std::collections::BTreeSet;

use casper_types::{
    bytesrepr::{Bytes, ToBytes},
    EntityAddr, EntityVersions, Groups, Package, PackageStatus, StoredValue, U256
};
use odra_schema::casper_contract_schema::{
    CustomType, EnumVariant, NamedCLType, StructMember, Type, TypeName
};
use odra_storage_reader::{current_contract_hash, Error, Location, StorageReader};
use serde_json::{json, Value};

const ACCOUNT: &str =
    "account-hash-9918c11ac0ccdd67942a143985499fc7c30c1ea10e7e8d049222ae1cdcdb39d3";

fn fixture(name: &str) -> Value {
    let path = format!("{}/tests/fixtures/{name}.json", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn reader(name: &str) -> StorageReader {
    StorageReader::from_json(&fixture(name)).unwrap()
}

fn cl_value(name: &str) -> Value {
    fixture(name)["stored_value"]["CLValue"].clone()
}

fn keys(keys: &[&str]) -> Vec<String> {
    keys.iter().map(|k| k.to_string()).collect()
}

fn dictionary(name: &str, key: &str, ty: NamedCLType) -> Location {
    Location::Dictionary {
        dictionary_name: name.to_string(),
        dictionary_item_key: key.to_string(),
        ty: Type(ty)
    }
}

#[test]
fn reads_erc20_balance_from_state() {
    let reader = reader("erc20_layout");
    assert_eq!(reader.contract(), "Erc20");

    let location = reader.locate("balances", &keys(&[ACCOUNT])).unwrap();
    // The key `odra-cli storage Erc20 balances --key ...` computed and NCTL returned.
    assert_eq!(
        location,
        dictionary(
            "state",
            "caebe967a9aa74246258b4583af2cf81dc5397df326a0d09c4830ee8c808c972",
            NamedCLType::U256
        )
    );

    // Stored as `List<U8>` (`03000000` + U256 `021027`).
    let value = cl_value("erc20_balance_item");
    assert_eq!(reader.decode(&value, &location, false).unwrap(), "10000");
    assert_eq!(reader.decode(&value, &location, true).unwrap(), "021027");
}

#[test]
fn reads_cep18_named_key_and_dictionary() {
    let reader = reader("cep18_layout");

    let location = reader.locate("decimals", &[]).unwrap();
    assert_eq!(
        location,
        Location::NamedKey {
            name: "decimals".to_string(),
            ty: Type(NamedCLType::U8)
        }
    );
    assert_eq!(
        reader
            .decode(&cl_value("cep18_decimals"), &location, false)
            .unwrap(),
        "2"
    );

    // A CEP-18 dictionary stores plain values, not `List<U8>`.
    let location = reader.locate("balances", &keys(&[ACCOUNT])).unwrap();
    assert_eq!(
        location,
        dictionary(
            "balances",
            "AJkYwRrAzN1nlCoUOYVJn8fDDB6hDn6NBJIirhzc2znT",
            NamedCLType::U256
        )
    );
    assert_eq!(
        reader
            .decode(&cl_value("cep18_balance_item"), &location, false)
            .unwrap(),
        "10000"
    );
}

#[test]
fn decode_accepts_the_whole_stored_value() {
    let reader = reader("erc20_layout");
    let location = reader.locate("balances", &keys(&[ACCOUNT])).unwrap();
    let stored_value = fixture("erc20_balance_item")["stored_value"].clone();
    assert_eq!(
        reader.decode(&stored_value, &location, false).unwrap(),
        "10000"
    );
}

#[test]
fn tuple_key_is_a_single_key() {
    let reader = reader("erc20_layout");
    let pair = format!("{ACCOUNT}:{ACCOUNT}");
    let location = reader.locate("allowances", &keys(&[&pair])).unwrap();
    assert!(
        matches!(location, Location::Dictionary { ref dictionary_name, .. } if dictionary_name == "state")
    );

    let err = reader
        .locate("allowances", &keys(&[ACCOUNT, ACCOUNT]))
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("expected tuple with 2 elements, found 1"),
        "{err}"
    );
}

#[test]
fn location_json_matches_the_rpc_params() {
    let reader = reader("erc20_layout");
    let location = reader.locate("balances", &keys(&[ACCOUNT])).unwrap();
    assert_eq!(
        serde_json::to_value(&location).unwrap(),
        json!({
            "kind": "dictionary",
            "dictionaryName": "state",
            "dictionaryItemKey": "caebe967a9aa74246258b4583af2cf81dc5397df326a0d09c4830ee8c808c972",
            "type": "U256"
        })
    );
    // What the frontend passes back to `decode`.
    let back: Location = serde_json::from_value(serde_json::to_value(&location).unwrap()).unwrap();
    assert_eq!(back, location);

    let named_key = self::reader("cep18_layout")
        .locate("decimals", &[])
        .unwrap();
    assert_eq!(
        serde_json::to_value(&named_key).unwrap(),
        json!({ "kind": "named_key", "name": "decimals", "type": "U8" })
    );
}

#[test]
fn resolves_the_legacy_contract_package() {
    // NCTL (and Casper 2.0 without addressable entities) returns `ContractPackage`.
    assert_eq!(
        current_contract_hash(&fixture("erc20_package")["stored_value"]).unwrap(),
        "hash-61d26cde57d2dc2f848e20525899b669afea2a96fd5e9d674d22aa990a2196bd"
    );
    // The whole RPC result works too.
    assert_eq!(
        current_contract_hash(&fixture("erc20_package")).unwrap(),
        "hash-61d26cde57d2dc2f848e20525899b669afea2a96fd5e9d674d22aa990a2196bd"
    );
}

#[test]
fn resolves_the_smart_contract_package_skipping_disabled_versions() {
    let mut package = Package::new(
        EntityVersions::default(),
        BTreeSet::new(),
        Groups::default(),
        PackageStatus::Unlocked
    );
    package.insert_entity_version(2, EntityAddr::SmartContract([1; 32]));
    let newest = EntityAddr::SmartContract([2; 32]);
    package.insert_entity_version(2, newest);
    package.disable_entity_version(newest).unwrap();

    let stored_value = serde_json::to_value(StoredValue::SmartContract(package)).unwrap();
    assert_eq!(
        current_contract_hash(&stored_value).unwrap(),
        format!("hash-{}", "01".repeat(32))
    );
}

#[test]
fn rejects_a_value_that_is_not_a_package() {
    let err = current_contract_hash(&fixture("cep18_decimals")["stored_value"]).unwrap_err();
    assert!(
        err.to_string()
            .contains("expected a contract package, found CLValue"),
        "{err}"
    );
}

#[test]
fn reports_path_and_key_errors() {
    let reader = reader("erc20_layout");
    let message = |path: &str, k: &[&str]| reader.locate(path, &keys(k)).unwrap_err().to_string();

    assert_eq!(
        message("balance", &[]),
        "Cannot resolve `balance`: Unknown field `balance`. Available fields: decimals, symbol, name, total_supply, balances, allowances"
    );
    assert_eq!(
        message("balances", &[]),
        "Cannot resolve `balances`: `balances` requires a key of type Key"
    );
    assert_eq!(
        message("total_supply", &[ACCOUNT]),
        "Cannot resolve `total_supply`: 1 key(s) were given but not used"
    );
    assert!(message("balances", &["xyz"])
        .starts_with("Cannot resolve `balances`: Invalid key for `balances`: cannot parse `xyz` as hash-...|account-hash-..."));
    // The CLI's form with the key in the path.
    assert_eq!(
        message(&format!(".balances.{ACCOUNT}"), &[]),
        "Cannot resolve `.balances.account-hash-9918c11ac0ccdd67942a143985499fc7c30c1ea10e7e8d049222ae1cdcdb39d3`: `balances` requires a key of type Key"
    );
}

#[test]
fn checks_the_layout_version() {
    let mut layout = fixture("erc20_layout");
    layout.as_object_mut().unwrap().remove("version");
    assert!(matches!(
        StorageReader::from_json(&layout),
        Err(Error::MissingVersion)
    ));

    layout["version"] = json!(2);
    let err = StorageReader::from_json(&layout).unwrap_err();
    assert_eq!(
        err.to_string(),
        "Unsupported layout file version 2, expected 1"
    );
}

#[test]
fn reports_a_type_missing_from_the_layout_file() {
    let layout = json!({
        "version": 1,
        "contract": "Vault",
        "ident": "Vault",
        "layout": {
            "kind": "module",
            "fields": [{ "name": "config", "index": 1, "kind": "value", "ty": { "Custom": "Config" } }]
        },
        "types": []
    });
    let reader = StorageReader::from_json(&layout).unwrap();
    let location = reader.locate("config", &[]).unwrap();
    let value = json!({ "cl_type": { "List": "U8" }, "bytes": "020000000102", "parsed": [1, 2] });

    let err = reader.decode(&value, &location, false).unwrap_err();
    assert_eq!(
        err.to_string(),
        "Cannot decode the stored value: type `Config` is not defined in the layout file (raw: 0x0102)"
    );
    assert_eq!(reader.decode(&value, &location, true).unwrap(), "0102");
}

#[test]
fn rejects_an_invalid_cl_value() {
    let reader = reader("erc20_layout");
    let location = reader.locate("total_supply", &[]).unwrap();
    let err = reader
        .decode(&json!({ "bytes": "00" }), &location, false)
        .unwrap_err();
    assert!(matches!(err, Error::InvalidClValue(_)), "{err}");
}

#[test]
fn decodes_a_struct_to_json() {
    let custom = |name: &str| NamedCLType::Custom(name.to_string());
    let types = vec![
        // Fields deliberately not in alphabetical order.
        CustomType::Struct {
            name: TypeName("Config".to_string()),
            description: None,
            members: vec![
                StructMember::new("name", "", NamedCLType::String),
                StructMember::new("limits", "", custom("Limits")),
                StructMember::new("kind", "", custom("Kind")),
                StructMember::new("tiers", "", NamedCLType::List(Box::new(custom("Limits")))),
                StructMember::new("kinds", "", NamedCLType::List(Box::new(custom("Kind")))),
            ]
        },
        CustomType::Struct {
            name: TypeName("Limits".to_string()),
            description: None,
            members: vec![StructMember::new("max", "", NamedCLType::U256)]
        },
        CustomType::Enum {
            name: TypeName("Kind".to_string()),
            description: None,
            variants: ["Free", "Pro"]
                .iter()
                .zip(0..)
                .map(|(name, discriminant)| EnumVariant {
                    name: name.to_string(),
                    description: None,
                    discriminant,
                    ty: Type(NamedCLType::Unit)
                })
                .collect()
        },
    ];
    let layout = json!({
        "version": 1,
        "contract": "Vault",
        "ident": "Vault",
        "layout": {
            "kind": "module",
            "fields": [{ "name": "config", "index": 1, "kind": "value", "ty": { "Custom": "Config" } }]
        },
        "types": types
    });
    let reader = StorageReader::from_json(&layout).unwrap();
    let location = reader.locate("config", &[]).unwrap();

    let mut bytes = "say \"hi\"".to_bytes().unwrap();
    bytes.extend(U256::from(5).to_bytes().unwrap());
    bytes.push(1); // Kind::Pro
    bytes.extend(2u32.to_bytes().unwrap());
    bytes.extend(U256::from(1).to_bytes().unwrap());
    bytes.extend(U256::from(2).to_bytes().unwrap());
    bytes.extend(2u32.to_bytes().unwrap());
    bytes.extend([0, 1]); // [Kind::Free, Kind::Pro]
    let value = json!({
        "cl_type": { "List": "U8" },
        "bytes": hex::encode(Bytes::from(bytes).to_bytes().unwrap())
    });

    let expected = json!({
        "name": "say \"hi\"",
        "limits": { "max": "5" },
        "kind": "Pro",
        "tiers": [{ "max": "1" }, { "max": "2" }],
        "kinds": ["Free", "Pro"]
    });
    assert_eq!(
        reader.decode(&value, &location, false).unwrap(),
        serde_json::to_string_pretty(&expected).unwrap()
    );
}
