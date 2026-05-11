#![allow(unused)]

use clap::{Command, Parser};
use congen::{CompositDescription, Configuration, Description, ValueEnumConfiguration};

#[derive(Configuration, Debug)]
struct Config {
    field: u32,
    comp: SubConfig,
    field_list: Vec<u32>,
    // TODO
    // field_map_string: HashMap<String, u32>
    // field_map_int: HashMap<u32, u32>
    // comp_list: Vec<SubConfig>
    // comp_map: HashMap<u32, SubConfig>
    // field_map_enum
}

#[derive(Configuration, Debug)]
struct SubConfig {
    a: u32,
    b: u32,
}

#[derive(ValueEnumConfiguration, clap::ValueEnum, Debug, Clone, Copy)]
enum Variants {
    A,
    B,
    C,
}

fn main() {
    let args = cli_test::TestCli::parse();
    match args.command {
        cli_test::Commands::DoSomething => {}
        cli_test::Commands::Config(congen_clap) => {
            let mut config = Config {
                field: 67,
                comp: SubConfig { a: 42, b: 69 },
                field_list: Vec::new(),
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
