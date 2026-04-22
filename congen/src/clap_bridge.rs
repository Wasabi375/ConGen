use clap::{Arg, Args, Command, FromArgMatches};

use crate::{ChangeVerb, Configuration, CongenChange, Description};

pub struct CongenClap<T: Configuration> {
    change: T::CongenChange,
}

impl<T: Configuration> CongenClap<T> {
    pub fn into_change(self) -> T::CongenChange {
        self.change
    }

    fn augment_args_internal(cmd: Command, for_update: bool) -> clap::Command {
        let Description::Composit(description) = T::description("__clap") else {
            todo!("CongenClap does not yet support FieldDescriptions");
        };

        cmd.subcommands(description.fields().map(|(field_name, field_desc)| {
            let mut field_command = Command::new(field_name).subcommand_required(!for_update);

            {
                let mut set = Command::new("set");
                if !field_desc.is_flag {
                    set = set.arg(
                        Arg::new("value")
                            .value_name(field_desc.type_name.to_uppercase())
                            .required(!for_update),
                    );
                }
                field_command = field_command.subcommand(set);
            }
            if field_desc.has_default {
                field_command = field_command.subcommand(Command::new("use-default"));
            }
            if field_desc.allow_unset {
                field_command = field_command.subcommand(Command::new("unset"));
            }

            field_command
        }))
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

        T::description("").is_path_valid(field_path.clone());

        let Some((verb_name, verb_cmd)) = field_cmd.subcommand() else {
            return Err(clap::Error::new(clap::error::ErrorKind::MissingSubcommand));
        };

        let verb = match verb_name {
            "set" => {
                let value: Option<String> = verb_cmd.get_one("value").cloned();
                if let Some(value) = value {
                    ChangeVerb::Set(value)
                } else {
                    ChangeVerb::SetFlag
                }
            }
            "unset" => ChangeVerb::Unset,
            "use-default" => ChangeVerb::UseDefault,
            _ => return Err(clap::Error::new(clap::error::ErrorKind::InvalidSubcommand)),
        };

        let change = T::CongenChange::from_path_and_verb(field_path, verb)
            .expect("Failed to create change for path");

        self.change.apply_change(change);
        Ok(())
    }
}
