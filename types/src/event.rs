pub trait Event {
    fn emit(&self);
    fn name(&self) -> String;
}

#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub enum EventError {
    UnexpectedType(String),
    IndexOutOfBounds,
    Formatting,
    Parsing,
}

impl From<casper_types::bytesrepr::Error> for EventError {
    fn from(err: casper_types::bytesrepr::Error) -> Self {
        match err {
            casper_types::bytesrepr::Error::Formatting => Self::Formatting,
            _ => Self::Parsing,
        }
    }
}
