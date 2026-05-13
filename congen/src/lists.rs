use std::{borrow::Cow, iter::empty, mem::take};

use crate::{
    ChangeVerb, Configuration, CongenChange, Description, FieldDescription, ListDescription,
    ListKey, ListVerb, NotSupported, VerbError, self_cast,
};

#[derive(Debug, Default)]
pub enum VecChange<T> {
    Append(T),
    Update(usize, T),
    Remove(usize),
    Empty,
    #[default]
    NoChange,
    ApplyMany(Vec<VecChange<T>>),
}

impl<T> Configuration for Vec<T>
where
    T: Configuration + 'static,
    VecChange<T::CongenChange>: CongenChange,
{
    type CongenChange = VecChange<T::CongenChange>;

    fn apply_change_with_default(
        &mut self,
        change: Self::CongenChange,
        default: Option<fn() -> Self>,
    ) {
        let assert_key = |index: usize| {
            assert!(
                index < self.len(),
                "User specified index {index} is to large. List is 0 indexed with a lenght of {} and a largest index of {}",
                self.len(),
                if self.is_empty() {
                    "NaN".to_string()
                } else {
                    format!("{}", self.len() - 1)
                }
            );
        };
        match change {
            VecChange::Append(change) => {
                let change = match change.unwrap_field() {
                    Ok(value) => {
                        self.push(self_cast(value));
                        return;
                    }
                    Err(change) => change,
                };
                if let Some(default) = default {
                    let mut default_vec = default();
                    if default_vec.len() == 1 {
                        let mut new_value = default_vec.pop().expect("just checked the lenght");
                        new_value.apply_change(change);
                        self.push(new_value);
                        return;
                    }
                }
                panic!(
                    "called apply_change_with_default on Append to Vec without a default constructor"
                );
            }
            VecChange::Update(index, change) => {
                assert_key(index);
                let current = &mut self[index];
                current.apply_change(change);
            }
            VecChange::Remove(index) => {
                assert_key(index);
                self.remove(index);
            }
            VecChange::Empty => self.clear(),
            VecChange::NoChange => {}
            VecChange::ApplyMany(changes) => {
                for change in changes {
                    self.apply_change_with_default(change, default);
                }
            }
        }
    }

    fn description(field_name: &'static str) -> Description {
        let inner_desc = Box::new(T::description(""));

        ListDescription {
            field_name,
            type_name: Self::type_name(),
            inner_desc,
            has_default: false,
            key_is_int: true,
        }
        .into()
    }

    fn default() -> Result<Self, NotSupported> {
        Ok(Vec::new())
    }

    fn type_name() -> Cow<'static, str> {
        format!("List<{}>", T::type_name()).into()
    }
}

impl<C> CongenChange for VecChange<C>
where
    C: CongenChange + 'static,
    C::Configuration: 'static,
{
    type Configuration = Vec<C::Configuration>;

    fn empty() -> Self {
        VecChange::NoChange
    }

    fn apply_change(&mut self, change: Self) {
        if matches!(change, VecChange::NoChange) {
            return;
        }
        if matches!(change, VecChange::Empty) {
            *self = VecChange::Empty;
            return;
        }

        let mut changes = match self {
            VecChange::NoChange => {
                *self = change;
                return;
            }
            VecChange::ApplyMany(changes) => take(changes),
            _ => {
                let current = take(self);
                vec![current]
            }
        };
        changes.push(change);
        *self = VecChange::ApplyMany(changes);
    }

    fn from_path_and_verb<'a, P>(mut path: P, verb: ChangeVerb) -> Result<Self, VerbError>
    where
        P: Iterator<Item = &'a str>,
    {
        assert!(path.next().is_none(), "field path should end at List<T>");

        let inner_desc = C::Configuration::description("");

        match verb {
            ChangeVerb::Set(_)
            | ChangeVerb::SetFlag
            | ChangeVerb::Unset
            | ChangeVerb::UseDefault
            | ChangeVerb::SetAny(_) => Err(VerbError::UnsupportedVerb(verb)),
            ChangeVerb::List(list) => match inner_desc {
                Description::Field(field_description) => {
                    Self::inner_field_from_verb(list, &field_description)
                }
                Description::Composit(_composit_description) => {
                    todo!("composite in list from verb")
                }
                Description::List(_list_description) => todo!("list in list from verb"),
            },
        }
    }
}

impl<C> VecChange<C>
where
    C: CongenChange + 'static,
    C::Configuration: 'static,
{
    fn inner_field_from_verb(
        verb: ListVerb,
        _field_desc: &FieldDescription,
    ) -> Result<Self, VerbError> {
        match verb {
            ListVerb::Append { new_value } => Ok(VecChange::Append(C::from_path_and_verb(
                empty(),
                ChangeVerb::Set(new_value),
            )?)),
            ListVerb::Update { key, updated_value } => {
                let ListKey::Int(key) = key else {
                    return Err(VerbError::WrongKeyType);
                };
                Ok(VecChange::Update(
                    key,
                    C::from_path_and_verb(empty(), ChangeVerb::Set(updated_value))?,
                ))
            }
            ListVerb::Remove { key } => {
                let ListKey::Int(key) = key else {
                    return Err(VerbError::WrongKeyType);
                };
                Ok(VecChange::Remove(key))
            }
            ListVerb::Empty => Ok(VecChange::Empty),
        }
    }
}
