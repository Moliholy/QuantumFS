use std::fmt;
use std::io::Write;

use sqlite::{Connection, OpenFlags, State};
use tempfile::NamedTempFile;

use crate::errors::errors::QFSError;
use crate::models::directoryentry;
use crate::models::directoryentry::DirectoryEntry;
use crate::operations::ipfs;
use crate::operations::path;
use crate::types::ipfs::IpfsHash;

lazy_static! {
    static ref LISTING_QUERY: String = format!("SELECT {} \
        FROM catalog \
        WHERE parent = ? \
        ORDER BY name ASC;", directoryentry::DATABASE_FIELDS);

    static ref FIND_PATH: String = format!("SELECT {} \
        FROM catalog \
        WHERE path = ? \
        ORDER BY name ASC \
        LIMIT 1;", directoryentry::DATABASE_FIELDS);

    static ref LIST_NESTED: String = String::from("SELECT path, hash, size FROM nested_catalogs;");
}


#[derive(Debug, Clone)]
pub struct CatalogReference {
    path: IpfsHash,
    hash: IpfsHash,
    size: i64,
}

impl CatalogReference {
    pub fn new(path: &IpfsHash, hash: &IpfsHash, size: i64) -> Self {
        Self {
            path: path.clone(),
            hash: hash.clone(),
            size
        }
    }

    pub fn path(&self) -> &IpfsHash {
        &self.path
    }

    pub fn hash(&self) -> &IpfsHash {
        &self.hash
    }

    pub fn size(&self) -> i64 {
        self.size
    }
}

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
    pub fn load(hash: &IpfsHash) -> Self {
        let db_bytes = ipfs::fetch(hash)
            .expect("Could not fetch the SQLite database");
        let mut tmpfile = NamedTempFile::new().unwrap();
        tmpfile.write_all(&db_bytes)
            .expect("Error writing the catalog in the temporary file");
        let connection = Connection::open_with_flags(
            tmpfile.path(),
            OpenFlags::new().set_read_only(),
        ).expect("Error opening the database file");
        Self {
            hash: hash.clone(),
            connection,
        }
    }

    pub fn hash(&self) -> &IpfsHash {
        &self.hash
    }

    pub fn list_nested(&self) -> Vec<CatalogReference> {
        let mut statement = self.connection
            .prepare(LIST_NESTED.as_str())
            .unwrap();
        let count = statement.count();
        let mut nested = Vec::with_capacity(count);
        while let State::Row = statement.next().unwrap() {
            let catalog_reference = CatalogReference {
                path: IpfsHash::new(statement.read::<String>(0).unwrap().as_str()).unwrap(),
                hash: IpfsHash::new(statement.read::<String>(1).unwrap().as_str()).unwrap(),
                size: statement.read::<i64>(2).unwrap(),
            };
            nested.push(catalog_reference);
        }
        nested
    }

    pub fn find_nested_for_path(&self, needle_path: &str) -> Option<CatalogReference> {
        let catalog_refs = self.list_nested();
        let mut best_match = None;
        let mut best_match_score: usize = 0;
        let real_needle_path = path::canonicalize_path(needle_path);
        for nested_catalog in catalog_refs.iter() {
            let nested_catalog_string = nested_catalog.path().to_string();
            let nested_catalog_path = nested_catalog_string.as_str();
            if real_needle_path.starts_with(nested_catalog_path) &&
                nested_catalog_path.len() > best_match_score &&
                path::is_sanitized(needle_path, nested_catalog_path)
            {
                best_match_score = nested_catalog_path.len();
                best_match = Some(nested_catalog);
            }
        }
        match best_match {
            Some(value) => Some(value.clone()),
            None => None
        }
    }

    pub fn find_directory_entry(&self, path: &str) -> Result<DirectoryEntry, QFSError> {
        let path = path::canonicalize_path(path);
        let real_path = path.as_str();
        let hash = ipfs::hash_bytes(real_path.as_bytes());
        let mut statement = self.connection
            .prepare(FIND_PATH.as_str())
            .unwrap();
        statement.bind(1, hash.as_str()).unwrap();
        statement.next().map_err(QFSError::from)?;
        Ok(DirectoryEntry::from_sql_statement(&statement))
    }

    pub fn list_directory(&self, path: &str) -> Result<Vec<DirectoryEntry>, QFSError> {
        let real_path = path::canonicalize_path(path);
        let mut dirents = Vec::new();
        let hash = ipfs::hash_bytes(real_path.as_bytes());
        let mut statement = self.connection
            .prepare(LISTING_QUERY.as_str())
            .unwrap();
        statement.bind(1, hash.as_str()).unwrap();
        while let State::Row = statement.next().map_err(QFSError::from)? {
            dirents.push(DirectoryEntry::from_sql_statement(&statement));
        }
        Ok(dirents)
    }
}
