use core::fmt;

use crate::errors::QFSError;
use crate::operations::ipfs::validate_ipfs_hash;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct IpfsHash {
    hash: String,
}

impl IpfsHash {
    pub fn new(hash: &str) -> Result<Self, QFSError> {
        match validate_ipfs_hash(hash) {
            true => Ok(Self { hash: String::from(hash) }),
            false => Err(QFSError::new("Invalid IPFS hash")),
        }
    }
}

impl AsRef<str> for IpfsHash {
    fn as_ref(&self) -> &str {
        self.hash.as_str()
    }
}

impl fmt::Display for IpfsHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.hash)
    }
}
