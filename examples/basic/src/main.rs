#![allow(unused)]

use clap::{Command, Parser};
use congen::{CompositDescription, Configuration, Description};

// TODO how do I want to handle Vec and or HashMap?
// TODO how do I want to handle transparent wrapper types like Box, Arc, Rc, etc?
// TODO do I want to support additional enum types? If so how?

#[derive(congen::Configuration, Debug)]
struct Config {
    // something default 13
    a: u32,
    // something default "foo"
    b: Option<String>,
    b2: String,
    c: bool,

    sub: SubConfig,
    // opt: Option<SubConfig>,
}

#[derive(congen::Configuration, Debug)]
pub struct SubConfig {
    #[congen(default = 5)]
    d: u32,
    #[congen(default)]
    e: Option<u32>,
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
    // config opt.d set 2
    // config opt unset

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
                // opt: None,
            };
            config.apply_change(congen_clap.into_change());

            println!("{config:?}",)
        }
    }
}

mod to_generate {

    // TODO I probs need to generate
    // impl CongenChange for Option<SubConfigChange>
    // to handle optional subconfigs
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
