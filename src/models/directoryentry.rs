use crate::types::ipfs::IpfsHash;

#[derive(Debug)]
pub struct DirectoryEntry {
    pub parent: IpfsHash,
    pub path: IpfsHash,
    pub content: IpfsHash,
    pub flags: i32,
    pub size: i32,
    pub mode: i32,
    pub mtime: i32,
    pub name: String,
    pub symlink: String,
}
