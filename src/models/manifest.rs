use crate::types::ipfs::IpfsHash;

#[derive(Debug)]
pub struct Manifest {
    hash: IpfsHash,
    revision: u128
}

impl Manifest {
    pub fn new(hash: IpfsHash, revision: u128) -> Self {
        Self { hash, revision }
    }
}


#[cfg(test)]
mod tests {
    use crate::models::manifest::Manifest;
    use crate::types::ipfs::IpfsHash;
    use crate::errors::errors::QFSError;

    fn instantiate(hash: &str) -> Result<Manifest, QFSError> {
        let hash = IpfsHash::new(hash)?;
        Ok(Manifest::new(hash, 1))
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
