use std::str::FromStr;

use crate::{
    ChangeVerb, Configuration, CongenChange, Description, FieldDescription, NotSupported,
    ParseError, VerbError,
};

impl Configuration for bool {
    type CongenChange = Option<bool>;

    fn apply_change_with_default(
        &mut self,
        change: Self::CongenChange,
        _default: Option<fn() -> Self>,
    ) {
        if let Some(new) = change {
            *self = new
        }
    }

    fn description(field_name: &'static str) -> Description {
        FieldDescription {
            field_name,
            type_name: Self::type_name(),
            is_flag: true,
            allow_unset: true,
            has_default: false,
            cmd_value_hint: clap::ValueHint::Unknown,
        }
        .into()
    }

    fn default() -> Result<Self, crate::NotSupported> {
        Ok(false)
    }

    fn type_name() -> std::borrow::Cow<'static, str> {
        std::any::type_name::<Self>().into()
    }
}

impl CongenChange for Option<bool> {
    type Configuration = bool;

    fn empty() -> Self {
        None
    }

    fn parse(input: &str) -> Result<Result<Self, ParseError>, NotSupported> {
        match bool::from_str(input) {
            Ok(value) => Ok(Ok(Some(value))),
            Err(e) => Ok(Err(ParseError(e.to_string()))),
        }
    }

    fn apply_change(&mut self, change: Self) {
        if let Some(new_change) = change {
            *self = Some(new_change)
        }
    }

    fn from_path_and_verb<'a, P>(mut path: P, verb: ChangeVerb) -> Result<Self, VerbError>
    where
        P: Iterator<Item = &'a str>,
    {
        assert!(
            path.next().is_none(),
            "OptionChange<Option<T>> implies this is a field"
        );
        match verb {
            ChangeVerb::Set(unparesd) => Ok(Self::parse(&unparesd)??),
            ChangeVerb::SetAny(value) => Ok(Some(
                *value.downcast().map_err(|_| VerbError::DowncastFailed)?,
            )),
            ChangeVerb::SetFlag => Ok(Some(true)),
            ChangeVerb::Unset => Ok(Some(false)),
            ChangeVerb::UseDefault => Ok(Some(Configuration::default()?)),
        }
    }

    fn unwrap_field(self) -> Result<Self::Configuration, Self> {
        Ok(self.unwrap())
    }
}

impl Configuration for String {
    type CongenChange = Option<String>;

    fn apply_change_with_default(
        &mut self,
        change: Self::CongenChange,
        _default: Option<fn() -> Self>,
    ) {
        if let Some(value) = change {
            *self = value;
        }
    }

    fn description(field_name: &'static str) -> Description {
        FieldDescription {
            field_name,
            type_name: Self::type_name(),
            is_flag: false,
            allow_unset: false,
            has_default: false,
            cmd_value_hint: clap::ValueHint::Unknown,
        }
        .into()
    }
}

impl CongenChange for Option<String> {
    type Configuration = String;

    fn empty() -> Self {
        None
    }

    fn parse(input: &str) -> Result<Result<Self, ParseError>, NotSupported> {
        Ok(Ok(Some(input.to_owned())))
    }

    fn apply_change(&mut self, change: Self) {
        if let Some(new_change) = change {
            *self = Some(new_change)
        }
    }

    fn from_path_and_verb<'a, P>(mut path: P, verb: ChangeVerb) -> Result<Self, VerbError>
    where
        P: Iterator<Item = &'a str>,
    {
        assert!(
            path.next().is_none(),
            "OptionChange<Option<T>> implies this is a field"
        );
        match verb {
            ChangeVerb::Set(unparesd) => Ok(Self::parse(&unparesd)??),
            ChangeVerb::SetAny(value) => Ok(Some(
                *value.downcast().map_err(|_| VerbError::DowncastFailed)?,
            )),
            ChangeVerb::UseDefault | ChangeVerb::SetFlag | ChangeVerb::Unset => {
                Err(VerbError::UnsupportedVerb(verb))
            }
        }
    }

    fn unwrap_field(self) -> Result<Self::Configuration, Self> {
        Ok(self.unwrap())
    }
}

impl Configuration for u32 {
    type CongenChange = Option<u32>;

    fn apply_change_with_default(
        &mut self,
        change: Self::CongenChange,
        _default: Option<fn() -> Self>,
    ) {
        if let Some(value) = change {
            *self = value;
        }
    }

    fn description(field_name: &'static str) -> Description {
        FieldDescription {
            field_name,
            type_name: Self::type_name(),
            is_flag: false,
            allow_unset: false,
            has_default: false,
            cmd_value_hint: clap::ValueHint::Other,
        }
        .into()
    }
}

impl CongenChange for Option<u32> {
    type Configuration = u32;

    fn empty() -> Self {
        None
    }

    fn parse(input: &str) -> Result<Result<Self, ParseError>, NotSupported> {
        match u32::from_str(input) {
            Ok(value) => Ok(Ok(Some(value))),
            Err(e) => Ok(Err(ParseError(e.to_string()))),
        }
    }

    fn apply_change(&mut self, change: Self) {
        if let Some(new_change) = change {
            *self = Some(new_change)
        }
    }

    fn from_path_and_verb<'a, P>(mut path: P, verb: ChangeVerb) -> Result<Self, VerbError>
    where
        P: Iterator<Item = &'a str>,
    {
        assert!(
            path.next().is_none(),
            "OptionChange<Option<T>> implies this is a field"
        );
        match verb {
            ChangeVerb::Set(unparesd) => Ok(Self::parse(&unparesd)??),
            ChangeVerb::SetAny(value) => Ok(Some(
                *value.downcast().map_err(|_| VerbError::DowncastFailed)?,
            )),
            ChangeVerb::UseDefault | ChangeVerb::SetFlag | ChangeVerb::Unset => {
                Err(VerbError::UnsupportedVerb(verb))
            }
        }
    }

    fn unwrap_field(self) -> Result<Self::Configuration, Self> {
        Ok(self.unwrap())
    }
}
