use crate::types::ipfs::IpfsHash;
use crate::operations::ipfs;
use tempfile::NamedTempFile;
use std::io::Write;
use sqlite::{Connection, OpenFlags};
use std::fmt;

pub struct Catalog {
    hash: IpfsHash,
    connection: Connection,
}

impl fmt::Debug for Catalog {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.hash.to_string().as_str())
    }
}

impl Catalog {
    pub fn load(hash: IpfsHash) -> Self {
        let db_bytes = ipfs::fetch(&hash)
            .expect("Could not fetch the SQLite database");
        let mut tmpfile = NamedTempFile::new().unwrap();
        tmpfile.write_all(&db_bytes)
            .expect("Error writing the catalog in the temporary file");
        let connection = Connection::open_with_flags(
            tmpfile.path(),
            OpenFlags::new().set_read_only()
        ).expect("Error opening the database file");
        Self {
            hash,
            connection,
        }
    }
}
