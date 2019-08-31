use std::collections::HashMap;

use web3::types::Address;

use crate::errors::QFSError;
use crate::models::catalog::Catalog;
use crate::models::manifest::Manifest;
use crate::models::revision::Revision;
use crate::types::ipfs::IpfsHash;

#[derive(Debug)]
pub struct Repository {
    manifest: Manifest,
    catalogs: HashMap<IpfsHash, Catalog>,
}

impl Repository {
    pub fn new(client_address: Address, contract_address: Address) -> Self {
        Self {
            manifest: Manifest::new(client_address, contract_address),
            catalogs: HashMap::new(),
        }
    }

    pub fn load_revision(&'static mut self, revision_number: u128) -> Result<Revision, QFSError> {
        let tag = self.manifest.fetch_revision(revision_number)?;
        Ok(Revision::new(self, tag))
    }

    pub fn load_current_revision(&'static mut self) -> Result<Revision, QFSError> {
        let tag = self.manifest.fetch_last_revision()?;
        Ok(Revision::new(self, tag))
    }

    pub fn retrieve_and_open_catalog(&mut self, hash: &IpfsHash) -> Result<&Catalog, QFSError> {
        let catalog = Catalog::load(hash)?;
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
            catalog
        );
    }
}
