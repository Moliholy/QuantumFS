use web3::contract::Contract;
use web3::transports::Http;
use web3::types::Address;
use web3::Web3;

use crate::errors::QFSError;
use crate::models::revision::RevisionTag;
use crate::operations::ethereum;

#[derive(Debug)]
pub struct Manifest {
    address: Address,
    contract: Contract<Http>,
    web3: Web3<Http>,
}

impl Manifest {
    pub fn new(address: Address, contract_address: Address) -> Self {
        let web3 = ethereum::get_web3(ethereum::WEB3_DEFAULT_URL);
        let contract = ethereum::get_contract(&web3, contract_address);
        Self {
            address,
            contract,
            web3,
        }
    }

    pub fn fetch_revision(&self, revision: u128) -> Result<RevisionTag, QFSError> {
        let (hash, revision) = ethereum::fetch_revision(&self.contract, self.address, revision)?;
        Ok(RevisionTag::new(&hash, revision))
    }

    pub fn fetch_last_revision(&self) -> Result<RevisionTag, QFSError> {
        let (hash, revision) = ethereum::fetch_last_revision(&self.contract, self.address)?;
        Ok(RevisionTag::new(&hash, revision))
    }
}


#[cfg(test)]
mod tests {
    use crate::models::manifest::Manifest;
    use crate::operations::ethereum;
    use crate::operations::ethereum::tests::{TEST_CONTRACT, TEST_WEB3};

    fn create_manifest() -> Manifest {
        Manifest::new(ethereum::coinbase(&TEST_WEB3), TEST_CONTRACT.address())
    }

    #[test]
    fn manifest_instantiation_should_work() {
        let manifest = create_manifest();
        let tag = manifest.fetch_last_revision().unwrap();
        assert_eq!(tag.hash().to_string().as_str(), "0000000000000000000000000000000000000000000000");
        assert_eq!(tag.revision(), 0);
    }

    #[test]
    fn manifest_instantiation_with_invalid_revision_should_fail() {
        let manifest = create_manifest();
        let tag = manifest.fetch_revision(100000);
        assert!(tag.is_err());
    }
}
