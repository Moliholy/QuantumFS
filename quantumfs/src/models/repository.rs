use std::collections::HashMap;

use web3::types::Address;

use crate::errors::QFSError;
use crate::models::catalog::Catalog;
use crate::models::manifest::Manifest;
use crate::models::revision::{Revision, RevisionTag};
use crate::types::ipfs::IpfsHash;
use web3::Web3;
use web3::transports::Http;
use crate::operations::ipfs::IPFS;
use crate::operations::ethereum;

#[derive(Debug)]
pub struct Repository {
    manifest: Manifest,
    catalogs: HashMap<IpfsHash, Catalog>,
    web3: Web3<Http>,
    ipfs: IPFS,
}

impl Repository {
    pub fn new(client_address: Address, contract_address: Address,
               web3_url: &str, ipfs_server: &str, ipfs_port: u16) -> Self {
        Self {
            manifest: Manifest::new(client_address, contract_address, web3_url),
            catalogs: HashMap::new(),
            web3: ethereum::get_web3(web3_url),
            ipfs: IPFS::new(ipfs_server, ipfs_port)
        }
    }

    pub fn web3(&self) -> &Web3<Http> {
        &self.web3
    }

    pub fn ipfs(&self) -> &IPFS {
        &self.ipfs
    }

    fn fetch_last_revision_tag(&self) -> Result<RevisionTag, QFSError> {
        self.manifest.fetch_last_revision()
    }

    fn fetch_revision_tag(&self, revision_number: u128) -> Result<RevisionTag, QFSError> {
        self.manifest.fetch_revision(revision_number)
    }

    pub fn load_revision(&'static mut self, revision_number: u128) -> Result<Revision, QFSError> {
        let tag = self.fetch_revision_tag(revision_number)?;
        Ok(Revision::new(self, tag))
    }

    pub fn load_current_revision(&'static mut self) -> Result<Revision, QFSError> {
        let tag = self.fetch_last_revision_tag()?;
        Ok(Revision::new(self, tag))
    }

    pub fn create_revision(&'static mut self) -> Result<Revision, QFSError> {
        let (hash, revision) = {
            let current_revision_tag = self.fetch_last_revision_tag()?;
            (current_revision_tag.hash().clone(), current_revision_tag.revision())
        };
        match revision {
            0 => Revision::genesis(self),
            _ => {
                let tag = RevisionTag::new(&hash, revision + 1);
                Ok(Revision::new(self, tag))
            }
        }
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
            catalog
        );
    }
}
