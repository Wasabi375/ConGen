use clap::Command;

pub trait Configuration {
    type CongenChange;

    fn apply_change(&mut self, change: Self::CongenChange);
    fn command() -> Command;
}

pub trait ConfigValue: Sized {
    fn default() -> Option<Self>;
    fn unset_value() -> Option<Self>;
    fn flag_value() -> Option<Self>;
}

pub trait ConfigValuePartialDefault: ConfigValue {
    fn default() -> Self;
}
