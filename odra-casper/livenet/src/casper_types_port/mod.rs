pub mod timestamp;
// use casper_types::Timestamp;
// use serde::{Deserialize, Serialize};
//
// pub struct TimestampWrapper(pub Timestamp);
//
// impl Serialize for TimestampWrapper {
//     fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
//         self.0.millis().serialize(serializer)
//     }
// }
//
// impl<'de> Deserialize<'de> for TimestampWrapper {
// fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
//         let millis = u64::deserialize(deserializer)?;
//         Ok(TimestampWrapper(Timestamp::from(millis)))
//     }
// }