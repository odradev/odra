use casper_event_standard::Event;
use odra::casper_event_standard;
use odra::prelude::*;
use odra::{Mapping, Module, OdraType, SubModule, UnwrapOrRevert, Var};

#[odra::module]
pub struct ResultsStorage {
    results: Mapping<u32, OperationResult>,
    results_count: Var<u32>
}

#[odra::module]
pub struct NestedOdraTypesContract {
    latest_result: Var<OperationResult>,
    current_generation_storage: SubModule<ResultsStorage>
}

#[odra::module]
impl NestedOdraTypesContract {
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

    pub fn latest_result(&self) -> Option<OperationResult> {
        self.latest_result.get()
    }

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
                    .unwrap_or_revert(&self.env())
            })
            .collect()
    }
}

#[derive(OdraType, PartialEq, Debug)]
pub enum Status {
    Failure,
    Success
}

#[derive(OdraType, PartialEq, Debug)]
pub struct OperationResult {
    pub id: u32,
    pub status: Status,
    pub description: String
}

#[derive(Event, PartialEq, Debug)]
pub struct OperationEnded {
    id: u32,
    status: Status,
    description: String
}

#[cfg(test)]
mod tests {
    use odra::host::{Deployer, HostRef, NoInit};

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
        let mut nested_odra_types = NestedOdraTypesContractHostRef::deploy(&test_env, NoInit);

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
            nested_odra_types.address(),
            &OperationEnded {
                id: operation_results()[0].clone().id,
                status: operation_results()[0].clone().status,
                description: operation_results()[0].clone().description
            }
        );
    }
}
