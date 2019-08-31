use std::fmt;
use std::io::Write;

use sqlite::{Connection, OpenFlags, State};
use tempfile::NamedTempFile;

use crate::errors::QFSError;
use crate::models::directoryentry;
use crate::models::directoryentry::DirectoryEntry;
use crate::operations::ipfs;
use crate::operations::path;
use crate::types::ipfs::IpfsHash;

lazy_static! {
    static ref LISTING_QUERY: String = format!(
    "SELECT {} \
        FROM catalog \
        WHERE parent = ? \
        ORDER BY name ASC;", directoryentry::DATABASE_FIELDS)
        ;

    static ref FIND_PATH: String = format!(
    "SELECT {} \
        FROM catalog \
        WHERE path = ? \
        ORDER BY name ASC \
        LIMIT 1;", directoryentry::DATABASE_FIELDS
    );

    static ref LIST_NESTED: String = String::from(
    "SELECT path, hash, size \
        FROM nested_catalogs;"
    );
    static ref CREATE_CATALOG: String = String::from(
    "CREATE TABLE catalog
        (path TEXT, parent TEXT,\
        hardlinks INTEGER, hash BLOB, size INTEGER, mode INTEGER, mtime INTEGER,\
        flags INTEGER, name TEXT, symlink TEXT, uid INTEGER, gid INTEGER, \
        xattr BLOB, \
        CONSTRAINT pk_catalog PRIMARY KEY (path));"
    );

    static ref CREATE_INDEX: String = String::from(
    "CREATE INDEX idx_catalog_parent \
        ON catalog (parent);"
    );

    static ref CREATE_NESTED_CATALOGS: String = String::from(
    "CREATE TABLE nested_catalogs (path TEXT, hash TEXT, size INTEGER, \
        CONSTRAINT pk_nested_catalogs PRIMARY KEY (path));"
    );
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
            size,
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
        write!(f, "Catalog<{}>", self.hash.to_string().as_str())
    }
}

impl Catalog {
    pub fn new() -> Result<Self, QFSError> {
        let mut tmpfile = NamedTempFile::new().unwrap();
        let connection = Connection::open(tmpfile.path())
            .map_err(QFSError::from)?;
        connection.execute(CREATE_CATALOG.as_str()).unwrap();
        connection.execute(CREATE_INDEX.as_str()).unwrap();
        connection.execute(CREATE_NESTED_CATALOGS.as_str()).unwrap();
        tmpfile.flush()?;
        let file = tmpfile.as_file();
        let hash = ipfs::add(&file)?;
        Ok(Self {
            connection,
            hash,
        })
    }

    pub fn load(hash: &IpfsHash) -> Result<Self, QFSError> {
        let db_bytes = ipfs::fetch(hash)?;
        let mut tmpfile = NamedTempFile::new().unwrap();
        tmpfile.write_all(&db_bytes)
            .map_err(QFSError::from)?;
        let connection = Connection::open_with_flags(
            tmpfile.path(),
            OpenFlags::new().set_read_only(),
        ).map_err(QFSError::from)?;
        Ok(Self {
            hash: hash.clone(),
            connection,
        })
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


#[cfg(test)]
mod tests {
    use crate::models::catalog::Catalog;

    #[test]
    fn test_create_catalog_should_work() {
        let result = Catalog::new();
        assert!(result.is_ok());
    }
}
