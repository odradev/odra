pub trait StaticInstance: Sized {
    /// Returns an instance with the `namespace`.
    fn instance(keys: &'static [&'static str]) -> (Self, &'static [&'static str]);
}

pub trait DynamicInstance {
    fn instance(namespace: &[u8]) -> Self;
}
