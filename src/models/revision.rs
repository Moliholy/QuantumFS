use crate::models::catalog::Catalog;
use crate::models::repository::Repository;
use crate::types::ipfs::IpfsHash;
use crate::models::directoryentry::DirectoryEntry;

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
    repository: &'static mut Repository,
    tag: RevisionTag
}

impl Revision {
    pub fn new(repository: &'static mut Repository, tag: RevisionTag) -> Self {
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

    pub fn lookup(&mut self, path: &str) -> DirectoryEntry {
        let mut path = path;
        if path == "/" {
            path = "";
        }
        let best_fit = self.retrieve_catalog_for_path(path);
        best_fit.find_directory_entry(path)
    }

    pub fn retrieve_catalog(&mut self, hash: &IpfsHash) -> &Catalog {
        self.repository.retrieve_catalog(hash)
    }

    pub fn retrieve_root_catalog(&mut self) -> &Catalog {
        let hash = self.hash().clone();
        self.retrieve_catalog(&hash)
    }

    pub fn retrieve_catalog_for_path(&mut self, path: &str) -> &Catalog {
        let mut hash = self.hash().clone();
        loop {
            match self.retrieve_catalog(&hash).find_nested_for_path(path) {
                None => return self.repository.get_opened_catalog(&hash).unwrap(),
                Some(nested_reference) => hash = nested_reference.hash().clone()
            };
        }
    }
}
