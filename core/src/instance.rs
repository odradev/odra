/// A trait that should be implemented by each smart contract to allow the backend
/// to instantiate a module.
/// 
/// The namespace is passed from the top to the bottom, but probably you don't need to
/// implement it manually, the default implementation is provided by [odra::module](crate::module) macro.
/// 
/// # Example
/// 
/// ```
/// # use odra::Instance;
/// # use odra::Variable;
/// 
/// struct Parent {
///     c1: C1,
///     c2: C2,
///     
/// }
/// 
/// struct C1 {
///     value: Variable<u32>
/// }
/// 
/// struct C2 {
///     value: Variable<u32>
/// }
/// 
/// impl Instance for Parent {
///     fn instance(namespace: &str) -> Self {
///         Self {
///             c1: Instance::instance(&format!("{}_{}", namespace, "c1")),
///             c2: Instance::instance(&format!("{}_{}", namespace, "c2")),
///         }
///     }
/// }
/// 
/// impl Instance for C1 {
///     fn instance(namespace: &str) -> Self {
///         Self {
///             value: Instance::instance(&format!("{}_{}", namespace, "value")),
///         }
///     }
/// }
/// 
/// impl Instance for C2 {
///     fn instance(namespace: &str) -> Self {
///         Self {
///             value: Instance::instance(&format!("{}_{}", namespace, "value")),
///         }
///     }
/// }
/// 
/// let parent: Parent = Instance::instance("parent");
/// 
/// // then the underlying variable will have unique namespaces
/// // "parent_c1_value" and "parent_c2_value" respectively.
/// ```
pub trait Instance {
    /// Returns an instance with the `namespace`.
    fn instance(namespace: &str) -> Self;
}
