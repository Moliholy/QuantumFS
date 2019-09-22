use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use filepath::FilePath;

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
    cache_dir: PathBuf,
}

impl Revision {
    pub fn new(ipfs: IPFS, tag: RevisionTag, cache_dir: &Path) -> Self {
        Self {
            ipfs,
            tag,
            catalogs: HashMap::new(),
            cache_dir: cache_dir.to_owned(),
        }
    }

    pub fn genesis(ipfs: IPFS, cache_dir: &Path) -> Result<Self, QFSError> {
        let catalog = Catalog::new(cache_dir)?;
        let hash = ipfs.add(catalog.file())?;
        let mut instance = Self {
            ipfs,
            tag: RevisionTag::new(&hash, 0),
            catalogs: HashMap::new(),
            cache_dir: PathBuf::from(cache_dir),
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
                Ok(None) => return Ok(self.get_opened_catalog(&hash).unwrap()),
                Ok(Some(nested_reference)) => hash = nested_reference.hash().clone(),
                Err(error) => return Err(error)
            };
        }
    }

    pub fn list_directory(&mut self, path: &str) -> Result<Vec<DirectoryEntry>, QFSError> {
        let dirent = self.lookup(path)?;
        if dirent.is_directory() {
            let catalog = self.retrieve_catalog_for_path(path)?;
            let entries = catalog.list_directory(path)?;
            return Ok(entries);
        }
        Err(QFSError::new(format!("{} is not a directory", path).as_str()))
    }

    pub fn get_file(&mut self, path: &str) -> Result<File, QFSError> {
        let result = self.lookup(path)?;
        if !result.is_file() {
            return Err(QFSError::new(format!("{} is not a file", path).as_str()));
        }
        let hash = &result.hash;
        self.get_object(hash)
    }

    fn fetch_object(&self, hash: &IpfsHash) -> Result<File, QFSError> {
        let cache_path = self.cache_path_for_hash(hash);
        let mut file = File::create(cache_path.as_path())
            .expect(format!("Failure creating a new file in {}", cache_path.to_str().unwrap()).as_str());
        let bytes = self.ipfs.fetch(hash)?;
        file.write_all(bytes.as_ref());
        Ok(file)
    }

    fn cache_path_for_hash(&self, hash: &IpfsHash) -> PathBuf {
        self.cache_dir.join(hash.as_ref())
    }

    fn get_object_from_cache(&self, hash: &IpfsHash) -> Option<File> {
        let path = self.cache_path_for_hash(hash);
        match File::open(path) {
            Err(_) => None,
            Ok(file) => Some(file)
        }
    }

    pub fn get_object(&mut self, hash: &IpfsHash) -> Result<File, QFSError> {
        match self.get_object_from_cache(hash) {
            None => self.fetch_object(hash),
            Some(file) => Ok(file)
        }
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
        let catalog_file = self.get_object(hash)?;
        let catalog_file_path = catalog_file.path()?;
        let catalog = Catalog::load(catalog_file_path.as_path())?;
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
