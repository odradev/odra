pub trait Event {
    fn emit(&self);
    fn name(&self) -> &str;
}
