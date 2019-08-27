pub mod manifest {
    use crate::operations::ipfs::ipfs::validate_ipfs_hash;

    #[readonly::make]
    #[derive(Debug)]
    pub struct Manifest {
        hash: String,
        timestamp: u64,
    }

    impl Manifest {
        pub fn new(hash: &str, timestamp: u64) -> Option<Manifest> {
            if validate_ipfs_hash(hash) {
                let manifest = Self { hash: String::from(hash), timestamp };
                return Some(manifest);
            }
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::models::manifest::manifest::Manifest;
    use std::time::SystemTime;

    fn instantiate(hash: &str) -> Option<Manifest> {
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        Manifest::new(hash, now)
    }

    #[test]
    fn manifest_instantiation_with_correct_hash_should_work() {
        let hash = "QmaozNR7DZHQK1ZcU9p7QdrshMvXqWK6gpu5rmrkPdT3L4";
        let manifest = instantiate(hash);
        assert!(manifest.is_some());
    }

    #[test]
    fn manifest_instantiation_with_incorrect_hash_should_fail() {
        let hash = "invalidhash";
        let manifest = instantiate(hash);
        assert!(manifest.is_none());
    }
}
