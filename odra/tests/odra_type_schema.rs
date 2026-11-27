//! What the schema of an `#[odra::odra_type]` says matches what the type is on the wire: an
//! enum variant's discriminant in the schema is the tag byte its value is serialized with, and
//! the custom types the fields of the type use are listed with it.
#![cfg(not(target_arch = "wasm32"))]
#![allow(missing_docs, dead_code)]

use odra::casper_types::bytesrepr::ToBytes;
use odra::prelude::*;
use odra::schema::casper_contract_schema::CustomType;
use odra::schema::SchemaCustomTypes;

#[odra::odra_type]
pub struct Point {
    pub x: u32,
    pub y: u32
}

#[odra::odra_type]
pub enum Color {
    Red = 1,
    Green = 2
}

/// Explicit discriminants on an enum with data (allowed with `#[repr]`).
#[odra::odra_type]
#[repr(u8)]
pub enum Shape {
    Square = 3,
    Circle(u32) = 5,
    Dot { at: Point } = 9
}

/// Variants carrying other Odra types, in a tuple and in named fields.
#[odra::odra_type]
pub enum Paint {
    Solid(Color),
    Stroke { from: Point, to: Point },
    None
}

fn schema_of<T: SchemaCustomTypes>(name: &str) -> CustomType {
    T::schema_types()
        .into_iter()
        .flatten()
        .find(|ty| custom_type_name(ty) == name)
        .unwrap_or_else(|| panic!("{name} is not in the schema"))
}

fn custom_type_name(ty: &CustomType) -> String {
    match ty {
        CustomType::Struct { name, .. } | CustomType::Enum { name, .. } => name.0.clone()
    }
}

fn names<T: SchemaCustomTypes>() -> BTreeSet<String> {
    T::schema_types()
        .into_iter()
        .flatten()
        .map(|ty| custom_type_name(&ty))
        .collect()
}

#[test]
fn enum_with_data_discriminants_are_the_wire_tags() {
    let CustomType::Enum { variants, .. } = schema_of::<Shape>("Shape") else {
        panic!("Shape is not an enum in the schema");
    };
    let values = [
        Shape::Square,
        Shape::Circle(7),
        Shape::Dot {
            at: Point { x: 1, y: 2 }
        }
    ];
    for (variant, value) in variants.iter().zip(values) {
        let tag = value.to_bytes().unwrap()[0];
        assert_eq!(
            variant.discriminant,
            u16::from(tag),
            "{}: the schema's discriminant is not the tag on the wire",
            variant.name
        );
    }
}

#[test]
fn unit_enum_discriminants_are_the_wire_tags() {
    let CustomType::Enum { variants, .. } = schema_of::<Color>("Color") else {
        panic!("Color is not an enum in the schema");
    };
    for (variant, value) in variants.iter().zip([Color::Red, Color::Green]) {
        assert_eq!(
            variant.discriminant,
            u16::from(value.to_bytes().unwrap()[0])
        );
    }
}

#[test]
fn enum_variant_field_types_are_in_the_schema() {
    let paint = names::<Paint>();
    for name in ["Paint", "Paint::Stroke", "Color", "Point"] {
        assert!(paint.contains(name), "{name} is missing from {paint:?}");
    }
    assert!(names::<Shape>().contains("Point"));
}
