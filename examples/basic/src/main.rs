#![allow(unused)]

use clap::Command;
use congen::Configuration;

struct Config {
    // something default 13
    a: u32,
    // something default "foo"
    b: Option<String>,
    c: bool,

    sub: SubConfig,

    opt: Option<SubConfig>,
}

struct SubConfig {
    d: i64,
    e: i32,
}

fn main() {
    // config use-default a
    // config set a 10
    // config use-default b
    // config set b "foo"
    // config unset b
    // config set c
    // config set c true
    // config set c false
    // config unset c
    // config set sub.d 5
    // config set sub.e 42
    // config set opt.d 2
    // config unset opt
    println!("Hello, world!");
}

// Gnerated
impl Configuration for Config {
    type CongenChange = ConfigChange;

    fn apply_change(&mut self, change: ConfigChange) {
        if let Some(a) = change.a {
            self.a = a;
        }
        if let Some(b) = change.b {
            self.b = b;
        }
        if let Some(c) = change.c {
            self.c = c;
        }
        // ...
    }

    fn command() -> Command {
        todo!()
    }
}

struct ConfigChange {
    a: Option<u32>,
    b: Option<Option<String>>,
    c: Option<bool>,
    // ...
}
