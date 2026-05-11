use std::{borrow::Cow, mem::take};

use crate::{Configuration, CongenChange, Description, ListDescription, NotSupported, self_cast};

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
                self.len() - 1
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

    fn from_path_and_verb<'a, P>(
        _path: P,
        _verb: crate::ChangeVerb,
    ) -> Result<Self, crate::VerbError>
    where
        P: Iterator<Item = &'a str>,
    {
        todo!()
    }
}
