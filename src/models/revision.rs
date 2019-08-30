use crate::models::repository::Repository;
use crate::types::ipfs::IpfsHash;

#[derive(Debug)]
pub struct RevisionTag {
    hash: IpfsHash,
    revision: u128,
}

impl RevisionTag {
    pub fn new(hash: IpfsHash, revision: u128) -> Self {
        Self {
            hash,
            revision
        }
    }

    pub fn hash(&self) -> &IpfsHash {
        &self.hash
    }

    pub fn revision(&self) -> u128 {
        self.revision
    }
}

#[derive(Debug)]
pub struct Revision {
    repository: &'static Repository,
    tag: RevisionTag
}

impl Revision {
    pub fn new(repository: &'static Repository, tag: RevisionTag) -> Self {
        Self {
            repository,
            tag
        }
    }

    pub fn hash(&self) -> &IpfsHash {
        &self.tag.hash
    }

    pub fn revision(&self) -> u128 {
        self.tag.revision
    }
}
