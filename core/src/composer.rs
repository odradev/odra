use crate::DynamicInstance;

/// A struct that can be used to compose an instance.
pub struct Composer {
    namespace: Vec<u8>
}
impl Composer {
    /// Creates a new composer with the `namespace` and `name`.
    pub fn new(root_namespace: &[u8], name: &str) -> Self {
        let mut namespace = Vec::with_capacity(root_namespace.len() + name.len() + 1);
        namespace.extend_from_slice(root_namespace);
        namespace.extend_from_slice(b"#");
        namespace.extend_from_slice(name.as_bytes());
        Self { namespace }
    }

    /// Builds an instance with the `namespace`.
    pub fn compose<T: DynamicInstance>(self) -> T {
        T::instance(&self.namespace)
    }
}
