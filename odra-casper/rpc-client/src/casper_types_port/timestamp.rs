extern crate alloc;
use alloc::vec::Vec;
use chrono::{DateTime, Utc};
use core::{
    ops::{Add, AddAssign, Div, Mul, Rem, Shl, Shr, Sub, SubAssign},
    time::Duration
};
use odra_core::casper_types::bytesrepr;
use odra_core::casper_types::bytesrepr::{FromBytes, ToBytes};
use std::str::FromStr;
use std::time::SystemTime;
use std::u64;

use datasize::DataSize;
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, DataSize, JsonSchema,
)]
pub struct Timestamp(u64);

impl Timestamp {
    /// The maximum value a timestamp can have.
    pub const MAX: Timestamp = Timestamp(u64::MAX);

    /// Returns a zero timestamp.
    pub fn zero() -> Self {
        Timestamp(0)
    }

    /// Returns the timestamp as the number of milliseconds since the Unix epoch
    pub fn millis(&self) -> u64 {
        self.0
    }

    /// Returns the difference between `self` and `other`, or `0` if `self` is earlier than `other`.
    pub fn saturating_diff(self, other: Timestamp) -> TimeDiff {
        TimeDiff(self.0.saturating_sub(other.0))
    }

    /// Returns the difference between `self` and `other`, or `0` if that would be before the epoch.
    #[must_use]
    pub fn saturating_sub(self, other: TimeDiff) -> Timestamp {
        Timestamp(self.0.saturating_sub(other.0))
    }

    /// Returns the sum of `self` and `other`, or the maximum possible value if that would be
    /// exceeded.
    #[must_use]
    pub fn saturating_add(self, other: TimeDiff) -> Timestamp {
        Timestamp(self.0.saturating_add(other.0))
    }

    /// Returns the number of trailing zeros in the number of milliseconds since the epoch.
    pub fn trailing_zeros(&self) -> u8 {
        self.0.trailing_zeros() as u8
    }
}

impl FromStr for Timestamp {
    type Err = String;

    fn from_str(s: &str) -> Result<Timestamp, String> {
        let system_time = humantime::parse_rfc3339_weak(s).map_err(|e| e.to_string())?;
        let inner = system_time
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| e.to_string())?;
        Ok(Timestamp(inner.as_millis() as u64))
    }
}

impl alloc::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut alloc::fmt::Formatter) -> Result<(), alloc::fmt::Error> {
        let datetime: DateTime<Utc> =
            DateTime::from_timestamp(self.0 as i64 / 1000, (self.0 % 1000) as u32 * 1_000_000)
                .unwrap();
        write!(
            f,
            "{}",
            datetime.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
        )
    }
}

impl Add<TimeDiff> for Timestamp {
    type Output = Timestamp;

    fn add(self, diff: TimeDiff) -> Timestamp {
        Timestamp(self.0 + diff.0)
    }
}

impl AddAssign<TimeDiff> for Timestamp {
    fn add_assign(&mut self, rhs: TimeDiff) {
        self.0 += rhs.0;
    }
}

#[cfg(any(feature = "testing", test))]
impl Sub<TimeDiff> for Timestamp {
    type Output = Timestamp;

    fn sub(self, diff: TimeDiff) -> Timestamp {
        Timestamp(self.0 - diff.0)
    }
}

impl Rem<TimeDiff> for Timestamp {
    type Output = TimeDiff;

    fn rem(self, diff: TimeDiff) -> TimeDiff {
        TimeDiff(self.0 % diff.0)
    }
}

impl<T> Shl<T> for Timestamp
where
    u64: Shl<T, Output = u64>
{
    type Output = Timestamp;

    fn shl(self, rhs: T) -> Timestamp {
        Timestamp(self.0 << rhs)
    }
}

impl<T> Shr<T> for Timestamp
where
    u64: Shr<T, Output = u64>
{
    type Output = Timestamp;

    fn shr(self, rhs: T) -> Timestamp {
        Timestamp(self.0 >> rhs)
    }
}

impl Serialize for Timestamp {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            self.to_string().serialize(serializer)
        } else {
            self.0.serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            let value_as_string = String::deserialize(deserializer)?;
            Timestamp::from_str(&value_as_string).map_err(serde::de::Error::custom)
        } else {
            let inner = u64::deserialize(deserializer)?;
            Ok(Timestamp(inner))
        }
    }
}

impl ToBytes for Timestamp {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        self.0.to_bytes()
    }

    fn serialized_length(&self) -> usize {
        self.0.serialized_length()
    }
}

impl FromBytes for Timestamp {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        u64::from_bytes(bytes).map(|(inner, remainder)| (Timestamp(inner), remainder))
    }
}

impl From<u64> for Timestamp {
    fn from(milliseconds_since_epoch: u64) -> Timestamp {
        Timestamp(milliseconds_since_epoch)
    }
}

/// A time difference between two timestamps.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, DataSize, JsonSchema,
)]
#[schemars(with = "String", description = "Human-readable duration.")]
pub struct TimeDiff(u64);

impl TimeDiff {
    /// Returns the time difference as the number of milliseconds since the Unix epoch
    pub fn millis(&self) -> u64 {
        self.0
    }

    /// Creates a new time difference from seconds.
    pub const fn from_seconds(seconds: u32) -> Self {
        TimeDiff(seconds as u64 * 1_000)
    }

    /// Creates a new time difference from milliseconds.
    pub const fn from_millis(millis: u64) -> Self {
        TimeDiff(millis)
    }

    /// Returns the product, or `TimeDiff(u64::MAX)` if it would overflow.
    #[must_use]
    pub fn saturating_mul(self, rhs: u64) -> Self {
        TimeDiff(self.0.saturating_mul(rhs))
    }
}

impl FromStr for TimeDiff {
    type Err = String;

    fn from_str(s: &str) -> Result<TimeDiff, String> {
        let inner = humantime::parse_duration(s).map_err(|e| e.to_string())?;
        Ok(TimeDiff(inner.as_millis() as u64))
    }
}

impl alloc::fmt::Display for TimeDiff {
    fn fmt(&self, f: &mut alloc::fmt::Formatter) -> Result<(), alloc::fmt::Error> {
        let duration = Duration::from_millis(self.0);
        let hours = duration.as_secs() / 3600;
        let minutes = (duration.as_secs() % 3600) / 60;
        let seconds = duration.as_secs() % 60;
        let millis = duration.subsec_millis();

        if hours > 0 {
            write!(f, "{}h", hours)?;
        }
        if minutes > 0 {
            write!(f, "{}m", minutes)?;
        }
        if seconds > 0 {
            write!(f, "{}s", seconds)?;
        }
        if millis > 0 {
            write!(f, "{}ms", millis)?;
        }
        Ok(())
    }
}

impl Add<TimeDiff> for TimeDiff {
    type Output = TimeDiff;

    fn add(self, rhs: TimeDiff) -> TimeDiff {
        TimeDiff(self.0 + rhs.0)
    }
}

impl AddAssign<TimeDiff> for TimeDiff {
    fn add_assign(&mut self, rhs: TimeDiff) {
        self.0 += rhs.0;
    }
}

impl Sub<TimeDiff> for TimeDiff {
    type Output = TimeDiff;

    fn sub(self, rhs: TimeDiff) -> TimeDiff {
        TimeDiff(self.0 - rhs.0)
    }
}

impl SubAssign<TimeDiff> for TimeDiff {
    fn sub_assign(&mut self, rhs: TimeDiff) {
        self.0 -= rhs.0;
    }
}

impl Mul<u64> for TimeDiff {
    type Output = TimeDiff;

    fn mul(self, rhs: u64) -> TimeDiff {
        TimeDiff(self.0 * rhs)
    }
}

impl Div<u64> for TimeDiff {
    type Output = TimeDiff;

    fn div(self, rhs: u64) -> TimeDiff {
        TimeDiff(self.0 / rhs)
    }
}

impl Div<TimeDiff> for TimeDiff {
    type Output = u64;

    fn div(self, rhs: TimeDiff) -> u64 {
        self.0 / rhs.0
    }
}

impl From<TimeDiff> for Duration {
    fn from(diff: TimeDiff) -> Duration {
        Duration::from_millis(diff.0)
    }
}

impl Serialize for TimeDiff {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            let duration = Duration::from_millis(self.0);
            let hours = duration.as_secs() / 3600;
            let minutes = (duration.as_secs() % 3600) / 60;
            let seconds = duration.as_secs() % 60;
            let millis = duration.subsec_millis();

            let mut s = String::new();
            if hours > 0 {
                s.push_str(&format!("{}h", hours));
            }
            if minutes > 0 {
                s.push_str(&format!("{}m ", minutes));
            }
            if seconds > 0 {
                s.push_str(&format!("{}s", seconds));
            }
            if millis > 0 {
                s.push_str(&format!("{}ms", millis));
            }
            s.serialize(serializer)
        } else {
            self.0.serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for TimeDiff {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            let value_as_string = String::deserialize(deserializer)?;
            TimeDiff::from_str(&value_as_string).map_err(serde::de::Error::custom)
        } else {
            let inner = u64::deserialize(deserializer)?;
            Ok(TimeDiff(inner))
        }
    }
}

impl ToBytes for TimeDiff {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        self.0.to_bytes()
    }

    fn serialized_length(&self) -> usize {
        self.0.serialized_length()
    }
}

impl FromBytes for TimeDiff {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        u64::from_bytes(bytes).map(|(inner, remainder)| (TimeDiff(inner), remainder))
    }
}

impl From<Duration> for TimeDiff {
    fn from(duration: Duration) -> TimeDiff {
        TimeDiff(duration.as_millis() as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_serialization_roundtrip() {
        let timestamp = Timestamp::zero();

        let timestamp_as_string = timestamp.to_string();
        assert_eq!(
            timestamp,
            Timestamp::from_str(&timestamp_as_string).unwrap()
        );

        let serialized_json = serde_json::to_string(&timestamp).unwrap();
        assert_eq!(timestamp, serde_json::from_str(&serialized_json).unwrap());
    }

    #[test]
    fn timediff_serialization_roundtrip() {
        let timediff = TimeDiff(123);

        let timediff_as_string = timediff.to_string();
        assert_eq!(timediff, TimeDiff::from_str(&timediff_as_string).unwrap());

        let serialized_json = serde_json::to_string(&timediff).unwrap();
        assert_eq!(timediff, serde_json::from_str(&serialized_json).unwrap());
    }

    #[test]
    fn does_not_crash_for_big_timestamp_value() {
        assert!(odra_core::casper_types::Timestamp::MAX
            .to_string()
            .starts_with("Invalid timestamp:"));
    }
}
