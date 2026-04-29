#![allow(unused)]

use clap::{Command, Parser};
use congen::{CompositDescription, Configuration, Description};

// TODO how do I want to handle Vec and or HashMap?
// TODO how do I want to handle transparent wrapper types like Box, Arc, Rc, etc?

// TODO remove Debug implementations from generated structs

#[derive(congen::Configuration, Debug)]
struct Config {
    // something default 13
    a: u32,
    // something default "foo"
    b: Option<String>,
    b2: String,
    c: bool,

    #[congen(default)] // FIXME default is not working here
    sub: SubConfig,
    #[congen(default)]
    opt: Option<SubConfig>,
}

#[derive(congen::Configuration, Debug)]
pub struct SubConfig {
    #[congen(default = 5)]
    d: u32,
    #[congen(default)]
    e: Option<u32>,
}

fn sub_config_default() -> SubConfigChange {
    use congen::{ChangeVerb, CongenChange};
    let mut d =
        SubConfigChange::from_path_and_verb(["d"].into_iter(), ChangeVerb::UseDefault).unwrap();
    let e = SubConfigChange::from_path_and_verb(["e"].into_iter(), ChangeVerb::UseDefault).unwrap();

    d.apply_change(e);
    d
}

fn main() {
    // config a use-default
    // config a set 10
    // config b use-default
    // config b set "foo"
    // config b unset
    // config c set
    // config c unset
    // config sub.d set 5
    // config sub.e set 42
    // config opt.e use-default
    // config opt.d set 2
    // config opt.d use-default
    // config opt.d unset
    // config opt unset
    // config opt use-default

    let args = cli_test::TestCli::parse();
    match args.command {
        cli_test::Commands::DoSomething => {}
        cli_test::Commands::Config(congen_clap) => {
            let mut config = Config {
                a: 1,
                b: Some("foo".to_string()),
                b2: "test".to_string(),
                c: false,
                sub: SubConfig { d: 2, e: None },
                opt: None,
            };
            dbg!(&mut config).apply_change(dbg!(congen_clap.into_change()));

            println!("{config:#?}",)
        }
    }
}

mod cli_test {
    use clap::{Parser, Subcommand};
    use congen::CongenClap;

    use crate::Config;

    #[derive(Parser)]
    pub struct TestCli {
        #[command(subcommand)]
        pub command: Commands,
    }

    #[derive(Subcommand)]
    pub enum Commands {
        DoSomething,
        Config(CongenClap<Config>),
    }
}
