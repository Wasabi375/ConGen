pub mod clap_bridge;
pub mod lists;
pub mod option;
pub mod primitives;

pub use congen_derive::{Configuration, ValueEnumConfiguration};

use std::{
    any::{Any, TypeId},
    borrow::Cow,
    collections::VecDeque,
    mem::MaybeUninit,
};

use thiserror::Error;

pub use clap_bridge::CongenClap;

/// Denotes that the operation is not supported by a [Configuration]
#[derive(Debug, Error)]
#[error(
    "operation not supported by this configuration type. Incorrect implementation of the Configuration trait"
)]
pub struct NotSupported;

#[derive(Debug, Error)]
#[error("failed to parse value: {0}")]
pub struct ParseError(pub String);

// TODO split into public and internal trait, only apply_change should be called by consumers of
// this lib while the rest of the functions are used internally to implement the interface
pub trait Configuration: Sized + core::fmt::Debug {
    /// The [CongenChange] type associated with this [Configuration]
    type CongenChange: crate::CongenChange;

    /// apply change to `self`
    fn apply_change(&mut self, change: Self::CongenChange) {
        Self::apply_change_with_default(self, change, None);
    }

    /// apply change to `self`
    ///
    /// providing a `default` value in case `Self = Option<Configuration>` and
    /// `self == None`. In this case if `default` is provided it must return `Some(default)`.
    // TODO internal
    fn apply_change_with_default(
        &mut self,
        change: Self::CongenChange,
        default: Option<fn() -> Self>,
    );

    /// Get a description of this [Configuration]
    fn description(field_name: &'static str) -> Description;

    /// Returns `Ok(default_value)` if this type has a default
    // TODO internal
    fn default() -> Result<Self, NotSupported> {
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

pub trait CongenChange: Sized + core::fmt::Debug {
    type Configuration: crate::Configuration;

    /// An empty change.
    ///
    /// applying the result of this function should have no effect
    fn empty() -> Self;

    fn default() -> Result<Self, NotSupported> {
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

    fn unwrap_field(self) -> Result<Self::Configuration, Self> {
        Err(self)
    }

    /// combine 2 changes.
    ///
    /// # Implementation
    ///
    /// given a `configuration: Self::Configuration` applying the result of
    /// `change_a.apply_change(change_b)` should be the same as applying `change_a` and
    /// then `change_b`, e.g.
    /// ```compile_fail
    /// let combined = change_a.apply_change(change_b);
    /// configuration.apply_change(combined)
    /// ```
    /// should be the same as
    /// ```compile_fail
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
#[non_exhaustive]
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
    #[error("the description is in an invalid state")]
    InvalidDescription,
}

/// A [ChangeVerb] is used to create a [CongenChange] based on a path.
///
/// See [CongenChange::from_path_and_verb].
// TODO internal
#[derive(Debug)]
pub enum ChangeVerb {
    /// Set the change to [CongenChange::parse] of the value
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
#[derive(Debug, Clone)]
pub enum Description {
    /// A composite type, e.g. structs or enums with values
    Composit(CompositDescription),
    /// A simple parsable field, e.g. String, int, anything parsable from the command line
    Field(FieldDescription),
    // TODO document
    List(ListDescription),
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
            Description::List(list) => Self::List(list.with_default()),
        }
    }

    /// The name of the Field/Struct/Enum
    pub fn name(&self) -> &'static str {
        match self {
            Description::Field(f) => f.field_name,
            Description::Composit(c) => c.field_name,
            Description::List(l) => l.field_name,
        }
    }

    /// Whether or not the [Configuration] provides a default value
    pub fn has_default(&self) -> bool {
        match self {
            Description::Field(f) => f.has_default,
            Description::Composit(c) => c.has_default,
            Description::List(l) => l.has_default,
        }
    }

    /// Whether or not the [Configuration] can be unset
    pub fn allow_unset(&self) -> bool {
        match self {
            Description::Field(f) => f.allow_unset,
            Description::Composit(c) => c.allow_unset,
            Description::List(_) => false,
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
            Description::Field(_) | Description::List(_) => path.next().is_none(),
        }
    }

    pub fn actionable_fields(&self) -> Vec<ActionableField> {
        let composite = match self {
            Description::Field(_field) => {
                return vec![ActionableField {
                    description: self.clone(),
                    path: VecDeque::new(),
                }];
            }
            Description::Composit(comp) => comp,
            Description::List(_list) => todo!(),
        };

        let mut actionable = Vec::new();

        for field in composite.fields.iter() {
            match field {
                Description::Field(field) => {
                    actionable.push(ActionableField {
                        description: field.clone().into(),
                        path: VecDeque::from([field.field_name]),
                    });
                }
                Description::Composit(composite) => {
                    let fields = field.actionable_fields().into_iter().map(|mut field| {
                        field.path.push_front(composite.field_name);
                        field
                    });
                    actionable.extend(fields);

                    if composite.is_actionable() {
                        actionable.push(ActionableField {
                            description: field.clone(),
                            path: VecDeque::from([composite.field_name]),
                        });
                    }
                }
                Description::List(_list) => todo!(),
            }
        }

        actionable
    }
}

#[derive(Debug)]
pub struct ActionableField {
    description: Description,
    path: VecDeque<&'static str>,
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
impl From<ListDescription> for Description {
    fn from(value: ListDescription) -> Self {
        Description::List(value)
    }
}

/// A description for a composite type, e.g. structs or enums with values
#[derive(Debug, Clone)]
pub struct CompositDescription {
    pub field_name: &'static str,
    pub type_name: Cow<'static, str>,
    pub fields: Vec<Description>,
    pub has_default: bool,
    pub allow_unset: bool,
}

impl CompositDescription {
    #[allow(missing_docs)]
    pub fn with_default(self) -> Self {
        Self {
            has_default: true,
            ..self
        }
    }

    /// returns `true` if the description describes a field that is actionable
    pub fn is_actionable(&self) -> bool {
        self.has_default || self.allow_unset
    }

    /// returns a reference to the fields [Description] if the field exists
    pub fn field<'d>(&'d self, name: &str) -> Option<&'d Description> {
        self.fields.iter().find(|f| f.name() == name)
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
    pub fn with_default(self) -> Self {
        Self {
            has_default: true,
            ..self
        }
    }
}

#[derive(Debug, Clone)]
pub struct ListDescription {
    pub field_name: &'static str,
    pub type_name: Cow<'static, str>,
    pub inner_desc: Box<Description>,
    pub has_default: bool,
}

impl ListDescription {
    pub fn with_default(self) -> Self {
        Self {
            has_default: true,
            ..self
        }
    }
}

/// A helper to safely cast between 2 identical generic types
///
/// this will panic if `T` and `F` are not of the same type.
/// Used to handle generics, where we know `T` and `F` should be of the same type,
/// but can't express this in the type system, e.g. circular type between `Configuration` and
/// `CongenChange`:
/// `<C as Configuration>::CongenChange::Configuration == C`
pub(crate) fn self_cast<F: 'static, T: 'static>(value: F) -> T {
    assert_eq!(TypeId::of::<F>(), TypeId::of::<T>());

    let mut maybe = MaybeUninit::new(value);

    unsafe {
        // Safety: created through MaybeUninit::new
        let value: &mut F = maybe.assume_init_mut();
        let value: &mut dyn Any = value;
        let value: &mut T = value.downcast_mut().unwrap_or_else(|| {
            panic!(
                "called downcast on incompatible types: {} => {}",
                core::any::type_name::<F>(),
                core::any::type_name::<T>()
            )
        });

        // Safety: value is properly initialized as it is a reference to "maybe"
        //      this is a valid "move" because "maybe" is of type MaybeUninit and never accessed
        //      again, not even dropped.
        core::ptr::read(value)
    }
}
