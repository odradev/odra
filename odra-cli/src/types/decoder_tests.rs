//! The decoder of `odra_schema::codec` with the CLI's test types.

use std::collections::BTreeMap;

use odra::{
    casper_types::bytesrepr::{ToBytes, RESULT_ERR_TAG, RESULT_OK_TAG},
    prelude::Address,
    schema::{
        casper_contract_schema::{NamedCLType, Type},
        SchemaCustomTypes
    }
};
use serde_json::json;

use crate::test_utils::{self, NameMintInfo, PaymentInfo, PaymentVoucher, Status};

const NAMED_TOKEN_METADATA_BYTES: [u8; 50] = [
    4, 0, 0, 0, 107, 112, 111, 98, 0, 32, 74, 169, 209, 1, 0, 0, 1, 1, 226, 74, 54, 110, 186, 196,
    135, 233, 243, 218, 49, 175, 91, 142, 42, 103, 172, 205, 97, 76, 95, 247, 61, 188, 60, 100, 10,
    52, 124, 59, 94, 73
];

const NAMED_TOKEN_METADATA_JSON: &str = r#"{
  "token_hash": "kpob",
  "expiration": "2000000000000",
  "resolver": "Key::Hash(e24a366ebac487e9f3da31af5b8e2a67accd614c5ff73dbc3c640a347c3b5e49)"
}"#;

#[test]
fn test_decode_custom_type() {
    let custom_types = test_utils::custom_types();

    let ty = Type(NamedCLType::Custom("NameTokenMetadata".to_string()));
    let (result, _bytes) = super::decode(&NAMED_TOKEN_METADATA_BYTES, &ty, &custom_types).unwrap();
    pretty_assertions::assert_eq!(result, NAMED_TOKEN_METADATA_JSON);
}

#[test]
fn test_decode_map() {
    let custom_types = test_utils::custom_types();

    let ty = Type(NamedCLType::Map {
        key: Box::new(NamedCLType::String),
        value: Box::new(NamedCLType::U64)
    });

    let map = BTreeMap::from_iter([("foo".to_string(), 1u64), ("bar".to_string(), 2u64)]);
    let bytes = map.to_bytes().unwrap();

    let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
    pretty_assertions::assert_eq!(result, r#"bar:2, foo:1"#);
}

#[test]
fn test_decode_list() {
    let custom_types = test_utils::custom_types();
    let ty = Type(NamedCLType::List(Box::new(NamedCLType::U64)));
    let list = vec![1u64, 2u64, 3u64];
    let bytes = list.to_bytes().unwrap();
    let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
    pretty_assertions::assert_eq!(result, "[1,2,3]");

    let list = Vec::<u64>::new();
    let bytes = list.to_bytes().unwrap();
    let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
    pretty_assertions::assert_eq!(result, "[]");
}

#[test]
fn test_decode_option() {
    let custom_types = test_utils::custom_types();

    let ty = Type(NamedCLType::Option(Box::new(NamedCLType::U64)));
    let value = Some(42u64);
    let bytes = value.to_bytes().unwrap();
    let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
    pretty_assertions::assert_eq!(result, "42");

    let value: Option<u64> = None;
    let bytes = value.to_bytes().unwrap();
    let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
    pretty_assertions::assert_eq!(result, "None");
}

#[test]
fn test_decode_result() {
    let custom_types = test_utils::custom_types();

    let ty = Type(NamedCLType::Result {
        ok: Box::new(NamedCLType::U64),
        err: Box::new(NamedCLType::String)
    });

    let value: Result<u64, String> = Ok(42u64);
    let bytes = value.to_bytes().unwrap();
    let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
    pretty_assertions::assert_eq!(result, "Ok(42)");

    let value: Result<u64, String> = Err("Error".to_string());
    let bytes = value.to_bytes().unwrap();
    let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
    pretty_assertions::assert_eq!(result, "Err(Error)");
}

#[test]
fn test_decode_tuple() {
    let custom_types = test_utils::custom_types();

    let ty = Type(NamedCLType::Tuple1([Box::new(NamedCLType::U64)]));

    let value = (42u64,);
    let bytes = value.to_bytes().unwrap();
    let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
    pretty_assertions::assert_eq!(result, "(42)");

    let ty = Type(NamedCLType::Tuple2([
        Box::new(NamedCLType::U64),
        Box::new(NamedCLType::String)
    ]));

    let value = (42u64, "Hello".to_string());
    let bytes = value.to_bytes().unwrap();
    let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
    pretty_assertions::assert_eq!(result, "(42, Hello)");

    let ty = Type(NamedCLType::Tuple3([
        Box::new(NamedCLType::U64),
        Box::new(NamedCLType::String),
        Box::new(NamedCLType::Bool)
    ]));

    let value = (42u64, "Hello".to_string(), true);
    let bytes = value.to_bytes().unwrap();
    let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
    pretty_assertions::assert_eq!(result, "(42, Hello, true)");
}

#[test]
fn test_option_custom_type() {
    let custom_types = test_utils::custom_types();

    let ty = Type(NamedCLType::Option(Box::new(NamedCLType::Custom(
        "NameTokenMetadata".to_string()
    ))));

    let mut bytes = vec![1];
    bytes.extend_from_slice(&NAMED_TOKEN_METADATA_BYTES);
    let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
    pretty_assertions::assert_eq!(result, NAMED_TOKEN_METADATA_JSON);

    let (result, _bytes) = super::decode(&[0], &ty, &custom_types).unwrap();
    pretty_assertions::assert_eq!(result, "None");
}

#[test]
fn test_decode_simple_type() {
    let custom_types = test_utils::custom_types();
    let ty = Type(NamedCLType::U64);
    let value = 42u64;
    let bytes = value.to_bytes().unwrap();
    let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
    pretty_assertions::assert_eq!(result, "42");

    let ty = Type(NamedCLType::String);
    let value = "Hello".to_string();
    let bytes = value.to_bytes().unwrap();
    let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
    pretty_assertions::assert_eq!(result, "Hello");
}

#[test]
fn test_decode_result_custom_type() {
    let custom_types = test_utils::custom_types();

    let ty = Type(NamedCLType::Result {
        ok: Box::new(NamedCLType::Custom("NameTokenMetadata".to_string())),
        err: Box::new(NamedCLType::String)
    });

    let mut bytes = vec![RESULT_OK_TAG];
    bytes.extend_from_slice(&NAMED_TOKEN_METADATA_BYTES);
    let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
    pretty_assertions::assert_eq!(result, format!("Ok({})", NAMED_TOKEN_METADATA_JSON));

    let mut bytes = vec![RESULT_ERR_TAG];
    bytes.extend_from_slice(&"Error".to_string().to_bytes().unwrap());
    let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
    pretty_assertions::assert_eq!(result, "Err(Error)");
}

#[test]
fn test_decode_map_custom_type() {
    let custom_types = test_utils::custom_types();

    let ty = Type(NamedCLType::Map {
        key: Box::new(NamedCLType::String),
        value: Box::new(NamedCLType::Custom("NameTokenMetadata".to_string()))
    });

    let mut bytes = 1u32.to_bytes().unwrap();
    bytes.extend_from_slice(&"foo".to_string().to_bytes().unwrap());
    bytes.extend_from_slice(&NAMED_TOKEN_METADATA_BYTES);

    let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
    pretty_assertions::assert_eq!(result, format!("foo:{}", NAMED_TOKEN_METADATA_JSON));
}

#[test]
fn test_decode_nested_custom_types() {
    let custom_types = test_utils::custom_types();
    let buyer = "account-hash-9918c11ac0ccdd67942a143985499fc7c30c1ea10e7e8d049222ae1cdcdb39d3";
    let voucher = PaymentVoucher::new(
        PaymentInfo::new(buyer, "say \"hi\"", "100"),
        vec![
            NameMintInfo::new("kpob", buyer, 1),
            NameMintInfo::new("odra", buyer, 2),
        ],
        3
    );
    let bytes = voucher.to_bytes().unwrap();
    let ty = Type(NamedCLType::Custom("PaymentVoucher".to_string()));
    let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();

    let owner = buyer.parse::<Address>().unwrap().as_key().to_string();
    let expected = json!({
        "payment": { "buyer": owner, "payment_id": "say \"hi\"", "amount": "100" },
        "names": [
            { "label": "kpob", "owner": owner, "token_expiration": "1" },
            { "label": "odra", "owner": owner, "token_expiration": "2" }
        ],
        "voucher_expiration": "3"
    });
    pretty_assertions::assert_eq!(result, serde_json::to_string_pretty(&expected).unwrap());
}

#[test]
fn test_decode_list_of_enums() {
    let custom_types = Status::schema_types().into_iter().flatten().collect();
    let ty = Type(NamedCLType::List(Box::new(NamedCLType::Custom(
        "Status".to_string()
    ))));
    let bytes = vec![Status::Active, Status::Terminated].to_bytes().unwrap();
    let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
    pretty_assertions::assert_eq!(
        result,
        serde_json::to_string_pretty(&json!(["Active", "Terminated"])).unwrap()
    );
}
