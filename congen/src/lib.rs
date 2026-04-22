mod clap_bridge;
mod impls;

pub use congen_derive::Configuration;

use std::{any::Any, borrow::Cow};

pub use clap_bridge::CongenClap;

/// Denotes that the operation is not supported by a [Configuration]
#[derive(Debug)]
pub struct NotSupported;

#[derive(Debug)]
pub struct ParseError;

// TODO split into public and internal trait, only apply_change should be called by consumers of
// this lib while the rest of the functions are used internally to implement the interface
pub trait Configuration: Sized {
    type CongenChange: crate::CongenChange;

    /// apply change to `self`
    fn apply_change(&mut self, change: Self::CongenChange);

    fn description(field_name: &'static str) -> Description;

    /// Returns `Ok(default_value)` if this type has a default
    fn default() -> Result<Self, NotSupported> {
        Err(NotSupported)
    }

    /// If [Self::CongenChange] supports an `unwrap` operation this should
    /// return `Ok(change.unwrap())`. Otherwise `Err(NotSupported)`
    fn unwrap_change(_change: Self::CongenChange) -> Result<Self, NotSupported> {
        Err(NotSupported)
    }

    /// Parses a simple field value.
    ///
    /// Returns `Err(NotSupported)` for complex structs that can't be directly parsed from a
    /// string.
    /// Otherwise it should either return `Ok(Ok(parsed_value))` or `Ok(Err(parse_error))`
    fn parse(_input: &str) -> Result<Result<Self, ParseError>, NotSupported> {
        Err(NotSupported)
    }

    // TODO verify function that can check that parsed values are within bounds, etc.
    //  Bounds check should be implemented on the composite. Fields can check during parse.
    //  However a composite might have additional requirements, e.g u32 must be greater than 10

    fn type_name() -> Cow<'static, str>;
}

pub trait CongenChange: Sized {
    /// An empty change.
    ///
    /// applying the result of this function should have no effect
    fn empty() -> Self;

    /// combine 2 changes.
    ///
    /// # Implementation
    ///
    /// given a `configuration: Self::Configuration` applying the result of
    /// `change_a.apply_change(change_b)` should be the same as applying `change_a` and
    /// then `change_b`, e.g.
    /// ```ignore
    /// let combined = change_a.apply_change(change_b);
    /// configuration.apply_change(combined)
    /// ```
    /// should be the same as
    /// ```ignore
    /// configuration.apply_change(change_a);
    /// configuration.apply_change(change_b);
    /// ```
    fn apply_change(&mut self, change: Self);

    fn from_path_and_verb<'a, P>(path: P, verb: ChangeVerb) -> Result<Self, FromVerbError>
    where
        P: Iterator<Item = &'a str>;
}

// TODO better name
// TODO implement Error (thiserror)
#[derive(Debug)]
pub enum FromVerbError {
    InvalidPath,
    UnsuportedVerb(ChangeVerb),
    NotSupported(NotSupported),
    ParseError(ParseError),
    DowncastFailed,
}

impl From<NotSupported> for FromVerbError {
    fn from(value: NotSupported) -> Self {
        FromVerbError::NotSupported(value)
    }
}

impl From<ParseError> for FromVerbError {
    fn from(value: ParseError) -> Self {
        FromVerbError::ParseError(value)
    }
}

#[derive(Debug)]
pub enum ChangeVerb {
    Set(String),
    SetFlag,
    Unset,
    UseDefault,
    SetAny(Box<dyn Any + 'static>),
}

#[derive(Debug)]
pub enum Description {
    Composit(CompositDescription),
    Field(FieldDescription),
}

impl Description {
    pub fn as_option(self) -> Self {
        match self {
            Description::Composit(composit) => Self::Composit(composit.as_option()),
            Description::Field(field) => Self::Field(field.as_option()),
        }
    }

    pub fn with_default(self) -> Self {
        match self {
            Description::Composit(composit) => Self::Composit(composit.with_default()),
            Description::Field(field) => Self::Field(field.with_default()),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Description::Field(f) => f.field_name,
            Description::Composit(c) => c.field_name,
        }
    }

    pub fn has_default(&self) -> bool {
        match self {
            Description::Field(f) => f.has_default,
            Description::Composit(c) => c.has_default,
        }
    }

    pub fn is_path_valid<'a, 's, P>(&'s self, mut path: P) -> bool
    where
        P: Iterator<Item = &'a str>,
    {
        match self {
            Description::Composit(composit_description) => {
                let Some(next_field_name) = path.next() else {
                    return false;
                };

                let Some(next_field) = composit_description.field(next_field_name) else {
                    return false;
                };

                next_field.is_path_valid(path)
            }
            Description::Field(_field_description) => path.next().is_none(),
        }
    }
}

impl From<CompositDescription> for Description {
    fn from(value: CompositDescription) -> Self {
        Description::Composit(value)
    }
}
impl From<FieldDescription> for Description {
    fn from(value: FieldDescription) -> Self {
        Description::Field(value)
    }
}

#[derive(Debug)]
pub struct CompositDescription {
    pub field_name: &'static str,
    pub type_name: Cow<'static, str>,
    pub fields: Vec<Description>,
    pub has_default: bool,
    pub allow_unset: bool,
}

impl CompositDescription {
    pub fn as_option(self) -> Self {
        Self {
            allow_unset: true,
            ..self
        }
    }

    pub fn with_default(self) -> Self {
        Self {
            has_default: true,
            ..self
        }
    }

    pub fn field<'d, 'n>(&'d self, name: &'n str) -> Option<&'d Description> {
        self.fields.iter().find(|f| f.name() == name)
    }

    pub fn fields(&self) -> impl Iterator<Item = (String, FieldDescription)> {
        self.fields.iter().flat_map(move |child| match child {
            Description::Field(field) => {
                let mut full_name = String::new();
                full_name.push_str(field.field_name);
                std::iter::once((full_name, field.clone())).collect::<Vec<_>>()
            }
            Description::Composit(composit) => composit
                .fields()
                .map(|(name, field)| {
                    let mut full_name = String::new();
                    full_name.push_str(composit.field_name);
                    full_name.push_str(".");
                    full_name.push_str(&name);
                    (full_name, field)
                })
                .collect::<Vec<_>>(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct FieldDescription {
    pub field_name: &'static str,
    pub type_name: Cow<'static, str>,
    pub is_flag: bool,
    pub allow_unset: bool,
    pub has_default: bool,
    pub cmd_value_hint: clap::ValueHint,
}

impl FieldDescription {
    pub fn as_option(self) -> Self {
        Self {
            is_flag: false,
            allow_unset: true,
            ..self
        }
    }

    pub fn with_default(self) -> Self {
        Self {
            has_default: true,
            ..self
        }
    }
}

#[derive(Default)]
pub enum OptionChange<T> {
    Apply(T),
    #[default]
    NoChange,
}

impl<T> OptionChange<T> {
    pub fn unwrap(self) -> T {
        match self {
            OptionChange::Apply(c) => c,
            OptionChange::NoChange => panic!("OptionChange is NoChange but unwrap was called!"),
        }
    }
}
