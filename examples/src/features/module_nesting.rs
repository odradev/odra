use alloc::{string::String, vec::Vec};
use odra::contract_env::emit_event;
use odra::{Event, List, Mapping, OdraType, UnwrapOrRevert, Variable};

#[odra::module]
pub struct ResultsStorage {
    results: Mapping<u32, OperationResult>,
    results_count: Variable<u32>
}

#[odra::module]
pub struct NestedOdraTypesContract {
    latest_result: Variable<OperationResult>,
    current_generation_storage: ResultsStorage,
    past_generations_storage: List<ResultsStorage>
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

        emit_event(OperationEnded {
            result: operation_result
        });
    }

    pub fn new_generation(&mut self) {
        // move current generation to past generations
        let keys = self
            .current_generation_storage
            .results_count
            .get_or_default();
        let keys_range = 0..keys;
        let mut past_generation = self.past_generations_storage.next_instance();

        keys_range.for_each(|key| {
            let current_result = self
                .current_generation_storage
                .results
                .get(&key)
                .unwrap_or_revert();
            past_generation.results.set(&key, current_result);
            past_generation.results_count.set(key + 1);
        });

        self.current_generation_storage.results_count.set(0);
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
                    .unwrap_or_revert()
            })
            .collect()
    }

    pub fn past_generations(&mut self) -> Vec<Vec<OperationResult>> {
        let keys = self.past_generations_storage.len();
        let keys_range = 0..keys;
        keys_range
            .map(|key| {
                let generation = self.past_generations_storage.get_instance(key);
                let keys = generation.results_count.get_or_default();
                let keys_range = 0..keys;
                keys_range
                    .map(|key| generation.results.get(&key).unwrap_or_revert())
                    .collect()
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
    id: u32,
    status: Status,
    description: String
}

#[derive(Event, PartialEq, Debug)]
pub struct OperationEnded {
    result: OperationResult
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alloc::string::ToString;
    use odra::assert_events;

    // generate operation results
    fn operation_results() -> Vec<OperationResult> {
        alloc::vec![
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
        let mut nested_odra_types = NestedOdraTypesContractDeployer::default();

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
        assert_events!(
            nested_odra_types,
            OperationEnded {
                result: operation_results()[0].clone()
            }
        );
    }

    #[test]
    fn module_in_mapping_in_mapping() {
        let mut nested_odra_types = NestedOdraTypesContractDeployer::default();
        let operations = operation_results();
        // fill the storage
        nested_odra_types.save_operation_result(operations[0].clone());
        nested_odra_types.save_operation_result(operations[1].clone());
        nested_odra_types.save_operation_result(operations[2].clone());

        // create a new generation
        nested_odra_types.new_generation();

        // fill the storage
        nested_odra_types.save_operation_result(operations[3].clone());
        nested_odra_types.save_operation_result(operations[4].clone());
        nested_odra_types.save_operation_result(operations[5].clone());

        // create a new generation
        nested_odra_types.new_generation();

        // fill the storage last time
        nested_odra_types.save_operation_result(operations[6].clone());
        nested_odra_types.save_operation_result(operations[7].clone());

        // check the storage
        assert_eq!(
            nested_odra_types.latest_result(),
            Some(operations[7].clone())
        );
        assert_eq!(
            nested_odra_types.current_generation(),
            alloc::vec![operations[6].clone(), operations[7].clone(),]
        );

        assert_eq!(
            nested_odra_types.past_generations(),
            alloc::vec![
                alloc::vec![
                    operations[0].clone(),
                    operations[1].clone(),
                    operations[2].clone()
                ],
                alloc::vec![
                    operations[3].clone(),
                    operations[4].clone(),
                    operations[5].clone()
                ]
            ]
        );
    }
}
