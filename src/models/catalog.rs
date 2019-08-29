use crate::types::ipfs::IpfsHash;
use crate::operations::ipfs;
use tempfile::NamedTempFile;
use std::io::Write;

#[derive(Debug)]
pub struct Catalog {
    hash: IpfsHash
}

impl Catalog {
    pub fn load(hash: IpfsHash) {
        let db_bytes = ipfs::fetch(hash)
            .expect("Could not fetch the SQLite database");
        let mut tmpfile = NamedTempFile::new().unwrap();
        tmpfile.write_all(&db_bytes)
            .expect("Error writing the catalog in the temporary file");
    }
}
