use crate::errors::QFSError;
use crate::models::catalog::Catalog;
use crate::models::directoryentry::DirectoryEntry;
use crate::models::repository::Repository;
use crate::operations::path;
use crate::types::ipfs::IpfsHash;
use crate::operations::ipfs::IPFS;

#[derive(Debug)]
pub struct RevisionTag {
    hash: IpfsHash,
    revision: u128,
}

impl RevisionTag {
    pub fn new(hash: &IpfsHash, revision: u128) -> Self {
        Self {
            hash: hash.clone(),
            revision,
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
    tag: RevisionTag,
}

impl Revision {
    pub fn new(repository: &'static mut Repository, tag: RevisionTag) -> Self {
        Self {
            repository,
            tag,
        }
    }

    pub fn genesis(repository: &'static mut Repository) -> Result<Self, QFSError> {
        let catalog = Catalog::new(repository.ipfs())?;
        let hash = catalog.hash().clone();
        repository.add_catalog(catalog);
        Ok(Self {
            repository,
            tag: RevisionTag::new(&hash, 0),
        })
    }

    pub fn hash(&self) -> &IpfsHash {
        &self.tag.hash
    }

    pub fn revision(&self) -> u128 {
        self.tag.revision
    }

    pub fn lookup(&mut self, path: &str) -> Result<DirectoryEntry, QFSError> {
        let path = path::canonicalize_path(path);
        let path = path.as_str();
        let best_fit = self.retrieve_catalog_for_path(path)?;
        best_fit.find_directory_entry(path)
    }

    pub fn retrieve_catalog(&mut self, hash: &IpfsHash) -> Result<&Catalog, QFSError> {
        self.repository.retrieve_catalog(hash)
    }

    pub fn retrieve_root_catalog(&mut self) -> Result<&Catalog, QFSError> {
        let hash = self.hash().clone();
        self.retrieve_catalog(&hash)
    }

    pub fn retrieve_catalog_for_path(&mut self, path: &str) -> Result<&Catalog, QFSError> {
        let root_catalog_hash = self.hash();
        let mut hash = root_catalog_hash.clone();
        loop {
            match self.retrieve_catalog(&hash)?.find_nested_for_path(path) {
                None => return Ok(self.repository.get_opened_catalog(&hash).unwrap()),
                Some(nested_reference) => hash = nested_reference.hash().clone()
            };
        }
    }

    pub fn list_directory(&mut self, path: &str) -> Result<Vec<DirectoryEntry>, QFSError> {
        let dirent = self.lookup(path)?;
        if dirent.is_directory() {
            let catalog = self.retrieve_catalog_for_path(path)?;
            return catalog.list_directory(path);
        }
        Err(QFSError::new(format!("{} is not a directory", path).as_str()))
    }

    pub fn stream_file(&mut self, path: &str) -> Result<impl Iterator<Item=u8>, QFSError> {
        let result = self.lookup(path)?;
        self.repository.ipfs().stream(&result.hash)
    }

    pub fn fetch_file(&mut self, path: &str) -> Result<Vec<u8>, QFSError> {
        let bytes = self.stream_file(path)?.collect();
        Ok(bytes)
    }
}
