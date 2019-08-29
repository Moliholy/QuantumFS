use crate::models::manifest::Manifest;
use std::collections::HashMap;
use crate::operations::ipfs::IpfsHash;
use crate::models::catalog::Catalog;

#[readonly::make]
#[derive(Debug)]
pub struct Repository {
    address: String,
    manifest: Manifest,
    catalogs: HashMap<IpfsHash, Catalog>,
}

impl Repository {

}
