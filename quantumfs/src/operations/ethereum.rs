use std::convert::TryFrom;

use serde_json;
use serde_json::Value;
use web3::contract::{Contract, Options};
use web3::futures::Future;
use web3::transports::Http;
use web3::types::{Address, U256};
use web3::Web3;

use crate::errors::QFSError;
use crate::types::ipfs::IpfsHash;

static CONTRACT_JSON_BYTES: &[u8] = include_bytes!("../../../ethereum/build/contracts/QuantumFS.json");


pub fn get_web3(url: &str) -> Web3<Http> {
    let (event_loop, transport) = web3::transports::Http::new(url)
        .expect(format!("Web3 could not be loaded from {}", url).as_ref());
    event_loop.into_remote();
    Web3::new(transport)
}

pub fn parse_address(address: &str) -> Result<Address, QFSError> {
    serde_json::from_str(&format!("{:?}", address))
        .map_err(QFSError::from)
}

pub fn get_contract(web3: &Web3<Http>, address: Address) -> Contract<Http> {
    let json: Value = serde_json::from_slice(CONTRACT_JSON_BYTES).expect("Malformed JSON ABI");
    let abi_json = &json["abi"];
    let abi_string = abi_json.to_string();
    let abi_bytes = abi_string.as_bytes();
    Contract::from_json(
        web3.eth(),
        address,
        &abi_bytes
    ).expect("Invalid ABI")
}

fn map_result(result: (String, U256)) -> (IpfsHash, u128) {
    let number = result.1.as_u128();
    let hash = IpfsHash::new(result.0.as_str())
        .expect("Invalid IPFS hash stored in the contract");
    (hash, number)
}

pub fn fetch_revision(contract: &Contract<Http>, address: Address, revision: u128) -> Result<(IpfsHash, u128), QFSError> {
    let revision_uint = U256::try_from(revision).unwrap();
    contract
        .query("getRevision",
               (revision_uint, ),
               address,
               Options::default(),
               None)
        .wait()
        .map_err(QFSError::from)
        .map(map_result)
}

pub fn fetch_last_revision(contract: &Contract<Http>, address: Address) -> Result<(IpfsHash, u128), QFSError> {
    contract
        .query("currentRevision",
               (),
               address,
               Options::default(),
               None)
        .wait()
        .map_err(QFSError::from)
        .map(map_result)
}


#[cfg(test)]
pub mod tests {
    use serde_json::{self, Value};
    use web3::contract::Contract;
    use web3::futures::Future;
    use web3::transports::Http;
    use web3::types::Address;
    use web3::Web3;

    use crate::operations::ethereum;
    use crate::operations::ethereum::{CONTRACT_JSON_BYTES, get_contract, get_web3};

    pub fn coinbase(web3: &Web3<Http>) -> Address {
        web3.eth().coinbase().wait()
            .expect("Could not get the coinbase address. Check the connection with the ethereum node")
    }

    lazy_static! {
        pub static ref TEST_WEB3: Web3<Http> = get_web3("http://127.0.0.1:7545");
        pub static ref TEST_CONTRACT: Contract<Http> = get_contract(&TEST_WEB3, get_contract_address());
    }

    fn get_contract_address() -> Address {
        let json: Value = serde_json::from_slice(CONTRACT_JSON_BYTES).expect("Malformed JSON ABI");
        let contract_address = &json["networks"]["5777"]["address"];
        let address_string = contract_address.as_str().unwrap().to_lowercase();
        ethereum::parse_address(address_string.as_str())
            .expect("Failure parsing the contract address")
    }

    #[test]
    fn fetch_last_revision_without_interaction_should_work() {
        let coinbase = coinbase(&TEST_WEB3);
        let result = ethereum::fetch_last_revision(&TEST_CONTRACT,coinbase).unwrap();
        assert_eq!(result.0.to_string().as_str(), "0000000000000000000000000000000000000000000000");
        assert_eq!(result.1, 0);
    }
}
