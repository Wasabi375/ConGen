#![allow(unused)]

use clap::Command;
use congen::{CompositDescription, Configuration, ConfigurationDefault, Description};

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
    d: Option<u32>,
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
    if let Description::Composit(builder) = builder {
        for f in builder.fields() {
            println!("{}: {:?}", f.0, f.1);
        }
    }
}

// Generated
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

    fn description(field_name: Option<&'static str>) -> Description {
        let mut fields = Vec::new();
        let mut composites = Vec::new();

        match <u32 as Configuration>::description(Some("a")) {
            Description::Composit(comp) => composites.push(comp),
            Description::Field(field) => fields.push(field),
        }
        match <Option<String> as Configuration>::description(Some("b")) {
            Description::Composit(comp) => composites.push(comp),
            Description::Field(field) => fields.push(field),
        }
        match <bool as Configuration>::description(Some("c")) {
            Description::Composit(comp) => composites.push(comp),
            Description::Field(field) => fields.push(field),
        }
        match <SubConfig as Configuration>::description(Some("some")) {
            Description::Composit(comp) => composites.push(comp),
            Description::Field(field) => fields.push(field),
        }
        match <Option<SubConfig> as Configuration>::description(Some("opt")) {
            Description::Composit(comp) => composites.push(comp),
            Description::Field(field) => fields.push(field),
        }

        CompositDescription {
            field_name,
            type_name: Self::type_name(),
            fields,
            composites,
        }
        .into()
    }

    fn type_name() -> std::borrow::Cow<'static, str> {
        "Config".into()
    }
}

struct ConfigChange {
    a: Option<u32>,
    b: Option<Option<String>>,
    c: Option<bool>,
    // ...
}

// TODO generate clap::Parser based on the ConfigBuilder

impl Configuration for SubConfig {
    type CongenChange = ();

    fn apply_change(&mut self, change: Self::CongenChange) {
        todo!()
    }

    fn description(field_name: Option<&'static str>) -> Description {
        let mut fields = Vec::new();
        let mut composites = Vec::new();

        match <Option<u32> as Configuration>::description(Some("d")) {
            Description::Composit(comp) => composites.push(comp),
            Description::Field(field) => fields.push(field),
        }
        match <Option<u32> as Configuration>::description(Some("e")) {
            Description::Composit(comp) => composites.push(comp),
            Description::Field(field) => fields.push(field),
        }
        // ...

        CompositDescription {
            field_name,
            type_name: Self::type_name(),
            fields,
            composites,
        }
        .into()
    }

    fn default() -> Result<Self, congen::NotSupported> {
        Ok(Self {
            d: <Option<u32> as Configuration>::default()?,
            e: <Option<u32> as Configuration>::default()?,
        })
    }

    fn type_name() -> std::borrow::Cow<'static, str> {
        "SubConfig".into()
    }
}

impl ConfigurationDefault for SubConfig where for<'a> Option<u32>: ConfigurationDefault {}
