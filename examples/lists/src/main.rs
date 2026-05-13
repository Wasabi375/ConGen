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
        cli_test::Commands::DoSomething => {
            println!("{:#?}", Config::description(""));
        }
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

#[cfg(test)]
mod test {
    use clap::{Command, FromArgMatches};
    use congen::{Configuration, CongenClap};

    use crate::{Config, SubConfig};

    fn test_command() -> Command {
        CongenClap::<Config>::create_cmd("config")
    }

    fn base_config() -> Config {
        Config {
            field: 67,
            comp: SubConfig { a: 42, b: 69 },
            field_list: vec![1, 2, 3],
        }
    }

    fn apply_change(config: &mut Config, args: &[&str]) {
        let matches = test_command().get_matches_from(args);
        let change = CongenClap::<Config>::from_arg_matches(&matches)
            .expect("parse command")
            .into_change();
        config.apply_change(change);
    }

    #[test]
    fn config_field_list_append() {
        let mut config = base_config();
        apply_change(&mut config, &["config", "field_list", "append", "10"]);
        assert_eq!(config.field_list, vec![1, 2, 3, 10]);
    }

    #[test]
    fn config_field_list_update_valid_key() {
        let mut config = base_config();
        apply_change(&mut config, &["config", "field_list", "update", "1", "10"]);
        assert_eq!(config.field_list, vec![1, 10, 3]);
    }

    #[test]
    #[should_panic(expected = "User specified index")]
    fn config_field_list_update_out_of_range_key_panics() {
        let mut config = base_config();
        apply_change(
            &mut config,
            &["config", "field_list", "update", "99", "10"],
        );
    }

    #[test]
    #[should_panic(expected = "User specified index")]
    fn config_field_list_update_empty_list_key_zero_panics() {
        let mut config = base_config();
        config.field_list.clear();
        apply_change(&mut config, &["config", "field_list", "update", "0", "10"]);
    }

    #[test]
    fn config_field_list_remove_valid_key() {
        let mut config = base_config();
        apply_change(&mut config, &["config", "field_list", "remove", "1"]);
        assert_eq!(config.field_list, vec![1, 3]);
    }

    #[test]
    #[should_panic(expected = "User specified index")]
    fn config_field_list_remove_out_of_range_key_panics() {
        let mut config = base_config();
        apply_change(&mut config, &["config", "field_list", "remove", "99"]);
    }

    #[test]
    #[should_panic(expected = "User specified index")]
    fn config_field_list_remove_empty_list_key_zero_panics() {
        let mut config = base_config();
        config.field_list.clear();
        apply_change(&mut config, &["config", "field_list", "remove", "0"]);
    }

    #[test]
    fn config_field_list_empty() {
        let mut config = base_config();
        apply_change(&mut config, &["config", "field_list", "empty"]);
        assert!(config.field_list.is_empty());
    }
}
