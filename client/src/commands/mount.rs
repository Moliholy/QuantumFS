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
    Repository::new(client_address, contract_address)
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
