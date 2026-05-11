use crate::{
    ChangeVerb, Configuration, CongenChange, Description, FieldDescription, NotSupported,
    ParseError, VerbError,
};

pub trait ValueEnumConfiguration: clap::ValueEnum + Clone + core::fmt::Debug + 'static {}

impl<T> Configuration for T
where
    T: ValueEnumConfiguration,
{
    type CongenChange = ValueEnumChange<T>;

    fn apply_change_with_default(
        &mut self,
        change: Self::CongenChange,
        _default: Option<fn() -> Self>,
    ) {
        if let ValueEnumChange::Some(value) = change {
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

#[derive(Debug, Clone, Default)]
pub enum ValueEnumChange<T> {
    Some(T),
    #[default]
    NoChange,
}

impl<T> CongenChange for ValueEnumChange<T>
where
    T: ValueEnumConfiguration,
{
    type Configuration = T;

    fn empty() -> Self {
        Self::NoChange
    }

    fn parse(input: &str) -> Result<Result<Self, ParseError>, NotSupported> {
        match T::from_str(input, false) {
            Ok(value) => Ok(Ok(Self::Some(value))),
            Err(error) => Ok(Err(ParseError(error))),
        }
    }

    fn apply_change(&mut self, change: Self) {
        if let Self::Some(new_change) = change {
            *self = Self::Some(new_change);
        }
    }

    fn from_path_and_verb<'a, P>(mut path: P, verb: ChangeVerb) -> Result<Self, VerbError>
    where
        P: Iterator<Item = &'a str>,
    {
        assert!(path.next().is_none(), "Option<T> implies this is a field");
        match verb {
            ChangeVerb::Set(unparsed) => Ok(Self::parse(&unparsed)??),
            ChangeVerb::SetAny(value) => Ok(Self::Some(
                *value.downcast().map_err(|_| VerbError::DowncastFailed)?,
            )),
            ChangeVerb::UseDefault | ChangeVerb::SetFlag | ChangeVerb::Unset => {
                Err(VerbError::UnsupportedVerb(verb))
            }
        }
    }

    fn unwrap_field(self) -> Result<Self::Configuration, Self> {
        match self {
            Self::Some(value) => Ok(value),
            Self::NoChange => Err(Self::NoChange),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::iter::empty;

    extern crate self as congen;

    use super::*;

    #[derive(
        clap::ValueEnum, congen_derive::ValueEnumConfiguration, Debug, Clone, PartialEq, Eq,
    )]
    enum Mode {
        Fast,
        Safe,
    }

    #[test]
    fn value_enum_description_is_field() {
        let desc = Mode::description("mode");
        let Description::Field(field) = desc else {
            panic!("ValueEnum should map to FieldDescription");
        };

        assert_eq!(field.field_name, "mode");
        assert_eq!(field.type_name, Mode::type_name());
        assert!(!field.is_flag);
        assert!(!field.allow_unset);
        assert!(!field.has_default);
    }

    #[test]
    fn value_enum_change_set_parses() {
        let change = <ValueEnumChange<Mode> as CongenChange>::from_path_and_verb(
            empty(),
            ChangeVerb::Set("fast".to_owned()),
        )
        .expect("parses enum change");

        assert!(matches!(change, ValueEnumChange::Some(Mode::Fast)));
    }
}
