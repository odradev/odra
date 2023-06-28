/// A trait that should be implemented by each smart contract to allow the backend
/// to instantiate a module.
///
/// It is the default method of an instance creation.
/// The namespace is passed from the top to the bottom, but probably you don't need to
/// implement it manually, the default implementation is provided by [odra::module](crate::module) macro.
///
/// # Example
///
/// ```
/// # use odra::StaticInstance;
/// # use odra::Variable;
///
/// struct Parent {
///     c1: C1,
///     c2: C2,
/// }
///
/// struct C1 {
///     value: Variable<u32>
/// }
///
/// impl StaticInstance for C1 {
///     fn instance(keys: &'static [&'static str]) -> (Self, &'static [&'static str]) {
///         let (value, keys) = StaticInstance::instance(keys);
///         (Self { value }, keys)
///     }
/// }
///
/// struct C2 {
///     value: Variable<u32>
/// }
///
/// impl StaticInstance for C2 {
///     fn instance(keys: &'static [&'static str]) -> (Self, &'static [&'static str]) {
///         let (value, keys) = StaticInstance::instance(keys);
///         (Self { value }, keys)
///     }
/// }
///
/// impl StaticInstance for Parent {
///     fn instance(keys: &'static [&'static str]) -> (Self, &'static [&'static str]) {
///         let (c1, keys) = StaticInstance::instance(keys);
///         let (c2, keys) = StaticInstance::instance(keys);
///         (Self { c1, c2 }, keys)
///     }
/// }
///
/// const KEYS: [&'static str; 2usize] = ["key1", "key2"];
/// let (parent, _): (Parent, _) = StaticInstance::instance(&KEYS);
/// ````
pub trait StaticInstance: Sized {
    /// Consumes keys required to create an instance, returns the instance with the remaining keys.
    fn instance(keys: &'static [&'static str]) -> (Self, &'static [&'static str]);
}

/// A trait that should be implemented by each smart contract to allow the backend
/// to instantiate a module.
///
/// This trait allows to take full control over instance creation.
///
/// # Example
///
/// ```
/// # use odra::DynamicInstance;
/// # use odra::Variable;
///
/// struct Parent {
///     c1: C1,
///     c2: C2,
/// }
///
/// struct C1 {
///     value: Variable<u32>
/// }
///
/// impl DynamicInstance for C1 {
///     fn instance(namespace: &[u8]) -> Self {
///        let mut buffer: Vec<u8> = Vec::with_capacity(namespace.len() + 1);
///        buffer.extend_from_slice(namespace);
///        buffer.extend_from_slice(b"v");
///        let value = DynamicInstance::instance(&buffer);
///        Self { value }
///     }
/// }
///
/// struct C2 {
///     value: Variable<u32>
/// }
///
/// impl DynamicInstance for C2 {
///     fn instance(namespace: &[u8]) -> Self {
///        let mut buffer: Vec<u8> = Vec::with_capacity(namespace.len() + 1);
///        buffer.extend_from_slice(namespace);
///        buffer.extend_from_slice(b"v");
///        let value = DynamicInstance::instance(&buffer);
///        Self { value }
///     }
/// }
///
/// impl DynamicInstance for Parent {
///     fn instance(namespace: &[u8]) -> Self {
///        let namespace_len = namespace.len();
///        let len = namespace_len + b"c1".len();
///        let mut buffer: Vec<u8> = Vec::with_capacity(len);
///
///        buffer.extend_from_slice(namespace);
///        buffer.extend_from_slice(b"c1");
///        let c1 = DynamicInstance::instance(&buffer);
///
///        buffer.clear();
///        buffer.extend_from_slice(namespace);
///        buffer.extend_from_slice(b"c2");
///        let c2 = DynamicInstance::instance(&buffer);
///
///        Self { c1, c2 }
///     }
/// }
///
/// let parent: Parent = DynamicInstance::instance(b"root");
/// ````
pub trait DynamicInstance {
    fn instance(namespace: &[u8]) -> Self;
}
