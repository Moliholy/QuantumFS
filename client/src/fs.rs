use std::collections::HashMap;
use std::fs::File;

use fuse_mt::FilesystemMT;

use quantumfs::errors::QFSError;
use quantumfs::models::repository::Repository;
use quantumfs::models::revision::Revision;

pub struct QuantumFS {
    repository: Repository,
    opened_files: HashMap<String, File>,
    revision: Revision,
}

impl FilesystemMT for QuantumFS {

}

impl QuantumFS {
    pub fn new(mut repository: Repository) -> Result<Self, QFSError> {
        let revision = repository.load_current_revision()?;
        Ok(Self {
            repository,
            opened_files: HashMap::new(),
            revision,
        })
    }
}
