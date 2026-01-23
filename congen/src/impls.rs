use crate::{Configuration, Description, FieldDescription, NotSupported};

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
            allow_unset: true,
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
