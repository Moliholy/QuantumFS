use std::error::Error;
use std::fmt;
use std::io::Write;

use rusqlite::{Connection, NO_PARAMS, OpenFlags, ToSql};
use tempfile::NamedTempFile;

use crate::errors::QFSError;
use crate::models::directoryentry;
use crate::models::directoryentry::DirectoryEntry;
use crate::operations::ipfs;
use crate::operations::ipfs::IPFS;
use crate::operations::path;
use crate::types::ipfs::IpfsHash;

lazy_static! {
    static ref LISTING_QUERY: String = format!(
    "SELECT {} \
        FROM catalog \
        WHERE parent = ? \
        ORDER BY name ASC;", directoryentry::DATABASE_FIELDS
    );

    static ref INSERT_QUERY: String = format!(
    "INSERT INTO catalog ({}) \
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)", directoryentry::DATABASE_FIELDS
    );

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
    _file: NamedTempFile,  // this is here just to avoid loosing the file
}

unsafe impl Sync for Catalog {
    // Sqlite does not allow multithreading, and hence whatever client
    // implementing QuantumFS MUST NOT write in the catalog databases concurrently.
}

impl fmt::Debug for Catalog {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Catalog<{}>", self.hash.to_string().as_str())
    }
}

impl Catalog {
    pub fn new(ipfs: &IPFS) -> Result<Self, QFSError> {
        let mut tmpfile = NamedTempFile::new().unwrap();
        let connection = Connection::open_with_flags(
            tmpfile.path(),
            OpenFlags::default(),
        ).map_err(QFSError::from)?;
        connection.execute_batch(
            format!(
                "BEGIN; \
                {} \
                {} \
                {} \
                END;",
                CREATE_CATALOG.as_str(), CREATE_INDEX.as_str(), CREATE_NESTED_CATALOGS.as_str()
            ).as_str()
        )?;
        tmpfile.flush()?;
        let file = tmpfile.as_file();
        let hash = ipfs.add(&file)?;
        Ok(Self {
            connection,
            hash,
            _file: tmpfile,
        })
    }

    pub fn load(hash: &IpfsHash, ipfs: &IPFS) -> Result<Self, QFSError> {
        let db_bytes = ipfs.fetch(hash)?;
        let mut tmpfile = NamedTempFile::new().unwrap();
        tmpfile.write_all(&db_bytes)
            .map_err(QFSError::from)?;
        let connection = Connection::open_with_flags(
            tmpfile.path(),
            OpenFlags::default(),
        ).map_err(QFSError::from)?;
        Ok(Self {
            hash: hash.clone(),
            connection,
            _file: tmpfile,
        })
    }

    pub fn hash(&self) -> &IpfsHash {
        &self.hash
    }

    pub fn list_nested(&self) -> Vec<CatalogReference> {
        let mut statement = self.connection
            .prepare(LIST_NESTED.as_str())
            .unwrap();
        let mut nested = Vec::new();
        let mut rows = statement.query(NO_PARAMS).unwrap();
        while let Ok(Some(row)) = rows.next() {
            let path: String = row.get(0).unwrap();
            let hash: String = row.get(1).unwrap();
            let catalog_reference = CatalogReference {
                path: IpfsHash::new(path.as_str()).unwrap(),
                hash: IpfsHash::new(hash.as_str()).unwrap(),
                size: row.get(3).unwrap(),
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
        let mut rows = statement.query(&[hash]).unwrap();
        if let Ok(Some(row)) = rows.next() {
            let dirent = DirectoryEntry::from_sql_row(row);
            return Ok(dirent);
        }
        Err(QFSError::new(format!("Entry at {} not found", path).as_str()))
    }

    pub fn list_directory(&self, path: &str) -> Vec<DirectoryEntry> {
        let real_path = path::canonicalize_path(path);
        let hash = ipfs::hash_bytes(real_path.as_bytes());
        let mut statement = self.connection
            .prepare(LISTING_QUERY.as_str())
            .unwrap();
        let mut rows = statement.query(&[hash]).unwrap();
        let mut dirents = Vec::new();
        while let Ok(Some(row)) = rows.next() {
            dirents.push(DirectoryEntry::from_sql_row(row));
        }
        dirents
    }

    pub fn add_directory_entry(&self, dirent: &DirectoryEntry) -> Result<(), QFSError> {
        let mut statement = self.connection
            .prepare(INSERT_QUERY.as_str())
            .unwrap();
        let result = statement.insert(&[
            &dirent.path.to_string() as &dyn ToSql,
            &dirent.parent.to_string() as &dyn ToSql,
            &dirent.hash.to_string() as &dyn ToSql,
            &dirent.flags,
            &dirent.size,
            &dirent.mode,
            &dirent.mtime,
            &dirent.name,
            &dirent.symlink,
        ]);
        match result {
            Ok(num_rows) => if num_rows == 1 { Ok(()) } else { Err(QFSError::new("Error adding a directory entry")) },
            Err(error) => Err(QFSError::new(error.description()))
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::models::catalog::Catalog;
    use crate::models::directoryentry::{DirectoryEntry, flags};
    use crate::operations::ipfs::{self, IPFS};
    use crate::types::ipfs::IpfsHash;

    #[test]
    fn test_create_catalog_should_work() {
        let ipfs = IPFS::new("127.0.0.1", 5001);
        let result = Catalog::new(&ipfs);
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_entry_should_work() {
        let ipfs = IPFS::new("127.0.0.1", 5001);
        let catalog = Catalog::new(&ipfs).unwrap();
        let path = "/file1";
        let content = "this is file1";
        let dirent = DirectoryEntry {
            path: IpfsHash::new(ipfs::hash_bytes(path.as_bytes()).as_str()).unwrap(),
            parent: IpfsHash::new(ipfs::hash_bytes("/".as_bytes()).as_str()).unwrap(),
            hash: IpfsHash::new(ipfs::hash_bytes(content.as_bytes()).as_str()).unwrap(),
            flags: flags::FILE,
            size: content.len() as i64,
            mode: 0,
            mtime: 0,
            name: "file1".to_string(),
            symlink: "".to_string(),
        };
        let result = catalog.add_directory_entry(&dirent);
        assert!(result.is_ok(), format!("{:?}", result));
        let files = catalog.list_directory("/");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], dirent);
    }
}
