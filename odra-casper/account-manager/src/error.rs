use casper_types::{
    account::{AddKeyFailure, RemoveKeyFailure, SetThresholdFailure, UpdateKeyFailure},
    ApiError
};

/// The reasons the account manager wasm reverts with, as `ApiError::User(code)`.
///
/// Each maps one failure of the execution engine's key management API, or one invalid argument,
/// to its own code, so a failed transaction says which step failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum Error {
    /// The arguments ask for no change at all.
    EmptyChange = 1,
    /// A key in `set_keys` has weight 0; remove it with `remove_keys` instead.
    ZeroWeight = 2,
    /// A threshold of 0 was requested.
    ZeroThreshold = 3,
    /// A key is both in `set_keys` and in `remove_keys`, or twice in one of them.
    ConflictingKey = 4,
    /// Adding a key would exceed the maximum number of associated keys.
    MaxKeysLimit = 10,
    /// The signers' weight does not reach the `key_management` threshold (adding a key, or
    /// changing the weight of one in `set_keys`: the wasm tries to add it first).
    AddKeyPermissionDenied = 11,
    /// The key to update is not an associated key.
    UpdateMissingKey = 20,
    /// The signers' weight does not reach the `key_management` threshold (updating a key).
    UpdateKeyPermissionDenied = 21,
    /// Lowering the key's weight would take the total weight below a threshold.
    UpdateKeyThresholdViolation = 22,
    /// The key to remove is not an associated key.
    RemoveMissingKey = 30,
    /// The signers' weight does not reach the `key_management` threshold (removing a key).
    RemoveKeyPermissionDenied = 31,
    /// Removing the key would take the total weight below a threshold.
    RemoveKeyThresholdViolation = 32,
    /// The `key_management` threshold would be below the `deployment` threshold.
    KeyManagementBelowDeployment = 40,
    /// The `deployment` threshold would be above the `key_management` threshold.
    DeploymentAboveKeyManagement = 41,
    /// The signers' weight does not reach the `key_management` threshold (setting a threshold).
    SetThresholdPermissionDenied = 42,
    /// A threshold would be above the total weight of all associated keys.
    ThresholdAboveTotalWeight = 43,
    /// The execution engine reported a failure this version does not know.
    UnknownFailure = 99
}

impl Error {
    /// The code the wasm reverts with (`ApiError::User`).
    pub const fn code(self) -> u16 {
        self as u16
    }

    /// The error a reverted transaction's `ApiError::User(code)` stands for.
    pub fn from_code(code: u16) -> Option<Self> {
        ALL.iter().copied().find(|error| error.code() == code)
    }

    /// The error for a failed `add_associated_key`. The wasm never reports `DuplicateKey`: it
    /// updates the existing key instead.
    pub fn from_add_key_failure(failure: AddKeyFailure) -> Self {
        match failure {
            AddKeyFailure::MaxKeysLimit => Error::MaxKeysLimit,
            AddKeyFailure::DuplicateKey => Error::ConflictingKey,
            AddKeyFailure::PermissionDenied => Error::AddKeyPermissionDenied,
            _ => Error::UnknownFailure
        }
    }

    /// The error for a failed `update_associated_key`.
    pub fn from_update_key_failure(failure: UpdateKeyFailure) -> Self {
        match failure {
            UpdateKeyFailure::MissingKey => Error::UpdateMissingKey,
            UpdateKeyFailure::PermissionDenied => Error::UpdateKeyPermissionDenied,
            UpdateKeyFailure::ThresholdViolation => Error::UpdateKeyThresholdViolation,
            _ => Error::UnknownFailure
        }
    }

    /// The error for a failed `remove_associated_key`.
    pub fn from_remove_key_failure(failure: RemoveKeyFailure) -> Self {
        match failure {
            RemoveKeyFailure::MissingKey => Error::RemoveMissingKey,
            RemoveKeyFailure::PermissionDenied => Error::RemoveKeyPermissionDenied,
            RemoveKeyFailure::ThresholdViolation => Error::RemoveKeyThresholdViolation,
            _ => Error::UnknownFailure
        }
    }

    /// The error for a failed `set_action_threshold`.
    pub fn from_set_threshold_failure(failure: SetThresholdFailure) -> Self {
        match failure {
            SetThresholdFailure::KeyManagementThreshold => Error::KeyManagementBelowDeployment,
            SetThresholdFailure::DeploymentThreshold => Error::DeploymentAboveKeyManagement,
            SetThresholdFailure::PermissionDeniedError => Error::SetThresholdPermissionDenied,
            SetThresholdFailure::InsufficientTotalWeight => Error::ThresholdAboveTotalWeight,
            _ => Error::UnknownFailure
        }
    }

    /// A one-line description for a person reading a failed transaction.
    pub const fn description(self) -> &'static str {
        match self {
            Error::EmptyChange => "the arguments ask for no change",
            Error::ZeroWeight => "a key to set has weight 0; remove it instead",
            Error::ZeroThreshold => "a threshold of 0 was requested",
            Error::ConflictingKey => "a key is listed twice, or both set and removed",
            Error::MaxKeysLimit => "the account would have too many associated keys",
            Error::AddKeyPermissionDenied => {
                "adding or updating a key needs signers whose weight reaches key_management"
            }
            Error::UpdateMissingKey => "the key to update is not an associated key",
            Error::UpdateKeyPermissionDenied => {
                "updating a key needs signers whose weight reaches key_management"
            }
            Error::UpdateKeyThresholdViolation => {
                "lowering the weight would take the total below a threshold"
            }
            Error::RemoveMissingKey => "the key to remove is not an associated key",
            Error::RemoveKeyPermissionDenied => {
                "removing a key needs signers whose weight reaches key_management"
            }
            Error::RemoveKeyThresholdViolation => {
                "removing the key would take the total below a threshold"
            }
            Error::KeyManagementBelowDeployment => "key_management would be below deployment",
            Error::DeploymentAboveKeyManagement => "deployment would be above key_management",
            Error::SetThresholdPermissionDenied => {
                "setting a threshold needs signers whose weight reaches key_management"
            }
            Error::ThresholdAboveTotalWeight => {
                "a threshold would be above the total weight of all keys"
            }
            Error::UnknownFailure => "the execution engine reported an unknown failure"
        }
    }
}

const ALL: [Error; 17] = [
    Error::EmptyChange,
    Error::ZeroWeight,
    Error::ZeroThreshold,
    Error::ConflictingKey,
    Error::MaxKeysLimit,
    Error::AddKeyPermissionDenied,
    Error::UpdateMissingKey,
    Error::UpdateKeyPermissionDenied,
    Error::UpdateKeyThresholdViolation,
    Error::RemoveMissingKey,
    Error::RemoveKeyPermissionDenied,
    Error::RemoveKeyThresholdViolation,
    Error::KeyManagementBelowDeployment,
    Error::DeploymentAboveKeyManagement,
    Error::SetThresholdPermissionDenied,
    Error::ThresholdAboveTotalWeight,
    Error::UnknownFailure
];

impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        ApiError::User(error.code())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_unique_and_round_trip() {
        for (i, error) in ALL.iter().enumerate() {
            assert_eq!(Error::from_code(error.code()), Some(*error));
            for other in &ALL[i + 1..] {
                assert_ne!(error.code(), other.code());
            }
        }
        assert_eq!(Error::from_code(0), None);
    }
}
