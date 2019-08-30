use regex::Regex;
use ipfsapi::IpfsApi;
use crate::errors::errors::QFSError;
use crate::types::ipfs::IpfsHash;

static IPFS_HASH_PATTERN: &str = "^[a-zA-z0-9]{46}$";
static IPFS_DEFAULT_URL: &str = "ipfs.io";
static IPFS_DEFAULT_PORT: u16 = 80;

fn api() -> IpfsApi {
    IpfsApi::new(IPFS_DEFAULT_URL, IPFS_DEFAULT_PORT)
}

pub fn validate_ipfs_hash(hash: &str) -> bool {
    Regex::new(IPFS_HASH_PATTERN).unwrap().is_match(hash)
}

pub fn stream(ipfs_hash: &IpfsHash) -> Result<impl Iterator<Item=u8>, QFSError> {
    api()
        .cat(ipfs_hash.to_string().as_str())
        .map_err(QFSError::from)
}

pub fn fetch(ipfs_hash: &IpfsHash) -> Result<Vec<u8>, QFSError> {
    let bytes = stream(ipfs_hash)?.collect();
    Ok(bytes)
}


#[cfg(test)]
mod tests {
    use crate::operations::ipfs::{validate_ipfs_hash, fetch, IpfsHash};

    #[test]
    fn validate_ipfs_hash_with_valid_hash_should_work() {
        let hash = "QmaozNR7DZHQK1ZcU9p7QdrshMvXqWK6gpu5rmrkPdT3L4";
        let result = validate_ipfs_hash(hash);
        assert!(result);
    }

    #[test]
    fn test_fetch_with_valid_hash_should_work() {
        let hash = IpfsHash::new("Qmaisz6NMhDB51cCvNWa1GMS7LU1pAxdF4Ld6Ft9kZEP2a").unwrap();
        let result = fetch(&hash);
        assert!(result.is_ok());
        let content = String::from_utf8(result.unwrap()).unwrap();
        assert_eq!(content.as_str(), "Hello from IPFS Gateway Checker\n");
    }
}
