#![allow(unused)]

use clap::Command;
use congen::{CompositDescription, Configuration, Description};

// TODO how do I want to handle Vec and or HashMap?
// TODO how do I want to handle transparent wrapper types like Box, Arc, Rc, etc?
// TODO do I want to support additional enum types? If so how?

#[derive(congen::Configuration)]
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

#[derive(congen::Configuration)]
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
    let builder = Config::description(None);
    println!("{:#?}", builder);
    println!();
    if let Description::Composit(builder) = builder {
        for f in builder.fields() {
            println!("{}: {:?}", f.0, f.1);
        }
    }
}

// Generated
// TODO generate clap::Parser based on the ConfigBuilder
