#[macro_use]
extern crate clap;
extern crate quantumfs;

use clap::App;

mod commands;

fn main() {
    let yaml = load_yaml!("../cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    if let Some(_) = matches.subcommand_matches("mount") {
        commands::mount::mount();
    } else if let Some(_) = matches.subcommand_matches("transaction") {
        commands::transaction::transaction();
    } else if let Some(_) = matches.subcommand_matches("commit") {
        commands::commit::commit();
    } else if let Some(_) = matches.subcommand_matches("push") {
        commands::push::push();
    } else {
        panic!("Invalid command");
    }
}
