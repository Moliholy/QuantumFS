use rusqlite::Row;

use crate::types::ipfs::IpfsHash;

pub static DATABASE_FIELDS: &str = "path, parent, hash, flags, size, mode, mtime, name, symlink";


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DirectoryEntry {
    pub path: IpfsHash,
    pub parent: IpfsHash,
    pub hash: IpfsHash,
    pub flags: i64,
    pub size: i64,
    pub mode: i64,
    pub mtime: i64,
    pub name: String,
    pub symlink: String,
}

impl DirectoryEntry {
    pub fn from_sql_row(row: &Row) -> Self {
        let path: String = row.get(0).unwrap();
        let parent: String = row.get(1).unwrap();
        let hash: String = row.get(2).unwrap();
        Self {
            path: IpfsHash::new(path.as_str()).unwrap(),
            parent: IpfsHash::new(parent.as_str()).unwrap(),
            hash: IpfsHash::new(hash.as_str()).unwrap(),
            flags: row.get(3).unwrap(),
            size: row.get(4).unwrap(),
            mode: row.get(5).unwrap(),
            mtime: row.get(6).unwrap(),
            name: row.get(7).unwrap(),
            symlink: row.get(8).unwrap(),
        }
    }

    pub fn is_directory(&self) -> bool {
        (self.flags & flags::DIRECTORY) > 0
    }

    pub fn is_file(&self) -> bool {
        (self.flags & flags::FILE) > 0
    }

    pub fn is_symlink(&self) -> bool {
        (self.flags & flags::LINK) > 0
    }

    pub fn is_nested_catalog_root(&self) -> bool {
        (self.flags & flags::NESTED_CATALOG_ROOT) > 0
    }
}


pub mod flags {
    pub static DIRECTORY: i64 = 1;
    pub static FILE: i64 = 4;
    pub static LINK: i64 = 8;
    pub static NESTED_CATALOG_ROOT: i64 = 32;
}
