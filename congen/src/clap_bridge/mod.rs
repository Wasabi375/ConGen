mod value_enum;

use clap::{Arg, Args, Command, FromArgMatches};

use crate::{ChangeVerb, Configuration, CongenChange, Description};

pub use value_enum::ValueEnumConfiguration;

/// provides a [clap::Args] implementation for a [Configuration] for clap-derive.
///
/// Clap will parse a [CongenChange] for the [Configuration] which can
/// then be applied to an existing config.
pub struct CongenClap<T: Configuration> {
    change: T::CongenChange,
}

impl<T: Configuration> CongenClap<T> {
    /// Provides the [CongenChange]
    pub fn into_change(self) -> T::CongenChange {
        self.change
    }

    /// Create a [clap::Command] that will result in a [CongenClap]
    ///
    /// This can be used with [clap]s non-derive based builders.
    pub fn create_cmd(cmd_name: impl Into<clap::builder::Str>) -> clap::Command {
        let cmd = clap::Command::new(cmd_name);
        Self::augment_args_internal(cmd, false)
    }

    fn augment_args_internal(cmd: Command, for_update: bool) -> clap::Command {
        cmd.subcommands(
            T::description("__clap")
                .actionable_fields()
                .into_iter()
                .map(|mut actionable| {
                    let field_name = actionable.path.make_contiguous().join(".");
                    let mut field_command =
                        Command::new(field_name).subcommand_required(!for_update);

                    match &actionable.description {
                        Description::Field(field) => {
                            let mut set = Command::new("set");
                            if !field.is_flag {
                                set = set.arg(
                                    Arg::new("value")
                                        .value_name(field.type_name.to_uppercase())
                                        .required(!for_update),
                                );
                            }
                            field_command = field_command.subcommand(set);
                        }
                        Description::List(list) => {
                            let mut key_arg =
                                Arg::new("key").value_name("AT").required(!for_update);
                            if list.key_is_int {
                                key_arg = key_arg.value_parser(clap::value_parser!(usize));
                            }

                            // append, update, remove, empty
                            field_command = field_command
                                .subcommand(Command::new("empty"))
                                .subcommand(Command::new("remove").arg(key_arg.clone()));

                            match list.inner_desc.as_ref() {
                                Description::Field(inner_field) => {
                                    let value_arg = Arg::new("value")
                                        .value_name(inner_field.type_name.to_uppercase())
                                        .required(!for_update);
                                    field_command = field_command
                                        .subcommand(Command::new("append").arg(value_arg.clone()))
                                        .subcommand(
                                            Command::new("update").arg(key_arg).arg(value_arg),
                                        );
                                }
                                Description::Composit(_inner_comp) => todo!(),
                                Description::List(_inner_list) => todo!(),
                            }
                        }
                        _ => (),
                    }

                    if actionable.description.has_default() {
                        field_command = field_command.subcommand(Command::new("use-default"));
                    }
                    if actionable.description.allow_unset() {
                        field_command = field_command.subcommand(Command::new("unset"));
                    }

                    field_command
                }),
        )
        .subcommand_required(true)
    }
}

impl<T: Configuration> Args for CongenClap<T> {
    fn augment_args(cmd: clap::Command) -> clap::Command {
        Self::augment_args_internal(cmd, false)
    }

    fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
        Self::augment_args_internal(cmd, true)
    }
}

impl<T: Configuration> FromArgMatches for CongenClap<T>
where
    T::CongenChange: Sized,
{
    fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
        let mut res = Self {
            change: CongenChange::empty(),
        };
        res.update_from_arg_matches(matches)?;
        Ok(res)
    }

    fn update_from_arg_matches(&mut self, matches: &clap::ArgMatches) -> Result<(), clap::Error> {
        let Some((field_name, field_cmd)) = matches.subcommand() else {
            return Err(clap::Error::new(clap::error::ErrorKind::MissingSubcommand));
        };

        let field_path = field_name.split(".");

        let Some(_field_desc) = T::description("").actionable_field(field_path.clone()) else {
            return Err(clap::Error::raw(
                clap::error::ErrorKind::InvalidValue,
                "invalid path",
            ));
        };

        let Some((verb_name, verb_cmd)) = field_cmd.subcommand() else {
            return Err(clap::Error::new(clap::error::ErrorKind::MissingSubcommand));
        };

        let verb = match verb_name {
            "set" => match verb_cmd.try_get_one("value").map(|value| value.cloned()) {
                Ok(Some(value)) => ChangeVerb::Set(value),
                Err(clap::parser::MatchesError::UnknownArgument { .. }) => ChangeVerb::SetFlag,
                Ok(None) => return Err(clap::Error::new(clap::error::ErrorKind::TooFewValues)),
                Err(_err) => return Err(clap::Error::new(clap::error::ErrorKind::InvalidValue)),
            },
            "unset" => ChangeVerb::Unset,
            "use-default" => ChangeVerb::UseDefault,
            _ => return Err(clap::Error::new(clap::error::ErrorKind::InvalidSubcommand)),
        };

        dbg!(&verb);
        dbg!(field_path.clone().collect::<Vec<_>>());

        let change = T::CongenChange::from_path_and_verb(field_path, verb)
            .expect("Failed to create change for path");

        self.change.apply_change(change);
        Ok(())
    }
}
