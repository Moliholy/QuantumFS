use web3::Web3;
use web3::futures::Future;
use web3::types::Address;
use web3::contract::{Contract, Error, Options};
use web3::transports::Http;
use std::fs;
use serde_json;
use serde_json::Value;
use crate::types::ipfs::IpfsHash;


static CONTRACT_ADDRESS: &str = "93319F0d80bF17A6689947386b36A7e76582500F";
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

pub fn fetch_last_revision(address: Address) -> Result<IpfsHash, Error> {
    CONTRACT
        .query("currentRevision", (address,), None, Options::default(), None)
        .wait()
        .map(|hash: String| IpfsHash::new(hash.as_str())
            .expect("Invalid IPFS hash stored in the contract"))
}


#[cfg(test)]
mod tests {
    use crate::operations::ethereum::fetch_last_revision;
    use crate::operations::ethereum::WEB3;
    use web3::futures::Future;

    #[test]
    fn fetch_last_revision_without_interaction_should_fail() {
        let accounts = WEB3
            .eth()
            .accounts()
            .wait()
            .unwrap();
        let last_revision = fetch_last_revision(accounts[0]);
        assert!(last_revision.is_err());
    }
}
