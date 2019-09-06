use std::fmt::{Debug, Error, Formatter};
use std::fs::File;

use base58;
use base58::ToBase58;
use ipfsapi::IpfsApi;
use multihash;
use regex::Regex;

use crate::errors::QFSError;
use crate::types::ipfs::IpfsHash;

static IPFS_HASH_PATTERN: &str = "^[a-zA-z0-9]{46}$";


pub fn validate_ipfs_hash(hash: &str) -> bool {
    Regex::new(IPFS_HASH_PATTERN).unwrap().is_match(hash)
}

pub fn hash_bytes(bytes: &[u8]) -> String {
    multihash::encode(multihash::Hash::SHA2256, bytes)
        .unwrap()
        .as_slice()
        .to_base58()
}

pub struct IPFS {
    api: IpfsApi
}

impl Debug for IPFS {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "IPFS server")
    }
}

impl IPFS {
    pub fn new(server: &str, port: u16) -> IPFS {
        Self {
            api: IpfsApi::new(server, port)
        }
    }

    pub fn stream(&self, ipfs_hash: &IpfsHash) -> Result<impl Iterator<Item=u8>, QFSError> {
        self.api
            .block_get(ipfs_hash.to_string().as_str())
            .map_err(QFSError::from)
    }

    pub fn fetch(&self, ipfs_hash: &IpfsHash) -> Result<Vec<u8>, QFSError> {
        let bytes = self.stream(ipfs_hash)?.collect();
        Ok(bytes)
    }

    pub fn add(&self, file: &File) -> Result<IpfsHash, QFSError> {
        self.api
            .block_put(file.try_clone()?)
            .map_err(QFSError::from)
            .map(|hash| IpfsHash::new(hash.as_str()).unwrap())
    }
}


#[cfg(test)]
mod tests {
    use crate::operations::ipfs::{hash_bytes, IPFS, IpfsHash, validate_ipfs_hash};

    fn ipfs() -> IPFS {
        IPFS::new("127.0.0.1", 5001)
    }

    #[test]
    fn validate_ipfs_hash_with_valid_hash_should_work() {
        let hash = "QmaozNR7DZHQK1ZcU9p7QdrshMvXqWK6gpu5rmrkPdT3L4";
        let result = validate_ipfs_hash(hash);
        assert!(result);
    }

    #[test]
    fn test_fetch_with_valid_hash_should_work() {
        let hash = IpfsHash::new("QmWE6s8qazNrzGEHLfVA5PAFieT1nsoqU11pggfoWwSis5").unwrap();
        let result = ipfs().fetch(&hash);
        assert!(result.is_ok());
        let content = String::from_utf8(result.unwrap()).unwrap();
        assert_eq!(content.as_str(), "Hello from IPFS Gateway Checker\n");
    }

    #[test]
    fn test_ipfs_hashing_should_work() {
        let result = hash_bytes(b"hello world");
        assert_eq!(result, "QmaozNR7DZHQK1ZcU9p7QdrshMvXqWK6gpu5rmrkPdT3L4");
    }
}
