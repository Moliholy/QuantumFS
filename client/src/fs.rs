use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;

use fuse_mt::{FileAttr, FilesystemMT, RequestInfo, ResultData, ResultEmpty, ResultOpen, ResultReaddir, ResultStatfs, ResultXattr};
use time::Timespec;

use quantumfs::errors::QFSError;
use quantumfs::models::repository::Repository;
use quantumfs::models::revision::Revision;
use std::sync::RwLock;

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
        unimplemented!()
    }

    fn utimens(&self, _req: RequestInfo, _path: &Path, _fh: Option<u64>, _atime: Option<Timespec>, _mtime: Option<Timespec>) -> ResultEmpty {
        unimplemented!()
    }

    fn readlink(&self, _req: RequestInfo, _path: &Path) -> ResultData {
        unimplemented!()
    }

    fn open(&self, _req: RequestInfo, _path: &Path, _flags: u32) -> ResultOpen {
        unimplemented!()
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

    fn statfs(&self, _req: RequestInfo, _path: &Path) -> ResultStatfs {
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
