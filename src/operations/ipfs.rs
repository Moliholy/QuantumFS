pub mod ipfs {
    extern crate ipfsapi;
    extern crate regex;

    use regex::Regex;

    static IPFS_HASH_PATTERN: &str = "^Q[a-zA-z0-9]{45}$";


    pub fn validate_ipfs_hash(hash: &str) -> bool {
        Regex::new(IPFS_HASH_PATTERN).unwrap().is_match(hash)
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::ipfs::ipfs::validate_ipfs_hash;

    #[test]
    fn validate_ipfs_hash_with_valid_hash_should_work() {
        let hash = "QmaozNR7DZHQK1ZcU9p7QdrshMvXqWK6gpu5rmrkPdT3L4";
        let result = validate_ipfs_hash(hash);
        assert!(result);
    }
}
