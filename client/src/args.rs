use clap::{App, ArgMatches};
use yaml_rust::Yaml;

lazy_static! {
    static ref YAML: Yaml = load_yaml!("../cli.yaml").clone();

    pub static ref ARGS: ArgMatches<'static> = {
        App::from_yaml(&YAML).get_matches()
    };
}
