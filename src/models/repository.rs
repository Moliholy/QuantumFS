use std::collections::HashMap;

use web3::contract::Error;
use web3::types::Address;

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
    pub fn new(address: Address) -> Self {
        Self {
            manifest: Manifest::new(address),
            catalogs: HashMap::new(),
        }
    }

    pub fn load_revision(&'static mut self, revision_number: u128) -> Result<Revision, Error> {
        let tag = self.manifest.fetch_revision(revision_number)?;
        Ok(Revision::new(self, tag))
    }

    pub fn load_current_revision(&'static mut self) -> Result<Revision, Error> {
        let tag = self.manifest.fetch_last_revision()?;
        Ok(Revision::new(self, tag))
    }

    pub fn retrieve_and_open_catalog(&mut self, hash: &IpfsHash) -> &Catalog {
        let catalog = Catalog::load(hash);
        self.catalogs.insert(
            hash.clone(),
            catalog
        );
        &self.catalogs[&hash]
    }

    pub fn retrieve_catalog(&mut self, hash: &IpfsHash) -> &Catalog {
        if self.catalogs.contains_key(&hash) {
            return &self.catalogs[&hash];
        }
        self.retrieve_and_open_catalog(hash)
    }

    pub fn get_opened_catalog(&self, hash: &IpfsHash) -> Option<&Catalog> {
        self.catalogs.get(hash)
    }
}
