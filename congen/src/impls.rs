use crate::{
    CompositDescription, Configuration, ConfigurationDefault, ConfigurationFlag,
    ConfigurationUnset, Description, FieldDescription, NotSupported,
};

pub enum OptionChange<T> {
    Some(T),
    None,
    Unchanged,
}

impl<T> Configuration for Option<T>
where
    T: Configuration,
{
    type CongenChange = OptionChange<T::CongenChange>;

    fn apply_change(&mut self, change: OptionChange<T::CongenChange>) {
        match change {
            OptionChange::Some(change) => match self {
                Some(inner) => inner.apply_change(change),
                None => {
                    if let Ok(mut new) = <T as Configuration>::default() {
                        new.apply_change(change);
                        *self = Some(new);
                    } else if let Ok(new) = <T as Configuration>::unwrap_change(change) {
                        *self = Some(new);
                    } else {
                        panic!(
                            "`Configuration` implementation of `{}` is inconsistent.
                            It either needs to implement default or unwrap_change",
                            core::any::type_name::<T>()
                        );
                    }
                }
            },
            OptionChange::None => *self = None,
            OptionChange::Unchanged => return,
        }
    }

    fn description(field_name: Option<&'static str>) -> Description {
        let child_desc = <T as Configuration>::description(None);
        match child_desc {
            Description::Composit(strukt) => CompositDescription {
                field_name,
                type_name: Self::type_name(),
                fields: Vec::new(),
                composites: vec![strukt],
            }
            .into(),
            Description::Field(_field) => FieldDescription {
                field_name: field_name.unwrap_or("_"),
                type_name: Self::type_name(),
                is_flag: false,
                allow_unset: true,
                has_default: true, // TODO how do I want to handle default here?
            }
            .into(),
        }
    }

    fn type_name() -> std::borrow::Cow<'static, str> {
        format!("Option<{}>", <T as Configuration>::type_name()).into()
    }
}

impl<T> ConfigurationUnset for Option<T>
where
    Option<T>: Configuration,
{
    fn unset_value() -> Self {
        None
    }
}

// TODO remove
impl<T> ConfigurationDefault for Option<T> where Option<T>: Configuration {}

impl Configuration for bool {
    type CongenChange = Option<bool>;

    fn apply_change(&mut self, change: Self::CongenChange) {
        if let Some(new) = change {
            *self = new
        }
    }

    fn description(field_name: Option<&'static str>) -> Description {
        FieldDescription {
            field_name: field_name.unwrap_or("_"),
            type_name: Self::type_name(),
            is_flag: true,
            allow_unset: false,
            has_default: false,
        }
        .into()
    }

    fn default() -> Result<Self, crate::NotSupported> {
        Ok(false)
    }

    fn unwrap_change(change: Self::CongenChange) -> Result<Self, NotSupported> {
        Ok(change.unwrap())
    }

    fn type_name() -> std::borrow::Cow<'static, str> {
        "bool".into()
    }
}
impl ConfigurationUnset for bool {
    fn unset_value() -> Self {
        false
    }
}
impl ConfigurationDefault for bool {}
impl ConfigurationFlag for bool {
    fn flag() -> Self {
        true
    }
}

impl Configuration for String {
    type CongenChange = Option<String>;

    fn apply_change(&mut self, change: Self::CongenChange) {
        if let Some(value) = change {
            *self = value;
        }
    }

    fn description(field_name: Option<&'static str>) -> Description {
        FieldDescription {
            field_name: field_name.unwrap_or("_"),
            type_name: Self::type_name(),
            is_flag: false,
            allow_unset: true,
            has_default: false,
        }
        .into()
    }

    fn unwrap_change(change: Self::CongenChange) -> Result<Self, NotSupported> {
        Ok(change.unwrap())
    }

    fn type_name() -> std::borrow::Cow<'static, str> {
        "String".into()
    }
}
impl ConfigurationUnset for String {
    fn unset_value() -> Self {
        String::new()
    }
}

impl Configuration for u32 {
    type CongenChange = Option<u32>;

    fn apply_change(&mut self, change: Self::CongenChange) {
        if let Some(value) = change {
            *self = value;
        }
    }

    fn description(field_name: Option<&'static str>) -> Description {
        FieldDescription {
            field_name: field_name.unwrap_or("_"),
            type_name: Self::type_name(),
            is_flag: false,
            allow_unset: false,
            has_default: false,
        }
        .into()
    }

    fn unwrap_change(change: Self::CongenChange) -> Result<Self, NotSupported> {
        Ok(change.unwrap())
    }

    fn type_name() -> std::borrow::Cow<'static, str> {
        "u32".into()
    }
}
