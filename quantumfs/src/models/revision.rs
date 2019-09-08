use std::collections::HashMap;

use crate::errors::QFSError;
use crate::models::catalog::Catalog;
use crate::models::directoryentry::DirectoryEntry;
use crate::operations::{ipfs, path};
use crate::operations::ipfs::IPFS;
use crate::types::ipfs::IpfsHash;

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
    ipfs: IPFS,
    tag: RevisionTag,
    catalogs: HashMap<IpfsHash, Catalog>,
}

impl Revision {
    pub fn new(ipfs: IPFS, tag: RevisionTag) -> Self {
        Self {
            ipfs,
            tag,
            catalogs: HashMap::new(),
        }
    }

    pub fn genesis(ipfs: IPFS) -> Result<Self, QFSError> {
        let catalog = Catalog::new(&ipfs)?;
        let hash = catalog.hash().clone();
        let mut instance = Self {
            ipfs,
            tag: RevisionTag::new(&hash, 0),
            catalogs: HashMap::new(),
        };
        instance.add_catalog(catalog);
        Ok(instance)
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

    pub fn retrieve_root_catalog(&mut self) -> Result<&Catalog, QFSError> {
        let hash = self.hash().clone();
        self.retrieve_catalog(&hash)
    }

    pub fn retrieve_catalog_for_path(&mut self, path: &str) -> Result<&Catalog, QFSError> {
        let root_catalog_hash = self.hash();
        let mut hash = root_catalog_hash.clone();
        loop {
            match self.retrieve_catalog(&hash)?.find_nested_for_path(path) {
                None => return Ok(self.get_opened_catalog(&hash).unwrap()),
                Some(nested_reference) => hash = nested_reference.hash().clone()
            };
        }
    }

    pub fn list_directory(&mut self, path: &str) -> Result<Vec<DirectoryEntry>, QFSError> {
        let dirent = self.lookup(path)?;
        if dirent.is_directory() {
            let catalog = self.retrieve_catalog_for_path(path)?;
            return Ok(catalog.list_directory(path));
        }
        Err(QFSError::new(format!("{} is not a directory", path).as_str()))
    }

    pub fn stream_file(&mut self, path: &str) -> Result<impl Iterator<Item=u8>, QFSError> {
        let result = self.lookup(path)?;
        self.ipfs.stream(&result.hash)
    }

    pub fn fetch_file(&mut self, path: &str) -> Result<Vec<u8>, QFSError> {
        let bytes = self.stream_file(path)?.collect();
        Ok(bytes)
    }

    pub fn add_directory_entry(&mut self, dirent: DirectoryEntry, path: &str) -> Result<(), QFSError> {
        if IpfsHash::new(ipfs::hash_bytes(path.as_bytes()).as_str())? != dirent.parent {
            return Err(QFSError::new("Invalid path"));
        }
        if !path.ends_with(format!("/{}", dirent.name).as_str()) {
            return Err(QFSError::new("File name does not match the one in the path"))
        }
        self.retrieve_catalog_for_path(path)?.add_directory_entry(&dirent)
    }

    pub fn retrieve_and_open_catalog(&mut self, hash: &IpfsHash) -> Result<&Catalog, QFSError> {
        let catalog = Catalog::load(hash, &self.ipfs)?;
        self.add_catalog(catalog);
        Ok(&self.catalogs[&hash])
    }

    pub fn retrieve_catalog(&mut self, hash: &IpfsHash) -> Result<&Catalog, QFSError> {
        if self.catalogs.contains_key(&hash) {
            return Ok(&self.catalogs[&hash]);
        }
        self.retrieve_and_open_catalog(hash)
    }

    pub fn get_opened_catalog(&self, hash: &IpfsHash) -> Option<&Catalog> {
        self.catalogs.get(hash)
    }

    pub fn add_catalog(&mut self, catalog: Catalog) {
        self.catalogs.insert(
            catalog.hash().clone(),
            catalog,
        );
    }
}
