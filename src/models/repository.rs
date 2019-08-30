use crate::models::manifest::Manifest;
use crate::types::ipfs::IpfsHash;
use crate::models::catalog::Catalog;
use crate::operations::ethereum;
use web3::types::Address;
use std::collections::HashMap;


#[derive(Debug)]
pub struct Repository {
    address: Address,
    manifest: Manifest,
    catalogs: HashMap<IpfsHash, Catalog>,
}

impl Repository {
    pub fn load(address: Address) -> Self {
        let (hash, revision) = ethereum::fetch_last_revision(address)
            .expect("Failure loading the repository");
        let manifest = Manifest::new(hash, revision);
        let catalogs: HashMap<IpfsHash, Catalog> = HashMap::new();
        Self {
            address,
            manifest,
            catalogs,
        }
    }
}
