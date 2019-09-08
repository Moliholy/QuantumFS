use web3::transports::Http;
use web3::types::Address;
use web3::Web3;

use crate::errors::QFSError;
use crate::models::manifest::Manifest;
use crate::models::revision::{Revision, RevisionTag};
use crate::operations::ethereum;
use crate::operations::ipfs::IPFS;

#[derive(Debug)]
pub struct Repository {
    manifest: Manifest,
    web3: Web3<Http>,
    ipfs_data: (String, u16),
}

impl Repository {
    pub fn new(client_address: Address, contract_address: Address,
               web3_url: &str, ipfs_server: &str, ipfs_port: u16) -> Self {
        Self {
            manifest: Manifest::new(client_address, contract_address, web3_url),
            web3: ethereum::get_web3(web3_url),
            ipfs_data: (ipfs_server.to_string(), ipfs_port),
        }
    }

    fn fetch_last_revision_tag(&self) -> Result<RevisionTag, QFSError> {
        self.manifest.fetch_last_revision()
    }

    fn fetch_revision_tag(&self, revision_number: u128) -> Result<RevisionTag, QFSError> {
        self.manifest.fetch_revision(revision_number)
    }

    fn get_ipfs(&self) -> IPFS {
        let (server, port) = &self.ipfs_data;
        IPFS::new(server.as_str(), *port)
    }

    pub fn load_revision(&mut self, revision_number: u128) -> Result<Revision, QFSError> {
        let tag = self.fetch_revision_tag(revision_number)?;
        Ok(Revision::new(self.get_ipfs(), tag))
    }

    pub fn load_current_revision(&mut self) -> Result<Revision, QFSError> {
        let tag = self.fetch_last_revision_tag()?;
        Ok(Revision::new(self.get_ipfs(), tag))
    }

    pub fn create_revision(&'static mut self) -> Result<Revision, QFSError> {
        let (hash, revision) = {
            let current_revision_tag = self.fetch_last_revision_tag()?;
            (current_revision_tag.hash().clone(), current_revision_tag.revision())
        };
        match revision {
            0 => Revision::genesis(self.get_ipfs()),
            _ => {
                let tag = RevisionTag::new(&hash, revision + 1);
                Ok(Revision::new(self.get_ipfs(), tag))
            }
        }
    }
}
