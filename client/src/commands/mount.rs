use std::ffi::OsStr;

use quantumfs::models::repository::Repository;

use crate::fs::QuantumFS;
use crate::settings::SETTINGS;
use web3::types::Address;

fn load_repository() -> Repository {
    let client_address = SETTINGS.get::<Address>("address")
        .expect("User ethereum address not provided");
    let contract_address = SETTINGS.get::<Address>("contract")
        .expect("Contract ethereum address not provided");
    let web3_url = SETTINGS.get::<String>("web")
        .expect("Web3 URL not provided");
    let ipfs_server = SETTINGS.get::<String>("ipfs-server")
        .expect("IPFS server not provided");
    let ipfs_port = SETTINGS.get::<u16>("ipfs-port")
        .expect("IPFS port not provided");
    Repository::new(
        client_address,
        contract_address,
        &web3_url,
        &ipfs_server,
        ipfs_port,
    )
}

pub fn mount() {
    let mountpoint = SETTINGS.get::<String>("mountpoint")
        .expect("Mount point not provided");
    let repository = load_repository();
    let file_system = QuantumFS::new(repository);
    let options = ["-o", "ro", "-o", "fsname=qfs"]
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&OsStr>>();
    fuse::mount(file_system, &mountpoint, &options)
        .expect("Failure mounting the file system");
}
