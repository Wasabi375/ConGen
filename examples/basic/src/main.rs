use clap::Parser;
use congen::Configuration;

// TODO how do I want to handle Vec and or HashMap?
// TODO how do I want to handle transparent wrapper types like Box, Arc, Rc, etc?

// TODO remove Debug implementations from generated structs

#[derive(Configuration, Debug)]
struct Config {
    a: u32,
    #[congen(default)]
    b: Option<String>,
    b2: String,
    c: bool,

    #[congen(default)]
    sub: SubConfig,
    #[congen(default)]
    opt: Option<SubConfig>,
}

#[derive(Configuration, Debug)]
pub struct SubConfig {
    #[congen(default = 5)]
    d: u32,
    #[congen(default)]
    e: Option<u32>,
}

fn main() {
    // config a set 10
    // config b use-default
    // config b set "foo"
    // config b unset
    // config b2 set "foo"
    // config c set
    // config c unset
    // config sub.d set 5
    // config sub.e set 42
    // config opt.e use-default
    // config opt.d set 2
    // config opt.d use-default
    // config opt unset
    // config opt use-default
    // config sub use-default

    let args = cli_test::TestCli::parse();
    match args.command {
        cli_test::Commands::DoSomething => {
            println!("{:#?}", Config::description(""));
        }
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
            a: 1,
            b: Some("foo".to_string()),
            b2: "test".to_string(),
            c: false,
            sub: SubConfig { d: 2, e: None },
            opt: None,
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
    fn config_a_set() {
        let mut config = base_config();
        apply_change(&mut config, &["config", "a", "set", "10"]);
        assert_eq!(config.a, 10);
    }

    #[test]
    fn config_b_use_default() {
        let mut config = base_config();
        apply_change(&mut config, &["config", "b", "use-default"]);
        assert!(config.b.is_none());
    }

    #[test]
    fn config_b_set() {
        let mut config = base_config();
        config.b = Some("shake".to_string());
        apply_change(&mut config, &["config", "b", "set", "foo"]);
        assert_eq!(config.b, Some("foo".to_string()));
    }

    #[test]
    fn config_b_unset() {
        let mut config = base_config();
        apply_change(&mut config, &["config", "b", "unset"]);
        assert!(config.b.is_none());
    }

    #[test]
    fn config_b2_set() {
        let mut config = base_config();
        apply_change(&mut config, &["config", "b2", "set", "foo"]);
        assert_eq!(config.b2, "foo");
    }

    #[test]
    fn config_c_set() {
        let mut config = base_config();
        apply_change(&mut config, &["config", "c", "set"]);
    }

    #[test]
    fn config_c_unset() {
        let mut config = base_config();
        config.c = true;
        apply_change(&mut config, &["config", "c", "unset"]);
        assert!(!config.c);
    }

    #[test]
    fn config_sub_d_set() {
        let mut config = base_config();
        apply_change(&mut config, &["config", "sub.d", "set", "5"]);
        assert_eq!(config.sub.d, 5);
    }

    #[test]
    fn config_sub_e_set() {
        let mut config = base_config();
        apply_change(&mut config, &["config", "sub.e", "set", "42"]);
        assert_eq!(config.sub.e, Some(42));
    }

    #[test]
    fn config_opt_e_use_default() {
        let mut config = base_config();
        apply_change(&mut config, &["config", "opt.e", "use-default"]);
        let opt = config.opt.as_ref().expect("opt created");
        assert_eq!(opt.d, 5);
        assert!(opt.e.is_none());
    }

    #[test]
    fn config_opt_d_set() {
        let mut config = base_config();
        apply_change(&mut config, &["config", "opt.d", "set", "2"]);
        let opt = config.opt.as_ref().expect("opt created");
        assert_eq!(opt.d, 2);
        assert!(opt.e.is_none());
    }

    #[test]
    fn config_opt_d_use_default() {
        let mut config = base_config();
        config.opt = Some(SubConfig { d: 99, e: Some(1) });
        apply_change(&mut config, &["config", "opt.d", "use-default"]);
        let opt = config.opt.as_ref().expect("opt still present");
        assert_eq!(opt.d, 5);
        assert_eq!(opt.e, Some(1));
    }

    #[test]
    fn config_opt_unset() {
        let mut config = base_config();
        config.opt = Some(SubConfig { d: 99, e: None });
        apply_change(&mut config, &["config", "opt", "unset"]);
        assert!(config.opt.is_none());
    }

    #[test]
    fn config_opt_use_default() {
        let mut config = base_config();
        config.opt = Some(SubConfig { d: 99, e: Some(2) });
        apply_change(&mut config, &["config", "opt", "use-default"]);
        let opt = config.opt.as_ref().expect("opt reset");
        assert_eq!(opt.d, 5);
        assert!(opt.e.is_none());
    }

    #[test]
    fn config_sub_use_default() {
        let mut config = base_config();
        config.sub = SubConfig { d: 99, e: Some(3) };
        apply_change(&mut config, &["config", "sub", "use-default"]);
        assert_eq!(config.sub.d, 5);
        assert!(config.sub.e.is_none());
    }
}
