use crate::types::ipfs::IpfsHash;

pub static DATABASE_FIELDS: &str = "path, parent, hash, flags, size, mode, mtime, name, symlink";

#[derive(Debug)]
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
