use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result};
use std::io::Error as IOError;

use failure::Error as FailureError;
use serde_json::Error as SerdeError;
use sqlite::Error as SqliteError;
use web3::contract::Error as ContractError;

#[derive(Debug)]
pub struct QFSError {
    details: String
}

impl Display for QFSError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "QuantumFS Error: \"{}\"", &self.details)
    }
}

impl StdError for QFSError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

impl From<SqliteError> for QFSError {
    fn from(err: SqliteError) -> Self {
        QFSError { details: format!("{}", err) }
    }
}

impl From<FailureError> for QFSError {
    fn from(err: FailureError) -> Self {
        QFSError { details: format!("{}", err) }
    }
}

impl From<ContractError> for QFSError {
    fn from(err: ContractError) -> Self {
        QFSError { details: format!("{}", err) }
    }
}

impl From<IOError> for QFSError {
    fn from(err: IOError) -> Self {
        QFSError { details: format!("{}", err) }
    }
}

impl From<SerdeError> for QFSError {
    fn from(err: SerdeError) -> Self {
        QFSError { details: format!("{}", err) }
    }
}

impl QFSError {
    pub fn new(details: &str) -> Self {
        Self { details: String::from(details) }
    }
}

