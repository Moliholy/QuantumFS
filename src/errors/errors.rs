use std::fmt::{Display, Formatter};
use std::fmt;
use std::error::Error as StdError;
use failure::Error;

#[derive(Debug)]
pub struct QFSError {
    details: String
}

impl Display for QFSError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "QuantumFS Error: \"{}\"", &self.details)
    }
}

impl StdError for QFSError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

impl From<Error> for QFSError {
    fn from(err: Error) -> Self {
        QFSError { details: err.as_fail().to_string()}
    }
}

impl QFSError {
    pub fn new(details: &str) -> Self {
        Self { details: String::from(details) }
    }
}

