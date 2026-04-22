use std::{any::TypeId, str::FromStr};

use crate::{
    ChangeVerb, Configuration, CongenChange, Description, FieldDescription, FromVerbError,
    NotSupported, OptionChange, ParseError,
};

impl<T> Configuration for Option<T>
where
    T: Configuration<CongenChange = Option<T>> + 'static,
    Option<T>: CongenChange,
{
    type CongenChange = OptionChange<Option<T>>;

    fn apply_change(&mut self, change: Self::CongenChange) {
        if let OptionChange::Apply(change) = change {
            *self = change;
        }
    }

    fn description(field_name: &'static str) -> Description {
        // TODO type_id
        T::description(field_name).as_option()
    }

    fn default() -> Result<Self, NotSupported> {
        Ok(None)
    }

    fn unwrap_change(change: Self::CongenChange) -> Result<Self, NotSupported> {
        Ok(change.unwrap())
    }

    fn type_name() -> std::borrow::Cow<'static, str> {
        let mut name = T::type_name();
        name.to_mut().push('?');
        name
    }
}

impl<T> CongenChange for OptionChange<Option<T>>
where
    T: Configuration<CongenChange = Option<T>> + 'static,
    Option<T>: CongenChange,
{
    fn empty() -> Self {
        OptionChange::NoChange
    }

    fn apply_change(&mut self, change: Self) {
        if let OptionChange::Apply(change) = change {
            *self = OptionChange::Apply(change);
        }
    }

    fn from_path_and_verb<'a, P>(mut path: P, verb: ChangeVerb) -> Result<Self, FromVerbError>
    where
        P: Iterator<Item = &'a str>,
    {
        assert!(
            path.next().is_none(),
            "OptionChange<Option<T>> implies this is a field"
        );
        let desc = T::description("");
        assert!(
            matches!(desc, Description::Field(_)),
            "OptionChange<Option<T>> implies this is a field"
        );

        return match verb {
            ChangeVerb::Set(value) => {
                let change = T::parse(&value)??;
                Ok(OptionChange::Apply(Some(change)))
            }
            ChangeVerb::SetAny(value) => {
                let change = value
                    .downcast()
                    .map_err(|_| FromVerbError::DowncastFailed)?;
                Ok(OptionChange::Apply(Some(*change)))
            }
            ChangeVerb::SetFlag => Err(FromVerbError::UnsuportedVerb(verb)),
            ChangeVerb::Unset => Ok(OptionChange::Apply(None)),
            ChangeVerb::UseDefault => {
                if !desc.has_default() {
                    return Err(FromVerbError::UnsuportedVerb(verb));
                }
                let default =
                    T::default().expect("use-default verb used, but default is not implemented");
                Ok(OptionChange::Apply(Some(default)))
            }
        };
    }
}

impl Configuration for bool {
    type CongenChange = Option<bool>;

    fn apply_change(&mut self, change: Self::CongenChange) {
        if let Some(new) = change {
            *self = new
        }
    }

    fn description(field_name: &'static str) -> Description {
        FieldDescription {
            field_name,
            type_name: Self::type_name(),
            type_id: TypeId::of::<Self>(),
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

    fn unwrap_change(change: Self::CongenChange) -> Result<Self, NotSupported> {
        Ok(change.unwrap())
    }

    fn type_name() -> std::borrow::Cow<'static, str> {
        "bool".into()
    }

    fn parse(input: &str) -> Result<Result<Self, crate::ParseError>, NotSupported> {
        match bool::from_str(input) {
            Ok(value) => Ok(Ok(value)),
            Err(_) => Ok(Err(ParseError)),
        }
    }
}

impl CongenChange for Option<bool> {
    fn empty() -> Self {
        None
    }

    fn apply_change(&mut self, change: Self) {
        if let Some(new_change) = change {
            *self = Some(new_change)
        }
    }

    fn from_path_and_verb<'a, P>(mut path: P, verb: ChangeVerb) -> Result<Self, FromVerbError>
    where
        P: Iterator<Item = &'a str>,
    {
        assert!(
            path.next().is_none(),
            "OptionChange<Option<T>> implies this is a field"
        );
        match verb {
            ChangeVerb::Set(unparesd) => Ok(Some(Configuration::parse(&unparesd)??)),
            ChangeVerb::SetAny(value) => Ok(Some(
                *value
                    .downcast()
                    .map_err(|_| FromVerbError::DowncastFailed)?,
            )),
            ChangeVerb::SetFlag => Ok(Some(true)),
            ChangeVerb::Unset => Ok(Some(false)),
            ChangeVerb::UseDefault => Ok(Some(Configuration::default()?)),
        }
    }
}

impl Configuration for String {
    type CongenChange = Option<String>;

    fn apply_change(&mut self, change: Self::CongenChange) {
        if let Some(value) = change {
            *self = value;
        }
    }

    fn description(field_name: &'static str) -> Description {
        FieldDescription {
            field_name,
            type_name: Self::type_name(),
            type_id: TypeId::of::<Self>(),
            is_flag: false,
            allow_unset: true,
            has_default: false,
            cmd_value_hint: clap::ValueHint::Unknown,
        }
        .into()
    }

    fn unwrap_change(change: Self::CongenChange) -> Result<Self, NotSupported> {
        Ok(change.unwrap())
    }

    fn type_name() -> std::borrow::Cow<'static, str> {
        "String".into()
    }

    fn parse(input: &str) -> Result<Result<Self, crate::ParseError>, NotSupported> {
        Ok(Ok(input.to_owned()))
    }
}

impl CongenChange for Option<String> {
    fn empty() -> Self {
        None
    }

    fn apply_change(&mut self, change: Self) {
        if let Some(new_change) = change {
            *self = Some(new_change)
        }
    }

    fn from_path_and_verb<'a, P>(mut path: P, verb: ChangeVerb) -> Result<Self, FromVerbError>
    where
        P: Iterator<Item = &'a str>,
    {
        assert!(
            path.next().is_none(),
            "OptionChange<Option<T>> implies this is a field"
        );
        match verb {
            ChangeVerb::Set(unparesd) => Ok(Some(Configuration::parse(&unparesd)??)),
            ChangeVerb::SetAny(value) => Ok(Some(
                *value
                    .downcast()
                    .map_err(|_| FromVerbError::DowncastFailed)?,
            )),
            ChangeVerb::UseDefault | ChangeVerb::SetFlag | ChangeVerb::Unset => {
                Err(FromVerbError::UnsuportedVerb(verb))
            }
        }
    }
}

impl Configuration for u32 {
    type CongenChange = Option<u32>;

    fn apply_change(&mut self, change: Self::CongenChange) {
        if let Some(value) = change {
            *self = value;
        }
    }

    fn description(field_name: &'static str) -> Description {
        FieldDescription {
            field_name,
            type_name: Self::type_name(),
            type_id: TypeId::of::<Self>(),
            is_flag: false,
            allow_unset: false,
            has_default: false,
            cmd_value_hint: clap::ValueHint::Other,
        }
        .into()
    }

    fn unwrap_change(change: Self::CongenChange) -> Result<Self, NotSupported> {
        Ok(change.unwrap())
    }

    fn type_name() -> std::borrow::Cow<'static, str> {
        "u32".into()
    }

    fn parse(input: &str) -> Result<Result<Self, crate::ParseError>, NotSupported> {
        match u32::from_str(input) {
            Ok(value) => Ok(Ok(value)),
            Err(_) => Ok(Err(ParseError)),
        }
    }
}

impl CongenChange for Option<u32> {
    fn empty() -> Self {
        None
    }

    fn apply_change(&mut self, change: Self) {
        if let Some(new_change) = change {
            *self = Some(new_change)
        }
    }

    fn from_path_and_verb<'a, P>(mut path: P, verb: ChangeVerb) -> Result<Self, FromVerbError>
    where
        P: Iterator<Item = &'a str>,
    {
        assert!(
            path.next().is_none(),
            "OptionChange<Option<T>> implies this is a field"
        );
        match verb {
            ChangeVerb::Set(unparesd) => Ok(Some(Configuration::parse(&unparesd)??)),
            ChangeVerb::SetAny(value) => Ok(Some(
                *value
                    .downcast()
                    .map_err(|_| FromVerbError::DowncastFailed)?,
            )),
            ChangeVerb::UseDefault | ChangeVerb::SetFlag | ChangeVerb::Unset => {
                Err(FromVerbError::UnsuportedVerb(verb))
            }
        }
    }
}
