pub trait Event {
    fn emit(&self);
    fn name(&self) -> String;
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum Error {
    UnexpectedType(String),
    IndexOutOfBounds,
    Formatting,
    Parsing,
}

impl From<casper_types::bytesrepr::Error> for Error {
    fn from(err: casper_types::bytesrepr::Error) -> Self {
        match err {
            casper_types::bytesrepr::Error::Formatting => Self::Formatting,
            _ => Self::Parsing,
        }
    }
}
