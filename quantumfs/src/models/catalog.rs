use std::{fmt, fs};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OpenFlags};
use tempfile::NamedTempFile;

use crate::errors::QFSError;
use crate::models::directoryentry::{DirectoryEntry, flags};
use crate::operations::{database, ipfs};
use crate::operations::ipfs::IPFS;
use crate::operations::path;
use crate::types::ipfs::IpfsHash;

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
    file: File,  // this is here just to avoid loosing the file
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
    pub fn new(cache_path: &Path) -> Result<Self, QFSError> {
        let mut tmpfile = NamedTempFile::new().unwrap();
        let connection = Connection::open_with_flags(
            tmpfile.path(),
            OpenFlags::default(),
        ).map_err(QFSError::from)?;
        database::create_catalog(&connection)?;
        let root_folder = DirectoryEntry {
            path: IpfsHash::new(ipfs::hash_bytes("/".as_bytes()).as_str()).unwrap(),
            parent: IpfsHash::new(ipfs::hash_bytes("/".as_bytes()).as_str()).unwrap(),
            hash: IpfsHash::new(ipfs::hash_bytes(&[]).as_str()).unwrap(),
            flags: flags::DIRECTORY | flags::NESTED_CATALOG_ROOT,
            size: 0,
            mode: 0,
            mtime: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
            name: "".to_string(),
            symlink: "".to_string(),
        };
        database::add_directory_entry(&connection, &root_folder)?;
        tmpfile.flush()?;
        let mut data = Vec::new();
        tmpfile.as_file().read_to_end(&mut data)?;
        let hash = ipfs::hash_bytes(&data);
        let catalog_file_path = cache_path.join(hash.as_str());
        fs::rename(tmpfile.path(), catalog_file_path.as_path())?;
        let catalog = Self {
            connection,
            hash: IpfsHash::new(hash.as_str())?,
            file: File::open(catalog_file_path.as_path())?,
        };
        Ok(catalog)
    }

    pub fn file(&self) -> &File {
        return &self.file
    }

    pub fn load(path: &Path) -> Result<Self, QFSError> {
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let connection = Connection::open_with_flags(
            path,
            OpenFlags::default(),
        ).map_err(QFSError::from)?;
        Ok(Self {
            hash: IpfsHash::new(file_name)?,
            connection,
            file: File::open(path)?,
        })
    }

    pub fn hash(&self) -> &IpfsHash {
        &self.hash
    }

    pub fn list_nested(&self) -> Result<Vec<CatalogReference>, QFSError> {
        database::list_nested(&self.connection)
    }

    pub fn find_nested_for_path(&self, needle_path: &str) -> Result<Option<CatalogReference>, QFSError> {
        let catalog_refs = self.list_nested()?;
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
            Some(value) => Ok(Some(value.clone())),
            None => Ok(None)
        }
    }

    pub fn find_directory_entry(&self, path: &str) -> Result<DirectoryEntry, QFSError> {
        let path = path::canonicalize_path(path);
        let real_path = path.as_str();
        let hash = ipfs::hash_bytes(real_path.as_bytes());
        database::find_directory_entry(&self.connection, hash)
    }

    pub fn list_directory(&self, path: &str) -> Result<Vec<DirectoryEntry>, QFSError> {
        let real_path = path::canonicalize_path(path);
        let hash = ipfs::hash_bytes(real_path.as_bytes());
        database::list_directory(&self.connection, hash)
    }

    pub fn add_directory_entry(&self, dirent: &DirectoryEntry) -> Result<(), QFSError> {
        database::add_directory_entry(&self.connection, &dirent)
    }
}


#[cfg(test)]
mod tests {
    use crate::models::catalog::Catalog;
    use crate::models::directoryentry::{DirectoryEntry, flags};
    use crate::operations::ipfs::{self, IPFS};
    use crate::types::ipfs::IpfsHash;
    use std::path::Path;

    #[test]
    fn test_create_catalog_should_work() {
        let ipfs = IPFS::new("127.0.0.1", 5001);
        let cache_path = Path::new("/tmp");
        let result = Catalog::new(cache_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_entry_should_work() {
        let ipfs = IPFS::new("127.0.0.1", 5001);
        let cache_path = Path::new("/tmp");
        let catalog = Catalog::new(cache_path).unwrap();
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
        let files = catalog.list_directory("/").unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], dirent);
    }
}
