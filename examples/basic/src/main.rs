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
    // #[congen(default = 5)]
    d: u32,
    #[congen(default)]
    e: Option<u32>,
}

fn main() {
    // config use-default a
    // config set a 10
    // config use-default b
    // config set b "foo"
    // config unset b
    // config set c
    // config unset c
    // config set sub.d 5
    // config set sub.e 42
    // config set opt.d 2
    // config unset optg

    // let builder = Config::description("");
    // println!("{:#?}", builder);
    // println!();
    // if let Description::Composit(builder) = builder {
    //     for f in builder.fields() {
    //         println!("{}: {:?}", f.0, f.1);
    //     }
    // }

    let args = cli_test::TestCli::parse();
    match args.command {
        cli_test::Commands::DoSomething => todo!(),
        cli_test::Commands::Config(congen_clap) => {
            let mut config = Config {
                a: 1,
                b: Some("foo".to_string()),
                b2: "test".to_string(),
                c: false,
                sub: SubConfig { d: 2, e: None },
            };
            config.apply_change(congen_clap.into_change());

            println!("{config:?}",)
        }
    }
}

mod to_generate {
    use std::{
        any::{Any, TypeId},
        mem::{self, ManuallyDrop},
    };

    use congen::{
        CompositDescription, Configuration, CongenChange, Description, FieldDescription,
        NotSupported, OptionChange,
    };

    use crate::{Config, ConfigChange, SubConfig, SubConfigChange};

    impl CongenChange for ConfigChange {
        fn empty() -> Self {
            ConfigChange {
                a: CongenChange::empty(),
                b: CongenChange::empty(),
                b2: CongenChange::empty(),
                c: CongenChange::empty(),
                sub: CongenChange::empty(),
            }
        }

        fn apply_change(&mut self, change: Self) {
            CongenChange::apply_change(&mut self.a, change.a);
            CongenChange::apply_change(&mut self.b, change.b);
            CongenChange::apply_change(&mut self.b2, change.b2);
            CongenChange::apply_change(&mut self.c, change.c);
            CongenChange::apply_change(&mut self.sub, change.sub);
        }

        fn from_path_and_verb<'a, P>(
            mut path: P,
            verb: congen::ChangeVerb,
        ) -> Result<Self, congen::FromVerbError>
        where
            P: Iterator<Item = &'a str>,
        {
            let field_name = path.next();
            eprintln!("ConfigChange from field({field_name:?}) and verb({verb:?})");
            let mut change = Self::empty();
            match field_name {
                Some("a") => CongenChange::apply_change(
                    &mut change.a,
                    <u32 as Configuration>::CongenChange::from_path_and_verb(path, verb)?,
                ),
                Some("b") => CongenChange::apply_change(
                    &mut change.b,
                    <Option<String> as Configuration>::CongenChange::from_path_and_verb(
                        path, verb,
                    )?,
                ),
                Some("b2") => CongenChange::apply_change(
                    &mut change.b2,
                    <String as Configuration>::CongenChange::from_path_and_verb(path, verb)?,
                ),
                Some("c") => CongenChange::apply_change(
                    &mut change.c,
                    <bool as Configuration>::CongenChange::from_path_and_verb(path, verb)?,
                ),
                Some("sub") => {
                    eprintln!("why");
                    CongenChange::apply_change(
                        &mut change.sub,
                        <SubConfig as Configuration>::CongenChange::from_path_and_verb(path, verb)?,
                    )
                }
                None => {
                    todo!()
                }
                Some(_) => return Err(congen::FromVerbError::InvalidPath),
            };
            Ok(change)
        }
    }

    impl CongenChange for SubConfigChange {
        fn empty() -> Self {
            SubConfigChange {
                d: None,
                e: OptionChange::NoChange,
            }
        }

        fn apply_change(&mut self, change: Self) {
            // TODO analog to ConfigChange
        }

        fn from_path_and_verb<'a, P>(
            path: P,
            verb: congen::ChangeVerb,
        ) -> Result<Self, congen::FromVerbError>
        where
            P: Iterator<Item = &'a str>,
        {
            todo!()
        }
    }
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
