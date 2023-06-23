use crate::DynamicInstance;

/// A struct that can be used to compose an instance.
pub struct Composer {
    namespace: Vec<u8>
}
impl Composer {
    /// Creates a new composer with the `namespace` and `name`.
    pub fn new(namespace: &[u8], name: &str) -> Self {
        Self {
            namespace: [namespace, "#".as_bytes(), name.as_bytes()].concat()
        }
    }

    /// Builds an instance with the `namespace`.
    pub fn compose<T: DynamicInstance>(self) -> T {
        T::instance(&self.namespace)
    }
}
