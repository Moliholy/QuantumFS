use crate::operations::ipfs::IpfsHash;

#[readonly::make]
#[derive(Debug)]
pub struct Manifest {
    hash: IpfsHash,
    timestamp: u64,
}

impl Manifest {
    pub fn new(hash: IpfsHash, timestamp: u64) -> Self {
        Self { hash , timestamp }
    }
}


#[cfg(test)]
mod tests {
    use crate::models::manifest::Manifest;
    use std::time::SystemTime;
    use crate::operations::ipfs::IpfsHash;
    use crate::errors::errors::QFSError;

    fn instantiate(hash: &str) -> Result<Manifest, QFSError> {
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        let hash = IpfsHash::new(hash)?;
        Ok(Manifest::new(hash, now))
    }

    #[test]
    fn manifest_instantiation_with_correct_hash_should_work() {
        let hash = "QmaozNR7DZHQK1ZcU9p7QdrshMvXqWK6gpu5rmrkPdT3L4";
        let manifest = instantiate(hash);
        assert!(manifest.is_ok());
    }

    #[test]
    fn manifest_instantiation_with_incorrect_hash_should_fail() {
        let hash = "invalidhash";
        let manifest = instantiate(hash);
        assert!(manifest.is_err());
    }
}
