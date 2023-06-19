use crate::DynamicInstance;

/// A struct that can be used to compose an instance.
pub struct Composer {
    namespace: String
}
impl Composer {
    /// Creates a new composer with the `namespace` and `name`.
    pub fn new(namespace: &str, name: &str) -> Self {
        Self {
            namespace: format!("{}_{}", namespace, name)
        }
    }

    /// Builds an instance with the `namespace`.
    pub fn compose<T: DynamicInstance>(self) -> T {
        T::instance(self.namespace.as_bytes())
    }
}
