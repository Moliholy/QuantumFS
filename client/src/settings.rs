use config::Config;

use crate::args::ARGS;
use std::fs;

lazy_static! {
    pub static ref SETTINGS: Config = init();
}

fn init() -> Config {
    fs::create_dir_all("~/.qfs/")
        .expect("Failure creating the ~/.qfs directory");
    let mut config = Config::default();
    // set default variables first
    config
        .set_default("web3", "http://127.0.0.1:8545").unwrap()
        .set_default("ipfs_gateway_server", "127.0.0.1").unwrap()
        .set_default("ipfs_gateway_port", 5001).unwrap();
    // Add the custom configuration file, if present
    if let Some(config_file) = ARGS.value_of("config") {
        config.merge(config::File::with_name(config_file))
            .expect(format!("Error loading the configuration from {}", config_file).as_str());
    } else {
        // load settings in ~/.qfs/settings, if present
        let file = config::File::with_name("~/.qfs/settings").required(false);
        config.merge(file).unwrap();
    }

    // Add settings from the environment (with a prefix of QFS)
    config.merge(config::Environment::with_prefix("QFS_")).unwrap();

    // Add the client's ethereum address if passed as a parameter
    if let Some(address) = ARGS.value_of("address") {
        config.set("address", address).unwrap();
    }
    // Add the contract's ethereum address if passed as a parameter
    if let Some(address) = ARGS.value_of("contract") {
        config.set("contract", address).unwrap();
    }
    // Add the mountpoint if passed as a parameter
    if let Some(mountpoint) = ARGS.value_of("mountpoint") {
        config.set("mountpoint", mountpoint).unwrap();
    }
    // Add web3 URL if passed as a parameter
    if let Some(web3) = ARGS.value_of("web3") {
        config.set("web3", web3).unwrap();
    }
    // Add IPFS server if passed as a parameter
    if let Some(ipfs_server) = ARGS.value_of("ipfs-server") {
        config.set("ipfs-server", ipfs_server).unwrap();
    }
    // Add IPFS server if passed as a parameter
    if let Some(ipfs_port) = ARGS.value_of("ipfs-port") {
        config.set("ipfs-port", ipfs_port).unwrap();
    }
    config
}
