mod impls;
pub use congen_derive::Configuration;

use std::borrow::Cow;

use clap::{Arg, Command};

/// Denotes that the operation is not supported by a [Configuration]
pub struct NotSupported;

// TODO split into public and internal trait, only apply_change should be called by consumers of
// this lib while the rest of the functions are used internally to implement the interface
pub trait Configuration: Sized {
    type CongenChange;

    fn apply_change(&mut self, change: Self::CongenChange);

    fn description(field_name: Option<&'static str>) -> Description;

    /// Returns `Ok(default_value)` if this type has a default
    fn default() -> Result<Self, NotSupported> {
        Err(NotSupported)
    }

    /// If [Self::CongenChange] supports an `unwrap` operation this should
    /// return `Ok(change.unwrap())`. Otherwise `Err(NotSupported)`
    fn unwrap_change(_change: Self::CongenChange) -> Result<Self, NotSupported> {
        Err(NotSupported)
    }

    fn type_name() -> Cow<'static, str>;
}

/// A value type used for configurations
// TODO implies something something Parse in clap from string
// e.g. primitive, string, value enum, etc
pub trait ConfigurationValue: Configuration {}

impl<T> ConfigurationOptionSafe for T where T: ConfigurationValue {}

/// Implies that [Configuration::default] or [Configuration::unwrap_change]
/// are supported
// TODO internal
pub trait ConfigurationOptionSafe: Configuration {}

// TODO internal
pub fn check_safe_in_option<T: ConfigurationOptionSafe>() -> bool {
    true
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
    pub field_name: Option<&'static str>,
    pub type_name: Cow<'static, str>,
    // TODO combine fields and composites?
    pub fields: Vec<FieldDescription>,
    pub composites: Vec<CompositDescription>,
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

    pub fn fields(&self) -> impl Iterator<Item = (String, FieldDescription)> {
        let own_fields = self.fields.iter().cloned().map(|field| {
            let mut full_name = String::new();
            if let Some(field_name) = self.field_name {
                full_name.push_str(field_name);
                full_name.push('.');
            }
            full_name.push_str(field.field_name);
            (full_name, field)
        });

        let child_fields =
            self.composites
                .iter()
                .flat_map(|child| child.fields())
                .map(|(mut name, field)| {
                    if let Some(field_name) = self.field_name {
                        name.insert(0, '.');
                        name.insert_str(0, field_name);
                    }
                    (name, field)
                });
        let child_fields: Box<dyn Iterator<Item = (String, FieldDescription)>> =
            Box::new(child_fields);

        own_fields.chain(child_fields)
    }

    pub fn extend_set_command(&self, cmd: Command) -> Command {
        cmd.subcommands(self.fields().map(|(full_name, field)| {
            let mut arg = Arg::new(field.field_name);
            arg = if field.is_flag {
                arg
            } else {
                arg.value_name(field.type_name.to_uppercase())
            };

            Command::new(full_name).arg(arg)
        }))
    }

    pub fn extend_unset_command(&self, cmd: Command) -> Command {
        cmd.subcommands(
            self.fields()
                .filter(|(_, field)| field.allow_unset)
                .map(|(full_name, _field)| Command::new(full_name)),
        )
    }

    pub fn extend_use_default_command(&self, cmd: Command) -> Command {
        cmd.subcommands(
            self.fields()
                .filter(|(_, field)| field.has_default)
                .map(|(full_name, _field)| Command::new(full_name)),
        )
    }
}

#[derive(Debug, Clone)]
pub struct FieldDescription {
    field_name: &'static str,
    type_name: Cow<'static, str>,
    is_flag: bool,
    allow_unset: bool,
    has_default: bool,
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
