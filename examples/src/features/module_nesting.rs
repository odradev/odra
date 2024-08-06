//! This example demonstrates how to use nested Odra types in a contract.
use odra::casper_event_standard;
use odra::prelude::*;
use odra::{Mapping, SubModule, UnwrapOrRevert, Var};

/// Module containing the results' storage.
#[odra::module]
pub struct ResultsStorage {
    results: Mapping<u32, OperationResult>,
    results_count: Var<u32>
}

/// Contract that uses a module with nested Odra types.
#[odra::module(events = [OperationEnded])]
pub struct NestedOdraTypesContract {
    latest_result: Var<OperationResult>,
    current_generation_storage: SubModule<ResultsStorage>
}

#[odra::module]
impl NestedOdraTypesContract {
    /// Saves the operation result in the storage.
    pub fn save_operation_result(&mut self, operation_result: OperationResult) {
        // save it in simple storage
        self.latest_result.set(operation_result.clone());

        // save it in the mapping in a module
        let mut results_count = self
            .current_generation_storage
            .results_count
            .get_or_default();
        self.current_generation_storage
            .results
            .set(&results_count, operation_result.clone());
        results_count += 1;
        self.current_generation_storage
            .results_count
            .set(results_count);

        self.env().emit_event(OperationEnded {
            id: operation_result.id,
            status: operation_result.status,
            description: operation_result.description
        });
    }

    /// Returns the latest operation result.
    pub fn latest_result(&self) -> Option<OperationResult> {
        self.latest_result.get()
    }

    /// Returns the current generation of operation results.
    pub fn current_generation(&self) -> Vec<OperationResult> {
        let keys = self
            .current_generation_storage
            .results_count
            .get_or_default();
        let keys_range = 0..keys;
        keys_range
            .map(|key| {
                self.current_generation_storage
                    .results
                    .get(&key)
                    .unwrap_or_revert(self)
            })
            .collect()
    }
}

/// Status of the operation.
#[odra::odra_type]
pub enum Status {
    /// Operation failed.
    Failure,
    /// Operation succeeded.
    Success
}

#[odra::odra_type]
/// Result of the operation.
pub struct OperationResult {
    /// Id of the operation.
    pub id: u32,
    /// Status of the operation.
    pub status: Status,
    /// Description of the operation.
    pub description: String
}

/// Event emitted when the operation ends.
#[odra::event]
pub struct OperationEnded {
    id: u32,
    status: Status,
    description: String
}

#[cfg(test)]
mod tests {
    use odra::host::{Deployer, NoArgs};

    use super::*;

    // generate operation results
    fn operation_results() -> Vec<OperationResult> {
        vec![
            OperationResult {
                id: 1,
                status: Status::Success,
                description: "Operation performed successfully".to_string()
            },
            OperationResult {
                id: 2,
                status: Status::Failure,
                description: "Operation failed".to_string()
            },
            OperationResult {
                id: 3,
                status: Status::Success,
                description: "Operation performed beautifully".to_string()
            },
            OperationResult {
                id: 4,
                status: Status::Failure,
                description: "Operation failed, but started fine".to_string()
            },
            OperationResult {
                id: 5,
                status: Status::Success,
                description: "Operation performed successfully, but ugly".to_string()
            },
            OperationResult {
                id: 6,
                status: Status::Failure,
                description: "Operation performed so ugly, it failed".to_string()
            },
            OperationResult {
                id: 7,
                status: Status::Success,
                description: "Perfect operation".to_string()
            },
            OperationResult {
                id: 8,
                status: Status::Failure,
                description: "Subject escaped".to_string()
            },
        ]
    }

    #[test]
    fn nested_odra_types() {
        let test_env = odra_test::env();
        let mut nested_odra_types = NestedOdraTypesContract::deploy(&test_env, NoArgs);

        // Storage is not set
        assert_eq!(nested_odra_types.latest_result(), None);

        // Set the storage
        nested_odra_types.save_operation_result(operation_results()[0].clone());

        // Storage is saved properly, even with nested Odra Types
        assert_eq!(
            nested_odra_types.latest_result(),
            Some(operation_results()[0].clone())
        );

        // Events are also saved properly, even with nested Odra Types
        test_env.emitted_event(
            &nested_odra_types,
            &OperationEnded {
                id: operation_results()[0].clone().id,
                status: operation_results()[0].clone().status,
                description: operation_results()[0].clone().description
            }
        );
    }
}
