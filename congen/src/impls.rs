use core::iter::empty;
use std::{any::Any, mem::MaybeUninit, str::FromStr};

use crate::{
    ChangeVerb, Configuration, CongenChange, Description, FieldDescription, NotSupported,
    OptionChange, ParseError, VerbError,
};

impl<T> Configuration for Option<T>
where
    T: Configuration + 'static,
    OptionChange<T::CongenChange>: CongenChange,
{
    type CongenChange = OptionChange<T::CongenChange>;

    fn apply_change_with_default(
        &mut self,
        change: Self::CongenChange,
        default: Option<fn() -> Self>,
    ) {
        let mut change = match change {
            OptionChange::Apply(change) => change,
            OptionChange::Unset => {
                *self = None;
                return;
            }
            OptionChange::NoChange => return,
        };
        if self.is_none() {
            change = match change.unwrap_field() {
                Ok(unwrapped) => {
                    *self = Some(downcast(unwrapped));
                    return;
                }
                Err(change) => change,
            };

            let Some(Some(default)) = default.map(|d| d()) else {
                eprintln!(
                    "type: Option<{}>\nchange: {change:?}\ndefault: {:?}",
                    std::any::type_name::<T>(),
                    default.map(|d| d())
                );
                panic!("called apply_change_with_default on None without a default constructor");
            };
            *self = Some(default);
        }
        let this = self
            .as_mut()
            .expect("initialized from default constructor if missing");

        this.apply_change(change);
    }

    fn description(field_name: &'static str) -> Description {
        match T::description(field_name) {
            Description::Composit(composit) => Description::Composit(composit.as_option()),
            Description::Field(field) => Description::Field(field.as_option()),
        }
    }

    fn default() -> Result<Self, NotSupported> {
        Ok(None)
    }

    // TODO move fn into CongenChange?
    fn parse(input: &str) -> Result<Result<Self::CongenChange, ParseError>, NotSupported> {
        // NOTE: we parse Option<T> just like T and assume Some. There is no way to parse
        // None.
        match T::parse(input) {
            Ok(Ok(inner)) => Ok(Ok(OptionChange::Apply(inner))),
            Ok(Err(parse_err)) => Ok(Err(parse_err)),
            Err(_) => Err(NotSupported),
        }
    }

    fn type_name() -> std::borrow::Cow<'static, str> {
        let mut name = T::type_name();
        name.to_mut().push('?');
        name
    }
}

impl<C> CongenChange for OptionChange<C>
where
    C: CongenChange + 'static,
    C::Configuration: 'static,
{
    type Configuration = Option<C::Configuration>;

    fn empty() -> Self {
        OptionChange::NoChange
    }

    fn apply_change(&mut self, change: Self) {
        match change {
            OptionChange::Apply(change) => {
                *self = OptionChange::Apply(change);
            }
            OptionChange::Unset => {
                *self = OptionChange::Unset;
            }
            OptionChange::NoChange => (),
        }
    }

    #[expect(unused_variables)]
    fn from_path_and_verb<'a, P>(mut path: P, verb: ChangeVerb) -> Result<Self, VerbError>
    where
        P: Iterator<Item = &'a str>,
    {
        match Self::Configuration::description("") {
            Description::Composit(desc) => match &verb {
                ChangeVerb::Set(_) | ChangeVerb::SetAny(_) => {
                    Ok(OptionChange::Apply(C::from_path_and_verb(path, verb)?))
                }
                ChangeVerb::SetFlag => {
                    let mut path = path.peekable();
                    if path.peek().is_none() {
                        return Err(VerbError::UnsupportedVerb(verb));
                    }
                    Ok(OptionChange::Apply(C::from_path_and_verb(path, verb)?))
                }
                ChangeVerb::Unset => {
                    let mut path = path.peekable();
                    if path.peek().is_none() {
                        Ok(OptionChange::Unset)
                    } else {
                        Ok(OptionChange::Apply(C::from_path_and_verb(path, verb)?))
                    }
                }
                ChangeVerb::UseDefault => {
                    let mut path = path.peekable();
                    if path.peek().is_none() {
                        let inner_default = C::from_path_and_verb(empty(), verb)?;
                        Ok(OptionChange::Apply(inner_default))
                    } else {
                        Ok(OptionChange::Apply(C::from_path_and_verb(path, verb)?))
                    }
                }
            },
            Description::Field(desc) => {
                if path.next().is_some() {
                    return Err(VerbError::InvalidPath);
                }

                match verb {
                    ChangeVerb::Set(value) => {
                        let change = Self::Configuration::parse(&value)??;
                        Ok(downcast(change))
                    }
                    ChangeVerb::SetAny(value) => {
                        let change = value.downcast().map_err(|_| VerbError::DowncastFailed)?;
                        Ok(OptionChange::Apply(*change))
                    }
                    ChangeVerb::SetFlag => Err(VerbError::UnsupportedVerb(verb)),
                    ChangeVerb::Unset => Ok(OptionChange::Unset),
                    ChangeVerb::UseDefault => Ok(OptionChange::Unset),
                }
            }
        }
    }

    fn unwrap_field(self) -> Result<Self::Configuration, Self> {
        Err(self)
    }
}

/// A helper to safely downcast types
///
/// this will panic if the downcast is not safe.
/// Used to handle generics, where we know `T` and `F` should be of the same type,
/// but can't express this in the type system, e.g. circular type between `Configuration` and
/// `CongenChange`:
/// `<C as Configuration>::CongenChange::Configuration == C`
fn downcast<F: 'static, T: 'static>(value: F) -> T {
    let mut maybe = MaybeUninit::new(value);

    unsafe {
        // Safety: created through MaybeUninit::new
        let value: &mut F = maybe.assume_init_mut();
        let value: &mut dyn Any = value;
        let value: &mut T = value.downcast_mut().expect(&format!(
            "called downcast on incompatible types: {} => {}",
            core::any::type_name::<F>(),
            core::any::type_name::<T>()
        ));

        // Safety: value is properly initialized as it is a reference to "maybe"
        //      this is a valid "move" because "maybe" is of type MaybeUninit and never accessed
        //      again, not even dropped.
        core::ptr::read(value)
    }
}

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

    fn parse(input: &str) -> Result<Result<Self::CongenChange, crate::ParseError>, NotSupported> {
        match bool::from_str(input) {
            Ok(value) => Ok(Ok(Some(value))),
            Err(e) => Ok(Err(ParseError(e.to_string()))),
        }
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
            ChangeVerb::Set(unparesd) => Ok(Self::Configuration::parse(&unparesd)??),
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

    fn parse(input: &str) -> Result<Result<Self::CongenChange, crate::ParseError>, NotSupported> {
        Ok(Ok(Some(input.to_owned())))
    }
}

impl CongenChange for Option<String> {
    type Configuration = String;

    fn empty() -> Self {
        None
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
            ChangeVerb::Set(unparesd) => Ok(Self::Configuration::parse(&unparesd)??),
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

    fn parse(input: &str) -> Result<Result<Self::CongenChange, crate::ParseError>, NotSupported> {
        match u32::from_str(input) {
            Ok(value) => Ok(Ok(Some(value))),
            Err(e) => Ok(Err(ParseError(e.to_string()))),
        }
    }
}

impl CongenChange for Option<u32> {
    type Configuration = u32;

    fn empty() -> Self {
        None
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
            ChangeVerb::Set(unparesd) => Ok(Self::Configuration::parse(&unparesd)??),
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
