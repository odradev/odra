use odra::contract_env::emit_event;
use odra::{Event, OdraType, Variable};

#[odra::module]
pub struct NestedOdraTypes {
    operation_result: Variable<OperationResult>
}

#[odra::module]
impl NestedOdraTypes {
    pub fn perform_operation(&mut self) {
        let operation_result = OperationResult {
            status: Status::Success,
            description: "Operation performed successfully".to_string()
        };
        self.operation_result.set(operation_result.clone());

        emit_event(OperationEnded {
            result: operation_result
        });
    }

    pub fn get_operation_result(&self) -> Option<OperationResult> {
        self.operation_result.get()
    }
}

#[derive(OdraType, PartialEq, Debug)]
pub enum Status {
    Failure,
    Success
}

#[derive(OdraType, PartialEq, Debug)]
pub struct OperationResult {
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
    use odra::{assert_events, test_env, OdraItem};

    #[test]
    fn nested_odra_types() {
        let mut nested_odra_types = NestedOdraTypesDeployer::default();

        // Storage is not set
        assert_eq!(nested_odra_types.get_operation_result(), None);

        // Set the storage
        nested_odra_types.perform_operation();

        // Storage is saved properly, even with nested Odra Types
        assert_eq!(
            nested_odra_types.get_operation_result(),
            Some(OperationResult {
                status: Status::Success,
                description: "Operation performed successfully".to_string()
            })
        );

        // Events are also saved properly, even with nested Odra Types
        assert_events!(
            nested_odra_types,
            OperationEnded {
                result: OperationResult {
                    status: Status::Success,
                    description: "Operation performed successfully".to_string()
                }
            }
        );
    }
}
