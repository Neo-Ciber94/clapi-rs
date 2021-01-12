use crate::serde::internal::{AnyToString, StringOrList, ValidType};
use crate::{ValueCount, Argument, ArgumentList, Command, CommandOption, OptionList};
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::export::{Formatter, Result};
use serde::ser::{SerializeSeq, SerializeStruct};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

// ArgCount
impl Serialize for ValueCount {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ArgCount", 2)?;
        state.serialize_field("min_count", &self.min_count())?;
        state.serialize_field("max_count", &self.max_count())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for ValueCount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const FIELDS: &'static [&'static str] = &["min_count", "max_count"];

        enum Field {
            Min,
            Max,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;
                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                        formatter.write_str("`min_count` or `max_count`")
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match v {
                            "min_count" => Ok(Field::Min),
                            "max_count" => Ok(Field::Max),
                            _ => Err(de::Error::unknown_field(v, FIELDS)),
                        }
                    }

                    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match v {
                            b"min_count" => Ok(Field::Min),
                            b"max_count" => Ok(Field::Max),
                            _ => {
                                let value = String::from_utf8_lossy(v);
                                Err(de::Error::unknown_field(&value, FIELDS))
                            }
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct ArgCountVisitor;
        impl<'de> Visitor<'de> for ArgCountVisitor {
            type Value = ValueCount;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                formatter.write_str("struct ArgCount")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let min = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let max = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                Ok(ValueCount::new_checked(min, max).expect("min < max"))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut min : Option<Option<usize>> = None;
                let mut max : Option<Option<usize>> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Min => {
                            if min.is_some() {
                                return Err(de::Error::duplicate_field("min_count"));
                            }
                            min = Some(map.next_value()?);
                        }
                        Field::Max => {
                            if max.is_some() {
                                return Err(de::Error::duplicate_field("max_count"));
                            }

                            max = Some(map.next_value()?);
                        }
                    }
                }

                Ok(ValueCount::new_checked(min.flatten(), max.flatten()).expect("min < max"))
            }
        }

        deserializer.deserialize_struct("ArgCount", FIELDS, ArgCountVisitor)
    }
}

//  Argument
impl Serialize for Argument {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use crate::validator::Validator;
        fn get_valid_type(validator: Option<&dyn Validator>) -> Option<ValidType> {
            match validator {
                Some(v) => {
                    match v.valid_type() {
                        Some(ref valid_type) => ValidType::from_type(valid_type),
                        None => None
                    }
                },
                None => None
            }
        }

        let mut state = serializer.serialize_struct("Argument", 7)?;
        state.serialize_field("name", &self.get_name())?;
        state.serialize_field("description", &self.get_description())?;
        state.serialize_field("min_count", &self.get_value_count().min_count())?;
        state.serialize_field("max_count", &self.get_value_count().max_count())?;
        state.serialize_field("type", &get_valid_type(self.get_validator()))?;
        state.serialize_field("valid_values", &self.get_valid_values())?;
        state.serialize_field("default_values", &self.get_default_values())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Argument {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const FIELDS: &'static [&'static str] = &[
            "name",
            "description",
            "min_count",
            "max_count",
            "type",
            "valid_values",
            "default_values",
        ];

        enum Field {
            Name,
            Description,
            MinCount,
            MaxCount,
            Type,
            ValidValues,
            DefaultValues,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;
                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                        formatter.write_str("`name`, `description`, `min_count`, `max_count`, `type`, `valid_values` or `default_values`")
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match v {
                            "name" => Ok(Field::Name),
                            "description" => Ok(Field::Description),
                            "min_count" => Ok(Field::MinCount),
                            "max_count" => Ok(Field::MaxCount),
                            "type" => Ok(Field::Type),
                            "valid_values" => Ok(Field::ValidValues),
                            "default_values" => Ok(Field::DefaultValues),
                            _ => Err(de::Error::unknown_field(v, FIELDS)),
                        }
                    }

                    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match v {
                            b"name" => Ok(Field::Name),
                            b"description" => Ok(Field::Description),
                            b"min_count" => Ok(Field::MinCount),
                            b"max_count" => Ok(Field::MaxCount),
                            b"type" => Ok(Field::Type),
                            b"valid_values" => Ok(Field::ValidValues),
                            b"default_values" => Ok(Field::DefaultValues),
                            _ => {
                                let value = String::from_utf8_lossy(v);
                                Err(de::Error::unknown_field(&value, FIELDS))
                            }
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct ArgumentVisitor;
        impl<'de> Visitor<'de> for ArgumentVisitor {
            type Value = Argument;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                formatter.write_str("struct Argument")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, <A as SeqAccess<'de>>::Error>
            where
                A: SeqAccess<'de>,
            {
                let name: String = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;

                let description: Option<String> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                let min_count: Option<usize> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;

                let max_count: Option<usize> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;

                let valid_type: Option<ValidType> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(4, &self))?;

                let valid_values: Vec<String> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(5, &self))?;

                let default_values: Vec<String> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(6, &self))?;

                let mut argument = Argument::with_name(name);

                match (min_count, max_count) {
                    (Some(min), Some(max)) => {
                        argument = argument.value_count(ValueCount::new(min, max));
                    },
                    (Some(min), None) => {
                        argument = argument.value_count(ValueCount::more_than(min));
                    },
                    (None, Some(max)) => {
                        argument = argument.value_count(ValueCount::less_than(max));
                    }
                    (None, None) => { /*By default an `Argument` takes 1 value */ }
                }

                if let Some(description) = description {
                    argument = argument.description(description);
                }

                if let Some(valid_type) = valid_type {
                    argument = valid_type.set_validator(argument);
                }

                if valid_values.len() > 0 {
                    argument = argument.valid_values(valid_values);
                }

                if default_values.len() > 0 {
                    argument = argument.defaults(default_values);
                }

                Ok(argument)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, <A as MapAccess<'de>>::Error>
            where
                A: MapAccess<'de>,
            {
                let mut name: Option<String> = None;
                let mut description: Option<Option<String>> = None;
                let mut min_count: Option<Option<usize>> = None;
                let mut max_count: Option<Option<usize>> = None;
                let mut valid_type : Option<Option<ValidType>> = None;
                let mut valid_values: Option<Vec<String>> = None;
                let mut default_values: Option<Vec<String>> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Name => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }

                            name = Some(map.next_value()?);
                        }
                        Field::Description => {
                            if description.is_some() {
                                return Err(de::Error::duplicate_field("description"));
                            }

                            description = Some(map.next_value()?);
                        }
                        Field::MinCount => {
                            if min_count.is_some() {
                                return Err(de::Error::duplicate_field("min_count"));
                            }

                            min_count = Some(map.next_value()?);
                        }
                        Field::MaxCount => {
                            if max_count.is_some() {
                                return Err(de::Error::duplicate_field("max_count"));
                            }

                            max_count = Some(map.next_value()?);
                        }
                        Field::Type => {
                            if valid_type.is_some() {
                                return Err(de::Error::duplicate_field("type"));
                            }

                            valid_type = Some(map.next_value()?);
                        }
                        Field::ValidValues => {
                            if valid_values.is_some() {
                                return Err(de::Error::duplicate_field("valid_values"));
                            }

                            valid_values = Some(
                                map.next_value::<Vec<AnyToString>>()?
                                    .into_iter()
                                    .map(|s| s.0)
                                    .collect::<Vec<String>>(),
                            );
                        }
                        Field::DefaultValues => {
                            if default_values.is_some() {
                                return Err(de::Error::duplicate_field("default_values"));
                            }

                            default_values = Some(
                                map.next_value::<Vec<AnyToString>>()?
                                    .into_iter()
                                    .map(|s| s.0)
                                    .collect::<Vec<String>>(),
                            );
                        }
                    }
                }

                let mut argument =
                    Argument::with_name(name.ok_or_else(|| de::Error::missing_field("name"))?);

                if let Some(Some(description)) = description {
                    argument = argument.description(description);
                }

                match (min_count.flatten(), max_count.flatten()) {
                    (Some(min), Some(max)) => {
                        argument = argument.value_count(ValueCount::new(min, max));
                    },
                    (Some(min), None) => {
                        argument = argument.value_count(ValueCount::more_than(min));
                    },
                    (None, Some(max)) => {
                        argument = argument.value_count(ValueCount::less_than(max));
                    }
                    (None, None) => { /*By default an `Argument` takes 1 value */ }
                }

                if let Some(Some(valid_type)) = valid_type {
                    argument = valid_type.set_validator(argument);
                }

                if let Some(valid_values) = valid_values {
                    if valid_values.len() > 0 {
                        argument = argument.valid_values(valid_values);
                    }
                }

                if let Some(default_values) = default_values {
                    if default_values.len() > 0 {
                        argument = argument.defaults(default_values);
                    }
                }

                Ok(argument)
            }
        }

        deserializer.deserialize_struct("Argument", FIELDS, ArgumentVisitor)
    }
}

// ArgumentList
impl Serialize for ArgumentList {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for arg in self {
            seq.serialize_element(arg)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for ArgumentList {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        struct ArgumentListVisitor;
        impl<'de> Visitor<'de> for ArgumentListVisitor {
            type Value = ArgumentList;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                formatter.write_str("struct ArgumentList")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, <A as SeqAccess<'de>>::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut args = ArgumentList::new();
                while let Some(next_arg) = seq.next_element()? {
                    args.add(next_arg).unwrap();
                }
                Ok(args)
            }
        }

        deserializer.deserialize_seq(ArgumentListVisitor)
    }
}

// CommandOption
impl Serialize for CommandOption {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("CommandOption", 5)?;
        state.serialize_field("name", self.get_name())?;

        if self.get_aliases().count() == 1 {
            state.serialize_field("alias", &self.get_aliases().next().cloned().unwrap())?;
        } else {
            state.serialize_field(
                "aliases",
                &self.get_aliases().cloned().collect::<Vec<String>>(),
            )?;
        }

        state.serialize_field("description", &self.get_description())?;
        state.serialize_field("required", &self.is_required())?;
        state.serialize_field("args", self.get_args())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for CommandOption {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        const FIELDS: &'static [&'static str] = &[
            "name",
            "alias",
            "aliases",
            "description",
            "required",
            "args",
        ];

        enum Field {
            Name,
            Aliases,
            Description,
            Required,
            Args,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;
                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                        formatter.write_str(
                            "`name`,  `alias`, `aliases`, `description`, `required` or `args`",
                        )
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match v {
                            "name" => Ok(Field::Name),
                            "aliases" | "alias" => Ok(Field::Aliases),
                            "description" => Ok(Field::Description),
                            "required" => Ok(Field::Required),
                            "args" => Ok(Field::Args),
                            _ => return Err(de::Error::unknown_field(v, FIELDS)),
                        }
                    }

                    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match v {
                            b"name" => Ok(Field::Name),
                            b"aliases" | b"alias" => Ok(Field::Aliases),
                            b"description" => Ok(Field::Description),
                            b"required" => Ok(Field::Required),
                            b"args" => Ok(Field::Args),
                            _ => {
                                let value = String::from_utf8_lossy(v);
                                return Err(de::Error::unknown_field(&value, FIELDS));
                            }
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct CommandOptionVisitor;
        impl<'de> Visitor<'de> for CommandOptionVisitor {
            type Value = CommandOption;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                formatter.write_str("struct CommandOption")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, <A as SeqAccess<'de>>::Error>
            where
                A: SeqAccess<'de>,
            {
                let name: String = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;

                let aliases: Vec<String> = seq
                    .next_element::<StringOrList>()?
                    .map(|s| match s {
                        StringOrList::String(s) => vec![s],
                        StringOrList::List(list) => list,
                    })
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                let description: Option<String> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;

                let is_required: bool = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;

                let args: ArgumentList = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(4, &self))?;

                let mut option = CommandOption::new(name).required(is_required).args(args);

                if aliases.len() > 0 {
                    for alias in aliases {
                        option = option.alias(alias);
                    }
                }

                if let Some(description) = description {
                    option = option.description(description);
                }

                Ok(option)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, <A as MapAccess<'de>>::Error>
            where
                A: MapAccess<'de>,
            {
                let mut name: Option<String> = None;
                let mut aliases: Option<Vec<String>> = None;
                let mut description: Option<Option<String>> = None;
                let mut required: Option<bool> = None;
                let mut args: Option<ArgumentList> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Name => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }

                            name = Some(map.next_value()?);
                        }
                        Field::Aliases => {
                            if aliases.is_some() {
                                return Err(de::Error::duplicate_field("alias"));
                            }

                            aliases = match map.next_value::<StringOrList>()? {
                                StringOrList::String(s) => Some(vec![s]),
                                StringOrList::List(list) => Some(list),
                            };
                            //aliases = Some(map.next_value()?);
                        }
                        Field::Description => {
                            if description.is_some() {
                                return Err(de::Error::duplicate_field("description"));
                            }

                            description = Some(map.next_value()?);
                        }
                        Field::Required => {
                            if required.is_some() {
                                return Err(de::Error::duplicate_field("required"));
                            }

                            required = Some(map.next_value()?);
                        }
                        Field::Args => {
                            if args.is_some() {
                                return Err(de::Error::duplicate_field("args"));
                            }

                            args = Some(map.next_value()?);
                        }
                    }
                }

                let mut option =
                    CommandOption::new(name.ok_or_else(|| de::Error::missing_field("name"))?);

                if let Some(aliases) = aliases {
                    for alias in aliases {
                        // Prints duplicated aliases warnings
                        if option.has_alias(&alias) {
                            println!("duplicated alias `{}`", alias);
                        }

                        option = option.alias(alias);
                    }
                }

                if let Some(Some(description)) = description {
                    option = option.description(description);
                }

                if let Some(required) = required {
                    option = option.required(required);
                }

                if let Some(args) = args {
                    option = option.args(args);
                }

                Ok(option)
            }
        }

        deserializer.deserialize_struct("CommandOption", FIELDS, CommandOptionVisitor)
    }
}

// OptionList
impl Serialize for OptionList {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for option in self {
            seq.serialize_element(option)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for OptionList {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        struct OptionListVisitor;
        impl<'de> Visitor<'de> for OptionListVisitor {
            type Value = OptionList;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                formatter.write_str("struct OptionList")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, <A as SeqAccess<'de>>::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut options = OptionList::new();
                while let Some(next_option) = seq.next_element()? {
                    options.add(next_option).unwrap();
                }
                Ok(options)
            }
        }

        deserializer.deserialize_seq(OptionListVisitor)
    }
}

// Command
impl Serialize for Command {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Command", 6)?;
        state.serialize_field("name", self.get_name())?;
        state.serialize_field("description", &self.get_description())?;
        state.serialize_field("about", &self.get_usage())?;
        state.serialize_field(
            "subcommands",
            &self.get_children().cloned().collect::<Vec<Command>>(),
        )?;
        state.serialize_field("options", &self.get_options())?;
        state.serialize_field("args", &self.get_args())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Command {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        const FIELDS: &'static [&'static str] = &[
            "name",
            "description",
            "about",
            "subcommands",
            "options",
            "args",
        ];

        enum Field {
            Name,
            Description,
            About,
            Subcommands,
            Options,
            Args,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;
                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                        formatter.write_str(
                            "`name`, `description`, `about`, `subcommands`, `options` or `args`",
                        )
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match v {
                            "name" => Ok(Field::Name),
                            "description" => Ok(Field::Description),
                            "about" => Ok(Field::About),
                            "subcommands" => Ok(Field::Subcommands),
                            "options" => Ok(Field::Options),
                            "args" => Ok(Field::Args),
                            _ => return Err(de::Error::unknown_field(v, FIELDS)),
                        }
                    }

                    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match v {
                            b"name" => Ok(Field::Name),
                            b"description" => Ok(Field::Description),
                            b"about" => Ok(Field::About),
                            b"subcommands" => Ok(Field::Subcommands),
                            b"options" => Ok(Field::Options),
                            b"args" => Ok(Field::Args),
                            _ => {
                                let value = String::from_utf8_lossy(v);
                                return Err(de::Error::unknown_field(&value, FIELDS));
                            }
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct CommandVisitor;
        impl<'de> Visitor<'de> for CommandVisitor {
            type Value = Command;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                formatter.write_str("struct Command")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let name: String = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;

                let description: Option<String> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                let about: Option<String> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;

                let subcommands: Vec<Command> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;

                let options: OptionList = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(4, &self))?;

                let args: ArgumentList = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(5, &self))?;

                let mut command = Command::new(name).options(options).args(args);

                for subcommand in subcommands {
                    command = command.subcommand(subcommand);
                }

                if let Some(description) = description {
                    command = command.description(description);
                }

                if let Some(about) = about {
                    command = command.usage(about);
                }

                Ok(command)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut name: Option<String> = None;
                let mut description: Option<Option<String>> = None;
                let mut about: Option<Option<String>> = None;
                let mut subcommands: Option<Vec<Command>> = None;
                let mut options: Option<OptionList> = None;
                let mut args: Option<ArgumentList> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Name => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }

                            name = Some(map.next_value()?);
                        }
                        Field::Description => {
                            if description.is_some() {
                                return Err(de::Error::duplicate_field("description"));
                            }

                            description = Some(map.next_value()?);
                        }
                        Field::About => {
                            if about.is_some() {
                                return Err(de::Error::duplicate_field("about"));
                            }

                            about = Some(map.next_value()?);
                        }
                        Field::Subcommands => {
                            if subcommands.is_some() {
                                return Err(de::Error::duplicate_field("subcommands"));
                            }

                            subcommands = Some(map.next_value()?);
                        }
                        Field::Options => {
                            if options.is_some() {
                                return Err(de::Error::duplicate_field("options"));
                            }

                            options = Some(map.next_value()?);
                        }
                        Field::Args => {
                            if args.is_some() {
                                return Err(de::Error::duplicate_field("args"));
                            }

                            args = Some(map.next_value()?);
                        }
                    }
                }

                let mut command =
                    Command::new(name.ok_or_else(|| de::Error::missing_field("name"))?);

                if let Some(Some(description)) = description {
                    command = command.description(description);
                }

                if let Some(Some(about)) = about {
                    command = command.usage(about);
                }

                if let Some(subcommands) = subcommands {
                    for subcommand in subcommands {
                        command = command.subcommand(subcommand)
                    }
                }

                if let Some(options) = options {
                    command = command.options(options)
                }

                if let Some(args) = args {
                    command = command.args(args)
                }

                Ok(command)
            }
        }

        deserializer.deserialize_struct("Command", FIELDS, CommandVisitor)
    }
}

mod internal {
    use serde::de::Visitor;
    use serde::export::{fmt, Formatter};
    use serde::{de, Serialize, Deserialize, Deserializer};
    use crate::args::validator::{Type, parse_validator};
    use std::any::TypeId;
    use std::net::{IpAddr, SocketAddr};
    use crate::Argument;
    use std::fmt::Display;

    macro_rules! visit_to_string {
        ($($method:ident $ty:ty),+ $(,)?) => {
            $(
                fn $method<E>(self, value: $ty) -> Result<Self::Value, E> where E: de::Error, {
                    Ok(AnyToString(value.to_string()))
                }
            )+
        };
    }

    /// Used for serialize any value to `String`.
    ///
    /// # Supported types:
    /// - `bool`
    /// - `char`
    /// - `String`
    /// - `&str`
    /// - `integer`s (i8, i16, i32, i64, i128, u8, u16, u32, u64, u128)
    /// - `float`s (f32, f64)
    pub struct AnyToString(pub String);

    impl<'de> Deserialize<'de> for AnyToString {
        fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
        where
            D: Deserializer<'de>,
        {
            struct AnyToStringVisitor;
            impl<'de> Visitor<'de> for AnyToStringVisitor {
                type Value = AnyToString;

                fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                    formatter.write_str("`bool`, `char`, `integer`, `float` or `string`")
                }

                visit_to_string! {
                    visit_bool bool, visit_char char, visit_str &str,
                    visit_i8 i8, visit_i16 i16, visit_i32 i32, visit_i64 i64,
                    visit_u8 u8, visit_u16 u16, visit_u32 u32, visit_u64 u64,
                    visit_f32 f32, visit_f64 f64
                }

                serde::serde_if_integer128! {
                    visit_to_string! {
                        visit_i128 i128, visit_u128 u128
                    }
                }
            }

            deserializer.deserialize_any(AnyToStringVisitor)
        }
    }

    /// Some properties can be represented as a single string or a list of string,
    /// for example the `CommandOption` aliases.
    #[derive(Deserialize)]
    #[serde(untagged)]
    pub enum StringOrList {
        String(String),
        List(Vec<String>),
    }

    /// Declares the enum `ValidType` used for serialize the type of an argument.
    macro_rules! declare_impl_valid_type {
        ('primitives $($ty:ty => $variant:ident),+
            'other $($ty2:ty => $variant2:ident $name:literal),* $(,)?) => {

            #[serde(rename_all="lowercase")]
            #[derive(Debug, Clone, Serialize, Deserialize)]
            pub enum ValidType {
                $(
                    $variant
                ),+,
                $(
                    #[serde(rename=$name)]
                    $variant2
                ),*
            }

            impl ValidType {
                pub fn from_type(ty: &Type) -> Option<ValidType>{
                    let id = ty.id();
                    $(
                        if TypeId::of::<$ty>() == id {
                            return Some(ValidType::$variant);
                        }
                    )+

                     $(
                        if TypeId::of::<$ty2>() == id {
                            return Some(ValidType::$variant2);
                        }
                     )*

                    None
                }

                pub fn set_validator(&self, arg: Argument) -> Argument {
                    match self {
                        $(
                            ValidType::$variant => arg.validator(parse_validator::<$ty>())
                        ),+,
                        $(
                            ValidType::$variant2 => arg.validator(parse_validator::<$ty2>())
                        ),*
                    }
                }

                pub fn as_str(&self) -> &'static str {
                    match self {
                        $(
                            ValidType::$variant => stringify!($ty)
                        ),+,
                        $(
                            ValidType::$variant2 => $name
                        ),*
                    }
                }
            }

            impl Display for ValidType {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", self.as_str())
                }
            }
        };
    }

    declare_impl_valid_type!{
        'primitives
        i8 => I8,
        i16 => I16,
        i32 => I32,
        i64 => I64,
        i128 => I128,
        isize => ISize,
        u8 => U8,
        u16 => U16,
        u32 => U32,
        u64 => U64,
        u128 => U128,
        usize => USize,
        f32 => F32,
        f64 => F64,
        bool => Bool,
        char => Char

        'other
        String => String "string",
        IpAddr => IpAddress "ip_address",
        SocketAddr => SocketAddress "socket_address",
    }
}

#[cfg(test)]
mod tests {
    use serde_test::Token;
    use crate::serde::internal::ValidType;

    #[cfg(test)]
    mod arg_count_tests {
        use crate::ValueCount;
        use serde_test::Token;

        #[test]
        fn arg_count_test1() {
            let arg_count = ValueCount::new(2, 10);
            serde_test::assert_tokens(
                &arg_count,
                &[
                    Token::Struct {
                        name: "ArgCount",
                        len: 2,
                    },
                    Token::Str("min_count"),
                    Token::Some,
                    Token::U64(2),
                    Token::Str("max_count"),
                    Token::Some,
                    Token::U64(10),
                    Token::StructEnd,
                ],
            )
        }

        #[test]
        fn arg_count_test2() {
            let arg_count = ValueCount::less_than(10);
            serde_test::assert_tokens(
                &arg_count,
                &[
                    Token::Struct {
                        name: "ArgCount",
                        len: 2,
                    },
                    Token::Str("min_count"),
                    Token::None,
                    Token::Str("max_count"),
                    Token::Some,
                    Token::U64(10),
                    Token::StructEnd,
                ],
            )
        }

        #[test]
        fn arg_count_test3() {
            let arg_count = ValueCount::any();
            serde_test::assert_tokens(
                &arg_count,
                &[
                    Token::Struct {
                        name: "ArgCount",
                        len: 2,
                    },
                    Token::Str("min_count"),
                    Token::None,
                    Token::Str("max_count"),
                    Token::None,
                    Token::StructEnd,
                ],
            )
        }
    }

    #[cfg(test)]
    mod args_tests {
        use super::*;
        use crate::{Argument, ArgumentList};
        use serde_test::Token;
        use crate::args::validator::{parse_validator, Type};

        #[test]
        fn argument_test() {
            let arg = Argument::with_name("numbers")
                .description("A set of numbers")
                .value_count(1..=10)
                .validator(parse_validator::<i64>())
                .valid_values(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9])
                .defaults(&[1, 2, 3]);

            serde_test::assert_tokens(
                &arg,
                &args_tokens(
                    "numbers",
                    Some("A set of numbers"),
                    Some(1),
                    Some(10),
                    Some(ValidType::I64),
                    vec!["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"],
                    vec!["1", "2", "3"],
                ),
            );
        }

        #[test]
        fn argument_missing_fields_test1() {
            let arg = Argument::with_name("numbers");

            serde_test::assert_tokens(&arg, &args_tokens(
                "numbers",
                None,
                Some(1),
                Some(1),
                None,
                vec![],
                vec![]
            ));
        }

        #[test]
        fn argument_missing_fields_test2() {
            let arg = Argument::with_name("numbers")
                .valid_values(&[0, 1, 2, 3])
                .default(0);

            serde_test::assert_tokens(
                &arg,
                &args_tokens(
                    "numbers",
                    None,
                    Some(1),
                    Some(1),
                    None,
                    vec!["0", "1", "2", "3"],
                    vec!["0"]
                ),
            );
        }

        #[test]
        fn argument_from_json_test() {
            let arg = serde_json::from_str::<Argument>(
                r#"
        {
            "name": "numbers",
            "valid_values": [1,2,3],
            "type" : "i32"
        }
        "#,
            )
            .unwrap();

            assert_eq!(arg.get_name(), "numbers");
            assert_eq!(
                arg.get_validator().unwrap().valid_type(),
                Some(Type::of::<i32>())
            );
            assert_eq!(
                arg.get_valid_values(),
                &["1".to_owned(), "2".to_owned(), "3".to_owned()]
            );
        }

        #[test]
        fn argument_list() {
            let mut args = ArgumentList::new();
            args.add(Argument::with_name("A").description("a")).unwrap();
            args.add(Argument::with_name("B").value_count(2)).unwrap();
            args.add(Argument::with_name("C").valid_values(&['a', 'b', 'c']))
                .unwrap();

            let mut tokens = Vec::new();
            tokens.push(Token::Seq { len: Some(3) });
            tokens.extend(args_tokens("A", Some("a"), Some(1), Some(1), None, vec![], vec![]));
            tokens.extend(args_tokens("B", None, Some(2), Some(2), None, vec![], vec![]));
            tokens.extend(args_tokens("C", None, Some(1), Some(1), None, vec!["a", "b", "c"], vec![]));
            tokens.push(Token::SeqEnd);

            serde_test::assert_tokens(&args, tokens.as_slice());
        }
    }

    #[cfg(test)]
    mod options_tests {
        use crate::serde::tests::{args_tokens, option_tokens};
        use crate::{ValueCount, Argument, CommandOption, OptionList};
        use serde_test::Token;

        #[test]
        fn option_test() {
            let opt = CommandOption::new("time")
                .alias("t")
                .alias("T")
                .description("Number of times")
                .required(false)
                .arg(Argument::with_name("N"));

            let mut args = Vec::new();
            args.extend(args_tokens(
                "N",
                None,
                Some(1),
                Some(1),
                None,
                vec![],
                vec![]
            ));

            serde_test::assert_tokens(
                &opt,
                &option_tokens(
                    "time",
                    vec!["t", "T"],
                    Some("Number of times"),
                    false,
                    (1, args),
                ),
            );
        }

        #[test]
        fn option_missing_fields_test() {
            let option = CommandOption::new("color")
                .required(true)
                .arg(Argument::with_name("color").valid_values(vec!["red", "blue", "green"]));

            let args = Vec::from(args_tokens(
                "color",
                None,
                Some(1),
                Some(1),
                None,
                vec!["red", "blue", "green"],
                vec![],
            ));

            serde_test::assert_tokens(
                &option,
                &option_tokens("color", vec![], None, true, (1, args)),
            )
        }

        #[test]
        fn option_from_json() {
            let option = serde_json::from_str::<CommandOption>(
                r#"
                    {
                        "name": "color",
                        "alias" : "c",
                        "required": true,
                        "description" : "The color to use",
                        "args" : [
                            {
                                "name" : "color",
                                "valid_values" : ["red", "green", "blue"]
                            }
                        ]
                    }
                "#,
            )
            .unwrap();

            assert_eq!(option.get_name(), "color");
            assert_eq!(
                option.get_aliases().cloned().collect::<Vec<String>>(),
                vec!["c".to_owned()]
            );
            assert_eq!(option.is_required(), true);
            assert_eq!(option.get_description(), Some("The color to use"));

            let arg = option.get_arg().unwrap();
            assert_eq!(arg.get_name(), "color");
            assert_eq!(arg.get_value_count(), &ValueCount::new(1, 1));
            assert_eq!(
                arg.get_valid_values(),
                &["red".to_owned(), "green".to_owned(), "blue".to_owned()]
            );
        }

        #[test]
        fn option_list() {
            let mut option_list = OptionList::new();
            option_list
                .add(CommandOption::new("A").description("generic description"))
                .unwrap();
            option_list
                .add(
                    CommandOption::new("B")
                        .alias("b")
                        .arg(Argument::with_name("value")),
                )
                .unwrap();

            let mut tokens = Vec::new();
            tokens.push(Token::Seq { len: Some(2) });
            tokens.extend(option_tokens(
                "A",
                vec![],
                Some("generic description"),
                false,
                (0, vec![]),
            ));
            tokens.extend(option_tokens(
                "B",
                vec!["b"],
                None,
                false,
                (1, args_tokens(
                    "value",
                    None,
                    Some(1),
                    Some(1),
                    None,
                    vec![],
                    vec![]
                )),
            ));
            tokens.push(Token::SeqEnd);

            serde_test::assert_tokens(&option_list, &tokens);
        }
    }

    #[cfg(test)]
    mod command_tests {
        use crate::serde::tests::{args_tokens, command_tokens, option_tokens};
        use crate::{ValueCount, Argument, Command, CommandOption};

        #[test]
        fn command_test() {
            let command = Command::new("echo")
                .description("Prints a value")
                .usage("echo 1.0")
                .subcommand(Command::new("version").description("Shows the version of the app"))
                .option(
                    CommandOption::new("color")
                        .arg(Argument::with_name("color").valid_values(vec!["red", "green", "blue"])),
                )
                .arg(Argument::with_name("values").value_count(1..));

            let subcommands = command_tokens(
                "version",
                Some("Shows the version of the app"),
                None,
                (0, vec![]),
                (0, vec![]),
                (0, vec![]),
            );

            let options = Vec::from(option_tokens(
                "color",
                vec![],
                None,
                false,
                (
                    1,
                    args_tokens(
                        "color",
                        None,
                        Some(1),
                        Some(1),
                        None,
                        vec!["red", "green", "blue"],
                        vec![]
                    ),
                ),
            ));

            let args = args_tokens(
                "values",
                None,
                Some(1),
                None,
                None,
                vec![],
                vec![]
            );

            serde_test::assert_ser_tokens(
                &command,
                &command_tokens(
                    "echo",
                    Some("Prints a value"),
                    Some("echo 1.0"),
                    (1, subcommands),
                    (1, options),
                    (1, args),
                ),
            );
        }

        #[test]
        fn command_missing_fields_test() {
            let command = Command::new("echo").arg(Argument::with_name("value"));

            let arg = args_tokens(
                "value",
                None,
                Some(1),
                Some(1),
                None,
                vec![],
                vec![]
            );

            serde_test::assert_tokens(
                &command,
                &command_tokens("echo", None, None, (0, vec![]), (0, vec![]), (1, arg)),
            );
        }

        #[test]
        fn command_from_json() {
            let command = serde_json::from_str::<Command>(
                r#"
                {
                    "name": "echo",
                    "description" : "Prints a value",
                    "about" : "echo 1.0",
                    "subcommands" : [
                        {
                            "name" : "version",
                            "description" : "Shows the version of the app"
                        }
                    ],
                    "options" : [
                        {
                            "name" : "color",
                            "args" : [
                                {
                                    "name" : "color",
                                    "valid_values" : ["red", "green", "blue"]
                                }
                            ]
                        }
                    ],
                    "args" : [
                        {
                            "name" : "values",
                            "min_count" : 1
                        }
                    ]
                }
                "#,
            )
            .unwrap();

            assert_eq!(command.get_name(), "echo");
            assert_eq!(command.get_description(), Some("Prints a value"));
            assert_eq!(command.get_usage(), Some("echo 1.0"));

            let subcommand = command.find_subcommand("version").unwrap();
            assert_eq!(subcommand.get_name(), "version");
            assert_eq!(
                subcommand.get_description(),
                Some("Shows the version of the app")
            );

            let option = command.get_options().get("color").unwrap();
            assert_eq!(option.get_name(), "color");

            let option_arg = option.get_arg().unwrap();
            assert_eq!(option_arg.get_name(), "color");
            assert_eq!(
                option_arg.get_valid_values(),
                &["red".to_owned(), "green".to_owned(), "blue".to_owned()]
            );

            let arg = command.get_arg().unwrap();
            assert_eq!(arg.get_name(), "values");
            assert_eq!(arg.get_value_count(), &ValueCount::more_than(1));
        }
    }

    // Utilities
    fn args_tokens(
        name: &'static str,
        description: Option<&'static str>,
        min_count: Option<u64>,
        max_count: Option<u64>,
        valid_type: Option<ValidType>,
        valid_values: Vec<&'static str>,
        default_values: Vec<&'static str>,
    ) -> Vec<Token> {
        let mut tokens = Vec::new();
        tokens.push(Token::Struct {
            name: "Argument",
            len: 7,
        });

        tokens.push(Token::Str("name"));
        tokens.push(Token::String(name));

        tokens.push(Token::Str("description"));
        if let Some(description) = description {
            tokens.push(Token::Some);
            tokens.push(Token::String(description));
        } else {
            tokens.push(Token::None);
        }

        tokens.push(Token::Str("min_count"));
        if let Some(min_count) = min_count {
            tokens.push(Token::Some);
            tokens.push(Token::U64(min_count))
        } else {
            tokens.push(Token::None);
        }

        tokens.push(Token::Str("max_count"));
        if let Some(max_count) = max_count {
            tokens.push(Token::Some);
            tokens.push(Token::U64(max_count))
        } else {
            tokens.push(Token::None);
        }

        tokens.push(Token::Str("type"));
        if let Some(valid_type) = valid_type {
            tokens.push(Token::Some);
            tokens.push(Token::UnitVariant { name: "ValidType", variant: valid_type.as_str() });
        } else {
            tokens.push(Token::None);
        }

        tokens.push(Token::Str("valid_values"));
        tokens.push(Token::Seq {
            len: Some(valid_values.len()),
        });
        for value in valid_values {
            tokens.push(Token::Str(value));
        }
        tokens.push(Token::SeqEnd);

        tokens.push(Token::Str("default_values"));
        tokens.push(Token::Seq {
            len: Some(default_values.len()),
        });
        for value in default_values {
            tokens.push(Token::Str(value));
        }
        tokens.push(Token::SeqEnd);

        tokens.push(Token::StructEnd);
        tokens
    }

    fn option_tokens(
        name: &'static str,
        aliases: Vec<&'static str>,
        description: Option<&'static str>,
        required: bool,
        args: (usize, Vec<Token>),
    ) -> Vec<Token> {
        let mut tokens = Vec::new();
        tokens.push(Token::Struct {
            name: "CommandOption",
            len: 5,
        });

        tokens.push(Token::Str("name"));
        tokens.push(Token::String(name));

        if aliases.len() == 1 {
            tokens.push(Token::Str("alias"));
            tokens.push(Token::String(aliases[0].clone()))
        } else {
            tokens.push(Token::Str("aliases"));
            tokens.push(Token::Seq {
                len: Some(aliases.len()),
            });
            for value in aliases {
                tokens.push(Token::Str(value));
            }
            tokens.push(Token::SeqEnd);
        }

        tokens.push(Token::Str("description"));
        if let Some(description) = description {
            tokens.push(Token::Some);
            tokens.push(Token::String(description));
        } else {
            tokens.push(Token::None);
        }

        tokens.push(Token::Str("required"));
        tokens.push(Token::Bool(required));

        tokens.push(Token::Str("args"));
        tokens.push(Token::Seq { len: Some(args.0) });
        tokens.extend(args.1);
        tokens.push(Token::SeqEnd);

        tokens.push(Token::StructEnd);
        tokens
    }

    fn command_tokens(
        name: &'static str,
        description: Option<&'static str>,
        about: Option<&'static str>,
        subcommands: (usize, Vec<Token>),
        options: (usize, Vec<Token>),
        args: (usize, Vec<Token>),
    ) -> Vec<Token> {
        let mut tokens = Vec::new();
        tokens.push(Token::Struct {
            name: "Command",
            len: 6,
        });

        tokens.push(Token::Str("name"));
        tokens.push(Token::String(name));

        tokens.push(Token::Str("description"));
        if let Some(description) = description {
            tokens.push(Token::Some);
            tokens.push(Token::String(description));
        } else {
            tokens.push(Token::None);
        }

        tokens.push(Token::Str("about"));
        if let Some(about) = about {
            tokens.push(Token::Some);
            tokens.push(Token::String(about));
        } else {
            tokens.push(Token::None);
        }

        tokens.push(Token::Str("subcommands"));
        tokens.push(Token::Seq {
            len: Some(subcommands.0),
        });
        tokens.extend(subcommands.1);
        tokens.push(Token::SeqEnd);

        tokens.push(Token::Str("options"));
        tokens.push(Token::Seq {
            len: Some(options.0),
        });
        tokens.extend(options.1);
        tokens.push(Token::SeqEnd);

        tokens.push(Token::Str("args"));
        tokens.push(Token::Seq { len: Some(args.0) });
        tokens.extend(args.1);
        tokens.push(Token::SeqEnd);

        tokens.push(Token::StructEnd);
        tokens
    }
}
