#[macro_use]
extern crate clap;
extern crate config;
extern crate fuse_mt;
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate quantumfs;
extern crate time;
extern crate web3;
extern crate yaml_rust;

use crate::args::ARGS;

mod commands;
mod fs;
mod settings;
mod args;

fn main() {
    if let Some(_) = ARGS.subcommand_matches("mount") {
        commands::mount::mount();
    } else if let Some(_) = ARGS.subcommand_matches("transaction") {
        commands::transaction::transaction();
    } else if let Some(_) = ARGS.subcommand_matches("commit") {
        commands::commit::commit();
    } else if let Some(_) = ARGS.subcommand_matches("push") {
        commands::push::push();
    } else {
        panic!("Invalid command");
    }
}
