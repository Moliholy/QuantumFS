use std::ffi::OsStr;

use fuse_mt::FuseMT;
use web3::types::Address;

use quantumfs::models::repository::Repository;

use crate::cache::CACHE;
use crate::fs::QuantumFS;
use crate::settings::SETTINGS;

fn load_repository() -> Repository {
    let client_address = SETTINGS.get::<Address>("address")
        .expect("User ethereum address not provided");
    let contract_address = SETTINGS.get::<Address>("contract")
        .expect("Contract ethereum address not provided");
    let web3_url = SETTINGS.get::<String>("web3")
        .expect("Web3 URL not provided");
    let ipfs_server = SETTINGS.get::<String>("ipfs-server")
        .expect("IPFS server not provided");
    let ipfs_port = SETTINGS.get::<u16>("ipfs-port")
        .expect("IPFS port not provided");
    Repository::new(
        client_address,
        contract_address,
        CACHE.data_dir().as_path(),
        &web3_url,
        &ipfs_server,
        ipfs_port,
    )
}

pub fn mount() {
    let mountpoint = SETTINGS.get::<String>("mountpoint")
        .expect("Mount point not provided");
    let repository = load_repository();
    let qfs = QuantumFS::new(repository)
        .expect("Failure mounting the file system");
    let options = ["-o", "ro", "-o", "fsname=qfs"]
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&OsStr>>();
    let filesystem = FuseMT::new(qfs, 0);
    fuse_mt::mount(filesystem, &mountpoint, &options)
        .expect("Failure mounting the file system");
}
