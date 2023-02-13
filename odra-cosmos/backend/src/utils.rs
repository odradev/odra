use odra_cosmos_types::AsString;

use crate::runtime::RT;

pub fn add_attribute<T: AsString>(key: &str, value: T) {
    RT.with(|runtime| {
        runtime
            .borrow_mut()
            .add_attribute(key.to_string(), value.as_string())
    });
}
