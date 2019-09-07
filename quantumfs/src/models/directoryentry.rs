use sqlite::Statement;

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
    pub fn from_sql_statement(statement: &Statement) -> Self {
        DirectoryEntry {
            path: IpfsHash::new(statement.read::<String>(0).unwrap().as_str()).unwrap(),
            parent: IpfsHash::new(statement.read::<String>(1).unwrap().as_str()).unwrap(),
            hash: IpfsHash::new(statement.read::<String>(2).unwrap().as_str()).unwrap(),
            flags: statement.read::<i64>(3).unwrap(),
            size: statement.read::<i64>(4).unwrap(),
            mode: statement.read::<i64>(5).unwrap(),
            mtime: statement.read::<i64>(6).unwrap(),
            name: statement.read::<String>(7).unwrap(),
            symlink: statement.read::<String>(8).unwrap(),
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
