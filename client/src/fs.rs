use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::File;
use std::io::{Read, Seek};
use std::io::SeekFrom::Start;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use fuse_mt::{DirectoryEntry, FileAttr, FilesystemMT, FileType, RequestInfo, ResultData, ResultEmpty, ResultOpen, ResultReaddir};
use libc;
use time::Timespec;

use quantumfs::errors::QFSError;
use quantumfs::models::directoryentry::DirectoryEntry as QFSDirent;
use quantumfs::models::repository::Repository;
use quantumfs::models::revision::Revision;

const TTL: Timespec = Timespec { sec: 1, nsec: 0 };


pub struct QuantumFS {
    opened_files: RwLock<HashMap<PathBuf, File>>,
    revision: RwLock<Revision>,
}

fn get_file_type(dirent: &QFSDirent) -> FileType {
    let mut kind = FileType::RegularFile;
    if dirent.is_directory() {
        kind = FileType::Directory;
    } else if dirent.is_symlink() {
        kind = FileType::Symlink
    }
    kind
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
                let kind = get_file_type(&dirent);
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
                    flags: dirent.flags as u32,
                }))
            }
        }
    }

    fn readlink(&self, _req: RequestInfo, path: &Path) -> ResultData {
        match self.revision.write().unwrap().lookup(path.to_str().unwrap()) {
            Err(_) => Err(libc::ENOENT),
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
        match self.revision.write().unwrap().get_file(path.to_str().unwrap()) {
            Err(_) => Err(libc::ENOENT),
            Ok(file) => {
                self.opened_files.write().unwrap().insert(PathBuf::from(path), file);
                Ok((0, libc::O_RDONLY as u32))
            }
        }
    }

    fn read(&self, _req: RequestInfo, path: &Path, _fh: u64, offset: u64, size: u32, result: impl FnOnce(Result<&[u8], i32>)) {
        if let Some(mut file) = self.opened_files.read().unwrap().get(path) {
            let size = size as usize;
            let mut buffer = Vec::with_capacity(size);
            file.seek(Start(offset)).unwrap();
            file.read_exact(&mut buffer[0..size]).unwrap();
            result(Ok(&buffer));
        } else {
            result(Err(libc::ENOENT));
        }
    }

    fn release(&self, _req: RequestInfo, path: &Path, _fh: u64, _flags: u32, _lock_owner: u64, _flush: bool) -> ResultEmpty {
        self.opened_files.write().unwrap().remove(path);
        Ok(())
    }

    fn readdir(&self, _req: RequestInfo, path: &Path, _fh: u64) -> ResultReaddir {
        match self.revision.write().unwrap().list_directory(path.to_str().unwrap()) {
            Err(_) => Err(libc::ENOENT),
            Ok(dirents) => {
                Ok(dirents.iter().map(|dirent| {
                    DirectoryEntry {
                        name: OsString::from(dirent.name.as_str()),
                        kind: get_file_type(&dirent),
                    }
                }).collect())
            }
        }
    }
}

impl QuantumFS {
    pub fn new(mut repository: Repository) -> Result<Self, QFSError> {
        let revision = match repository.load_current_revision()? {
            Some(current_revision) => current_revision,
            None => repository.create_revision()?,
        };
        Ok(Self {
            opened_files: RwLock::new(HashMap::new()),
            revision: RwLock::new(revision),
        })
    }
}
