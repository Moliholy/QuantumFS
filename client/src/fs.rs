use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;
use std::sync::RwLock;

use fuse_mt::{FileAttr, FilesystemMT, FileType, RequestInfo, ResultData, ResultEmpty, ResultOpen, ResultReaddir, ResultXattr};
use libc;
use time::Timespec;

use quantumfs::errors::QFSError;
use quantumfs::models::repository::Repository;
use quantumfs::models::revision::Revision;

const TTL: Timespec = Timespec { sec: 1, nsec: 0 };


pub struct QuantumFS {
    repository: RwLock<Repository>,
    opened_files: RwLock<HashMap<String, File>>,
    revision: RwLock<Revision>,
}

impl FilesystemMT for QuantumFS {
    fn init(&self, _req: RequestInfo) -> ResultEmpty {
        Ok(())
    }

    fn destroy(&self, _req: RequestInfo) {
        self.opened_files.write().unwrap().clear();
    }

    fn getattr(&self, _req: RequestInfo, _path: &Path, _fh: Option<u64>) -> Result<(Timespec, FileAttr), i32> {
        let mut revision = self.revision.write().unwrap();
        let path = _path.to_str().unwrap();
        match revision.lookup(path) {
            Err(_) => Err(1),
            Ok(dirent) => {
                let mut kind = FileType::RegularFile;
                if dirent.is_directory() {
                    kind = FileType::Directory;
                } else if dirent.is_symlink() {
                    kind = FileType::Symlink
                }
                Ok((TTL, FileAttr {
                    size: dirent.size as u64,
                    blocks: (1 + dirent.size / 512) as u64,
                    atime: Timespec { sec: dirent.mtime, nsec: 0 },
                    mtime: Timespec { sec: dirent.mtime, nsec: 0 },
                    ctime: Timespec { sec: dirent.mtime, nsec: 0 },
                    crtime: Timespec { sec: dirent.mtime, nsec: 0 },
                    kind,
                    perm: 0,
                    nlink: 1,
                    uid: 1,
                    gid: 1,
                    rdev: 1,
                    flags: dirent.flags as u32
                }))
            }
        }
    }

    fn readlink(&self, _req: RequestInfo, path: &Path) -> ResultData {
        match self.revision.write().unwrap().lookup(path.to_str().unwrap()) {
            Err(error) => Err(1),
            Ok(dirent) => {
                if dirent.is_symlink() {
                    Ok(Vec::from(dirent.symlink.as_bytes()))
                } else {
                    Err(libc::ENOLINK)
                }
            }
        }
    }

    fn open(&self, _req: RequestInfo, path: &Path, _flags: u32) -> ResultOpen {
        let path = path.to_str().unwrap();
        match self.revision.write().unwrap().get_file(path) {
            Err(error) => Err(libc::ENOENT),
            Ok(file) => {
                self.opened_files.write().unwrap().insert(path.into(), file);
                Ok((0, libc::O_RDONLY as u32))
            }
        }
    }

    fn read(&self, _req: RequestInfo, _path: &Path, _fh: u64, _offset: u64, _size: u32, result: impl FnOnce(Result<&[u8], i32>)) {
        unimplemented!()
    }

    fn release(&self, _req: RequestInfo, _path: &Path, _fh: u64, _flags: u32, _lock_owner: u64, _flush: bool) -> ResultEmpty {
        unimplemented!()
    }

    fn readdir(&self, _req: RequestInfo, _path: &Path, _fh: u64) -> ResultReaddir {
        unimplemented!()
    }

    fn getxattr(&self, _req: RequestInfo, _path: &Path, _name: &OsStr, _size: u32) -> ResultXattr {
        unimplemented!()
    }
}

impl QuantumFS {
    pub fn new(mut repository: Repository) -> Result<Self, QFSError> {
        let revision = repository.load_current_revision()?;
        Ok(Self {
            repository: RwLock::new(repository),
            opened_files: RwLock::new(HashMap::new()),
            revision: RwLock::new(revision),
        })
    }
}
