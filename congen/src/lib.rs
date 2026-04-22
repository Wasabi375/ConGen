mod clap_bridge;
mod impls;

pub use congen_derive::Configuration;

use std::{any::Any, borrow::Cow};

use thiserror::Error;

pub use clap_bridge::CongenClap;

/// Denotes that the operation is not supported by a [Configuration]
#[derive(Debug, Error)]
#[error("operation not supported by this configuration type")]
pub struct NotSupported;

#[derive(Debug, Error)]
#[error("failed to parse value: {0}")]
pub struct ParseError(pub String);

// TODO split into public and internal trait, only apply_change should be called by consumers of
// this lib while the rest of the functions are used internally to implement the interface
pub trait Configuration: Sized {
    /// The [CongenChange] type associated with this [Configuration]
    type CongenChange: crate::CongenChange;

    /// apply change to `self`
    fn apply_change(&mut self, change: Self::CongenChange);

    /// Get a description of this [Configuration]
    fn description(field_name: &'static str) -> Description;

    /// Returns `Ok(default_value)` if this type has a default
    // TODO internal
    fn default() -> Result<Self, NotSupported> {
        Err(NotSupported)
    }

    /// Parses a simple field value.
    ///
    /// Returns `Err(NotSupported)` for complex structs that can't be directly parsed from a
    /// string.
    /// Otherwise it should either return `Ok(Ok(parsed_value))` or `Ok(Err(parse_error))`
    // TODO internal
    fn parse(_input: &str) -> Result<Result<Self, ParseError>, NotSupported> {
        Err(NotSupported)
    }

    // TODO verify function that can check that parsed values are within bounds, etc.
    //  Bounds check should be implemented on the composite. Fields can check during parse.
    //  However a composite might have additional requirements, e.g u32 must be greater than 10

    /// The typename of this [Configuration].
    ///
    /// This is used in a user-facing system and does not need to exactly match the rust type.
    // TODO internal
    fn type_name() -> Cow<'static, str> {
        std::any::type_name::<Self>().into()
    }
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

    /// Create a change that fills the `path` based on the verb
    ///
    /// # Example
    ///
    /// A path `[a, b, c]` and a verb `ChangeVerb::UseDefault` means that the result
    /// should be something like `empty().a.b.c = default()`.
    ///
    /// # Implementation
    ///
    /// an empty path always refers to `Self` while any path entry refers to a subfield
    /// relative to `self`.
    /// The result should be initialized to `Self::empty` with just the final field in the path
    /// changed based on the verb.
    fn from_path_and_verb<'a, P>(path: P, verb: ChangeVerb) -> Result<Self, VerbError>
    where
        P: Iterator<Item = &'a str>;
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum VerbError {
    #[error("invalid path: the specified configuration path does not exist")]
    InvalidPath,
    #[error("unsupported verb '{0:?}' for this configuration field")]
    UnsupportedVerb(ChangeVerb),
    #[error(transparent)]
    NotSupported(#[from] NotSupported),
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error("failed to downcast value to the expected type")]
    DowncastFailed,
}

/// A [ChangeVerb] is used to create a [CongenChange] based on a path.
///
/// See [CongenChange::from_path_and_verb].
// TODO internal
#[derive(Debug)]
pub enum ChangeVerb {
    /// Set the change to [Configuration::parse] of the value
    Set(String),
    /// Enable the flag in the change
    SetFlag,
    /// Unset the change
    Unset,
    /// Use the default value for the change.
    UseDefault,
    /// Similar to [ChangeVerb::Set] but already contains the parsed value
    ///
    /// The value must be of the same type as defined by the [Configuration] path.
    SetAny(Box<dyn Any + 'static>),
}

/// Descibes a [Configuration]
// TODO internal
#[derive(Debug)]
pub enum Description {
    /// A composite type, e.g. structs or enums with values
    Composit(CompositDescription),
    /// A simple parsable field, e.g. String, int, anything parsable from the command line
    Field(FieldDescription),
}

impl Description {
    /// creates a [Description] that claims to have a default value.
    ///
    /// When calling this the caller must ensure that the coresponding [Configuration]
    /// can provide a default value.
    pub fn with_default(self) -> Self {
        match self {
            Description::Composit(composit) => Self::Composit(composit.with_default()),
            Description::Field(field) => Self::Field(field.with_default()),
        }
    }

    /// The name of the Field/Struct/Enum
    pub fn name(&self) -> &'static str {
        match self {
            Description::Field(f) => f.field_name,
            Description::Composit(c) => c.field_name,
        }
    }

    /// Whether or not the [Configuration] provides a default value
    pub fn has_default(&self) -> bool {
        match self {
            Description::Field(f) => f.has_default,
            Description::Composit(c) => c.has_default,
        }
    }

    /// Returns true if a path is valid for the [Configuration]
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

/// A description for a composite type, e.g. structs or enums with values
#[derive(Debug)]
pub struct CompositDescription {
    pub field_name: &'static str,
    pub type_name: Cow<'static, str>,
    pub fields: Vec<Description>,
    pub has_default: bool,
    pub allow_unset: bool,
}

impl CompositDescription {
    #[allow(missing_docs)]
    pub fn as_option(self) -> Self {
        Self {
            allow_unset: true,
            ..self
        }
    }

    #[allow(missing_docs)]
    pub fn with_default(self) -> Self {
        Self {
            has_default: true,
            ..self
        }
    }

    /// returns a reference to the fields [Description] if the field exists
    pub fn field<'d, 'n>(&'d self, name: &'n str) -> Option<&'d Description> {
        self.fields.iter().find(|f| f.name() == name)
    }

    /// Return an iterator over all terminal [FieldDescription] with their path.
    // TODO I probably want something better that also gives me access to all
    // [CompositeDescription]s  that allow for unset or use-default
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

/// A description for a simple parsable field, e.g. String, int, anything parsable from the command line
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
    #[allow(missing_docs)]
    pub fn as_option(self) -> Self {
        Self {
            is_flag: false,
            allow_unset: true,
            ..self
        }
    }

    #[allow(missing_docs)]
    pub fn with_default(self) -> Self {
        Self {
            has_default: true,
            ..self
        }
    }
}

/// [CongenChange] for [Option]
// TODO internal
#[derive(Default)]
pub enum OptionChange<T> {
    Apply(T),
    #[default]
    NoChange,
}

impl<T> OptionChange<T> {
    /// Same as [Option::unwrap]
    pub fn unwrap(self) -> T {
        match self {
            OptionChange::Apply(c) => c,
            OptionChange::NoChange => panic!("OptionChange is NoChange but unwrap was called!"),
        }
    }
}
