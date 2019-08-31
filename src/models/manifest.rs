use web3::types::Address;

use crate::models::revision::RevisionTag;
use crate::operations::ethereum;
use crate::errors::errors::QFSError;

#[derive(Debug)]
pub struct Manifest {
    address: Address
}

impl Manifest {
    pub fn new(address: Address) -> Self {
        Self { address }
    }

    pub fn fetch_revision(&self, revision: u128) -> Result<RevisionTag, QFSError> {
        let (hash, revision) = ethereum::fetch_revision(self.address, revision)?;
        Ok(RevisionTag::new(hash, revision))
    }

    pub fn fetch_last_revision(&self) -> Result<RevisionTag, QFSError> {
        let (hash, revision) = ethereum::fetch_last_revision(self.address)?;
        Ok(RevisionTag::new(hash, revision))
    }
}


#[cfg(test)]
mod tests {
    use crate::models::manifest::Manifest;
    use crate::operations::ethereum;

    #[test]
    fn manifest_instantiation_should_work() {
        let manifest = Manifest::new(ethereum::coinbase());
        let tag = manifest.fetch_last_revision().unwrap();
        assert_eq!(tag.hash().to_string().as_str(), "0000000000000000000000000000000000000000000000");
        assert_eq!(tag.revision(), 0);
    }

    #[test]
    fn manifest_instantiation_with_invalid_revision_should_fail() {
        let manifest = Manifest::new(ethereum::coinbase());
        let tag = manifest.fetch_revision(100000);
        assert!(tag.is_err());
    }
}
