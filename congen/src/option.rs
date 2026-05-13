use core::iter::empty;

use crate::{
    ChangeVerb, CompositDescription, Configuration, CongenChange, Description, FieldDescription,
    NotSupported, ParseError, VerbError, self_cast,
};

/// [CongenChange] for [Option]
#[derive(Default, Debug)]
pub enum OptionChange<T> {
    Apply(T),
    Unset,
    #[default]
    NoChange,
}

impl<T> OptionChange<T> {
    /// Same as [Option::unwrap]
    pub fn unwrap(self) -> T {
        match self {
            OptionChange::Apply(c) => c,
            OptionChange::Unset => panic!("OptionChange is Unset but unwrap was called!"),
            OptionChange::NoChange => panic!("OptionChange is NoChange but unwrap was called!"),
        }
    }
}

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
                    *self = Some(self_cast(unwrapped));
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
            Description::Composit(composit) => Description::Composit(CompositDescription {
                allow_unset: true,
                type_name: Self::type_name(),
                ..composit
            }),
            Description::Field(field) => Description::Field(FieldDescription {
                is_flag: false,
                allow_unset: true,
                type_name: Self::type_name(),
                ..field
            }),
            Description::List(_) => panic!("Option<List> is not supported"),
        }
    }

    fn default() -> Result<Self, NotSupported> {
        Ok(None)
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

    fn parse(input: &str) -> Result<Result<Self, ParseError>, NotSupported> {
        // NOTE: we parse Option<T> just like T and assume Some. There is no way to parse
        // None.
        match C::parse(input) {
            Ok(Ok(inner)) => Ok(Ok(OptionChange::Apply(inner))),
            Ok(Err(parse_err)) => Ok(Err(parse_err)),
            Err(_) => Err(NotSupported),
        }
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
                ChangeVerb::List(_) => Err(VerbError::UnsupportedVerb(verb)),
            },
            Description::Field(desc) => {
                if path.next().is_some() {
                    return Err(VerbError::InvalidPath);
                }

                match verb {
                    ChangeVerb::Set(value) => {
                        let change = Self::parse(&value)??;
                        Ok(self_cast(change))
                    }
                    ChangeVerb::SetAny(value) => {
                        let change = value.downcast().map_err(|_| VerbError::DowncastFailed)?;
                        Ok(OptionChange::Apply(*change))
                    }
                    ChangeVerb::SetFlag => Err(VerbError::UnsupportedVerb(verb)),
                    ChangeVerb::Unset => Ok(OptionChange::Unset),
                    ChangeVerb::UseDefault => Ok(OptionChange::Unset),
                    ChangeVerb::List(_) => Err(VerbError::UnsupportedVerb(verb)),
                }
            }
            Description::List(_) => Err(VerbError::InvalidDescription),
        }
    }

    fn unwrap_field(self) -> Result<Self::Configuration, Self> {
        Err(self)
    }
}
