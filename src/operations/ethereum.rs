use std::convert::TryFrom;
use std::fs;

use serde_json;
use serde_json::Value;
use web3::contract::{Contract, Options};
use web3::futures::Future;
use web3::transports::Http;
use web3::types::{Address, U256};
use web3::Web3;

use crate::errors::QFSError;
use crate::types::ipfs::IpfsHash;

static CONTRACT_ADDRESS: &str = "2d1FF468102Ba7742b29E72F1e652a465Ce527B1";
static CONTRACT_ABI_PATH: &str = "./ethereum/build/contracts/QuantumFS.json";

lazy_static! {
    pub static ref WEB3: Web3<Http> = {
        let url = "http://127.0.0.1:7545/";
        let (event_loop, transport) = web3::transports::Http::new(url)
            .expect(format!("Web3 could not be loaded from {}", url).as_ref());
        event_loop.into_remote();
        Web3::new(transport)
    };

    static ref CONTRACT: Contract<Http> = {
        let bytes = fs::read(CONTRACT_ABI_PATH)
            .expect(format!("Contract JSON ABI not found under {}", CONTRACT_ABI_PATH).as_ref());
        let json: Value = serde_json::from_slice(&bytes).expect("Malformed JSON ABI");
        let abi_json = &json["abi"];
        let abi_string = abi_json.to_string();
        let abi_bytes = abi_string.as_bytes();
        let contract_address: Address = CONTRACT_ADDRESS.parse().unwrap();
        Contract::from_json(
            WEB3.eth(),
            contract_address,
            &abi_bytes,
        ).expect("Invalid ABI")
    };
}

pub fn coinbase() -> Address {
    WEB3.eth().coinbase().wait()
        .expect("Could not get the coinbase address. Check the connection with the ethereum node")
}

fn map_result(result: (String, U256)) -> (IpfsHash, u128) {
    let number = result.1.as_u128();
    let hash = IpfsHash::new(result.0.as_str())
        .expect("Invalid IPFS hash stored in the contract");
    (hash, number)
}

pub fn fetch_revision(address: Address, revision: u128) -> Result<(IpfsHash, u128), QFSError> {
    let revision_uint = U256::try_from(revision).unwrap();
    CONTRACT
        .query("getRevision",
               (revision_uint, ),
               address,
               Options::default(),
               None)
        .wait()
        .map_err(QFSError::from)
        .map(map_result)
}

pub fn fetch_last_revision(address: Address) -> Result<(IpfsHash, u128), QFSError> {
    CONTRACT
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
mod tests {
    use web3::futures::Future;

    use crate::operations::ethereum::fetch_last_revision;
    use crate::operations::ethereum::WEB3;

    #[test]
    fn fetch_last_revision_without_interaction_should_fail() {
        let accounts = WEB3
            .eth()
            .accounts()
            .wait()
            .unwrap();
        let result = fetch_last_revision(accounts[0]).unwrap();
        assert_eq!(result.0.to_string().as_str(), "0000000000000000000000000000000000000000000000");
        assert_eq!(result.1, 0);
    }
}
