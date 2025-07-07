use odra::schema::casper_contract_schema::NamedCLType;

use crate::types::{into_bytes, Error, Format};

#[test]
fn test_bool() {
    let ty = NamedCLType::Bool;
    assert_eq!(into_bytes(&ty, "true").unwrap(), vec![1]);
    assert_eq!(into_bytes(&ty, "false").unwrap(), vec![0]);
    assert!(into_bytes(&ty, "a").is_err());
}

#[test]
fn test_i32() {
    let ty = NamedCLType::I32;
    assert_eq!(into_bytes(&ty, "0").unwrap(), vec![0, 0, 0, 0]);
    assert_eq!(into_bytes(&ty, "1").unwrap(), vec![1, 0, 0, 0]);
    assert_eq!(into_bytes(&ty, "-1").unwrap(), vec![255, 255, 255, 255]);
    assert!(into_bytes(&ty, "a").is_err());
}

#[test]
fn test_i64() {
    let ty = NamedCLType::I64;
    assert_eq!(into_bytes(&ty, "0").unwrap(), vec![0, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(into_bytes(&ty, "1").unwrap(), vec![1, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(
        into_bytes(&ty, "-1").unwrap(),
        vec![255, 255, 255, 255, 255, 255, 255, 255]
    );
    assert!(into_bytes(&ty, "a").is_err());
}

#[test]
fn test_u8() {
    let ty = NamedCLType::U8;
    assert_eq!(into_bytes(&ty, "0").unwrap(), vec![0]);
    assert_eq!(into_bytes(&ty, "1").unwrap(), vec![1]);
    assert_eq!(into_bytes(&ty, "255").unwrap(), vec![255]);
    assert_eq!(into_bytes(&ty, "0x0f").unwrap(), vec![15]);
    assert_eq!(into_bytes(&ty, "0b1111").unwrap(), vec![15]);
    assert_eq!(
        into_bytes(&ty, "a"),
        Err(Error::Formatting(Format::U8))
    );
}

#[test]
fn test_u32() {
    let ty = NamedCLType::U32;
    assert_eq!(into_bytes(&ty, "0").unwrap(), vec![0, 0, 0, 0]);
    assert_eq!(into_bytes(&ty, "1").unwrap(), vec![1, 0, 0, 0]);
    assert_eq!(
        into_bytes(&ty, "4294967295").unwrap(),
        vec![255, 255, 255, 255]
    );
    assert!(into_bytes(&ty, "a").is_err());
}

#[test]
fn test_u64() {
    let ty = NamedCLType::U64;
    assert_eq!(into_bytes(&ty, "0").unwrap(), vec![0, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(into_bytes(&ty, "1").unwrap(), vec![1, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(
        into_bytes(&ty, "18446744073709551615").unwrap(),
        vec![255, 255, 255, 255, 255, 255, 255, 255]
    );
    assert!(into_bytes(&ty, "a").is_err());
}

#[test]
fn test_u128() {
    let ty = NamedCLType::U128;
    assert_eq!(into_bytes(&ty, "0").unwrap(), vec![0]);
    assert_eq!(into_bytes(&ty, "1").unwrap(), vec![1, 1]);
    assert_eq!(
        into_bytes(&ty, "340282366920938463463374607431768211455").unwrap(),
        vec![16, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255]
    );
    assert!(into_bytes(&ty, "a").is_err());
}

#[test]
fn test_u256() {
    let ty = NamedCLType::U256;
    assert_eq!(into_bytes(&ty, "0").unwrap(), vec![0]);
    assert_eq!(into_bytes(&ty, "1").unwrap(), vec![1, 1]);
    assert!(into_bytes(&ty, "a").is_err());
}

#[test]
fn test_u512() {
    let ty = NamedCLType::U512;
    assert_eq!(into_bytes(&ty, "0").unwrap(), vec![0]);
    assert_eq!(into_bytes(&ty, "1").unwrap(), vec![1, 1]);
    assert!(into_bytes(&ty, "a").is_err());
}

#[test]
fn test_string() {
    let ty = NamedCLType::String;
    assert_eq!(
        into_bytes(&ty, "a").unwrap(),
        vec![1, 0, 0, 0, 97]
    );
    assert_eq!(
        into_bytes(&ty, "abc").unwrap(),
        vec![3, 0, 0, 0, 97, 98, 99]
    );
}

#[test]
fn test_key() {
    let ty = NamedCLType::Key;
    let value = "hash-000102030405060708090a0b0c0d0e0f000102030405060708090a0b0c0d0e0f";
    let expected = vec![
        1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
        11, 12, 13, 14, 15,
    ];
    assert_eq!(into_bytes(&ty, value).unwrap(), expected);

    let value = "account-hash-000102030405060708090a0b0c0d0e0f000102030405060708090a0b0c0d0e0f";
    let expected = vec![
        0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
        11, 12, 13, 14, 15,
    ];
    assert_eq!(into_bytes(&ty, value).unwrap(), expected);
}

#[test]
fn test_uref() {
    let ty = NamedCLType::URef;
    let value = "uref-000102030405060708090a0b0c0d0e0f000102030405060708090a0b0c0d0e0f-007";
    let expected = vec![
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
        11, 12, 13, 14, 15, 7,
    ];
    assert_eq!(into_bytes(&ty, value).unwrap(), expected);
}

#[test]
fn test_public_key() {
    let ty = NamedCLType::PublicKey;
    // Use a valid Ed25519 public key hex string (01 tag + 32 zero bytes)
    let value = "010000000000000000000000000000000000000000000000000000000000000000";
    // Expected bytes: 0x01 tag followed by 32 zero bytes
    let expected = vec![
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
    ];
    assert_eq!(into_bytes(&ty, value).unwrap(), expected);
}

#[test]
fn test_option() {
    let ty = NamedCLType::Option(Box::new(NamedCLType::U8));
    assert_eq!(into_bytes(&ty, "none").unwrap(), vec![0]);
    assert_eq!(into_bytes(&ty, "some:1").unwrap(), vec![1, 1]);
    assert_eq!(
        into_bytes(&ty, "a"),
        Err(Error::Formatting(Format::Option))
    );
}

#[test]
fn test_result() {
    let ty = NamedCLType::Result {
        ok: Box::new(NamedCLType::U8),
        err: Box::new(NamedCLType::String),
    };
    assert_eq!(into_bytes(&ty, "ok:1").unwrap(), vec![1, 1]);
    assert_eq!(
        into_bytes(&ty, r#"err:a"#).unwrap(),
        vec![0, 1, 0, 0, 0, 97]
    );
    assert_eq!(
        into_bytes(&ty, "a"),
        Err(Error::Formatting(Format::Result))
    );
}

#[test]
fn test_tuple1() {
    let ty = NamedCLType::Tuple1([Box::new(NamedCLType::U8)]);
    assert_eq!(into_bytes(&ty, "1").unwrap(), vec![1]);
}

#[test]
fn test_tuple2() {
    let ty = NamedCLType::Tuple2([
        Box::new(NamedCLType::U8),
        Box::new(NamedCLType::String),
    ]);
    assert_eq!(
        into_bytes(&ty, r#"1:a"#).unwrap(),
        vec![1, 1, 0, 0, 0, 97]
    );
}

#[test]
fn test_tuple3() {
    let ty = NamedCLType::Tuple3([
        Box::new(NamedCLType::U8),
        Box::new(NamedCLType::String),
        Box::new(NamedCLType::Bool),
    ]);
    assert_eq!(
        into_bytes(&ty, r#"1:a:true"#).unwrap(),
        vec![1, 1, 0, 0, 0, 97, 1]
    );
}

#[test]
fn test_unit() {
    let ty = NamedCLType::Unit;
    assert_eq!(into_bytes(&ty, "").unwrap(), Vec::<u8>::new());
}

#[test]
fn test_map() {
    let ty = NamedCLType::Map {
        key: Box::new(NamedCLType::U8),
        value: Box::new(NamedCLType::String),
    };
    assert_eq!(
        into_bytes(&ty, r#"1:a,2:b"#).unwrap(),
        vec![2, 0, 0, 0, 1, 1, 0, 0, 0, 97, 2, 1, 0, 0, 0, 98]
    );
}

#[test]
fn test_list() {
    let ty = NamedCLType::List(Box::new(NamedCLType::U8));
    assert_eq!(
        into_bytes(&ty, "1").unwrap(),
        vec![1, 0, 0, 0, 1]
    );

    let ty = NamedCLType::List(Box::new(NamedCLType::U8));
    assert_eq!(
        into_bytes(&ty, "1,2,3").unwrap(),
        vec![3, 0, 0, 0, 1, 2, 3]
    );
}

#[test]
fn test_byte_array() {
    let ty = NamedCLType::ByteArray(4);
    assert_eq!(into_bytes(&ty, "0x01020304").unwrap(), vec![1, 2, 3, 4]);
    assert_eq!(into_bytes(&ty, "0x01").unwrap(), vec![1, 1, 1, 1]);
    assert_eq!(into_bytes(&ty, "0x0").unwrap(), vec![0, 0, 0, 0]);
    assert_eq!(
        into_bytes(&ty, "1,2,3,4").unwrap(),
        vec![1, 2, 3, 4]
    );
    assert_eq!(
        into_bytes(&ty, "0x01,0x02,0x03,0x04").unwrap(),
        vec![1, 2, 3, 4]
    );
    let ty = NamedCLType::ByteArray(16);
    assert_eq!(
        into_bytes(&ty, "0x010203"),
        Err(Error::Formatting(Format::PatternLength {
            actual: 3,
            expected: 16
        }))
    );
}

#[test]
fn test_parse_hex_isolated() {
    assert_eq!(super::parse_hex("0x01").unwrap(), vec![1]);
    assert_eq!(super::parse_hex("0xdeadbeef").unwrap(), vec![0xde, 0xad, 0xbe, 0xef]);
    assert!(super::parse_hex("0x").is_err()); // Empty hex string after prefix
    assert!(super::parse_hex("0xG").is_err()); // Invalid hex char
    assert!(super::parse_hex("01").is_err()); // No 0x prefix
}