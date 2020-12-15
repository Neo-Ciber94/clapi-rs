use crate::{ArgCount, Argument, ArgumentList, Command, CommandOption, OptionList};
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::export::{Formatter, Result};
use serde::ser::{SerializeSeq, SerializeStruct};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use crate::serde::utils::AnyToString;

// ArgCount
impl Serialize for ArgCount {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ArgCount", 2)?;
        state.serialize_field("min_count", &self.min())?;
        state.serialize_field("max_count", &self.max())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for ArgCount {
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
            type Value = ArgCount;

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
                Ok(ArgCount::new(min, max))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut min = None;
                let mut max = None;

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

                let min = min.ok_or_else(|| de::Error::missing_field("min_count"))?;
                let max = max.ok_or_else(|| de::Error::missing_field("max_count"))?;
                Ok(ArgCount::new(min, max))
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
        let mut state = serializer.serialize_struct("Argument", 6)?;
        state.serialize_field("name", &self.get_name())?;
        state.serialize_field("description", &self.get_description())?;
        state.serialize_field("min_count", &self.get_arg_count().min())?;
        state.serialize_field("max_count", &self.get_arg_count().max())?;
        //state.skip_field("validator")?;
        state.serialize_field("valid_values", &self.get_valid_values())?;
        state.serialize_field("default_values", &self.get_default_values())?;
        //state.skip_field("values")?;
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
            "valid_values",
            "default_values",
        ];

        enum Field {
            Name,
            Description,
            MinCount,
            MaxCount,
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
                        formatter.write_str("`name`, `description`, `min_count`, `max_count`, `valid_values` or `default_values`")
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

                let min_count: usize = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;

                let max_count: usize = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;

                let valid_values: Vec<String> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(4, &self))?;

                let default_values: Vec<String> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(5, &self))?;

                let mut argument = Argument::new(name).arg_count(ArgCount::new(min_count, max_count));

                if let Some(description) = description {
                    argument = argument.description(description);
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
                let mut min_count: Option<usize> = None;
                let mut max_count: Option<usize> = None;
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

                let mut argument = Argument::new(name.ok_or_else(|| de::Error::missing_field("name"))?);

                if let Some(Some(description)) = description {
                    argument = argument.description(description);
                }

                match (min_count, max_count) {
                    (Some(min), Some(max)) => {
                        argument = argument.arg_count(ArgCount::new(min, max));
                    }
                    (Some(min), None) => {
                        argument = argument.arg_count(ArgCount::more_than(min));
                    }
                    (None, Some(max)) => {
                        argument = argument.arg_count(ArgCount::less_than(max));
                    }
                    (None, None) => {}
                };

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
        state.serialize_field(
            "aliases",
            &self.get_aliases().cloned().collect::<Vec<String>>(),
        )?;
        state.serialize_field("description", &self.get_description())?;
        state.serialize_field("is_required", &self.is_required())?;
        state.serialize_field("args", self.get_args())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for CommandOption {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        const FIELDS: &'static [&'static str] =
            &["name", "aliases", "description", "is_required", "args"];

        enum Field {
            Name,
            Aliases,
            Description,
            IsRequired,
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
                        formatter
                            .write_str("`name`, `aliases`, `description`, `is_required` or `args`")
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match v {
                            "name" => Ok(Field::Name),
                            "aliases" => Ok(Field::Aliases),
                            "description" => Ok(Field::Description),
                            "is_required" => Ok(Field::IsRequired),
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
                            b"aliases" => Ok(Field::Aliases),
                            b"description" => Ok(Field::Description),
                            b"is_required" => Ok(Field::IsRequired),
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
                    .next_element()?
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
                let mut is_required: Option<bool> = None;
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
                                return Err(de::Error::duplicate_field("aliases"));
                            }

                            aliases = Some(map.next_value()?);
                        }
                        Field::Description => {
                            if description.is_some() {
                                return Err(de::Error::duplicate_field("description"));
                            }

                            description = Some(map.next_value()?);
                        }
                        Field::IsRequired => {
                            if is_required.is_some() {
                                return Err(de::Error::duplicate_field("is_required"));
                            }

                            is_required = Some(map.next_value()?);
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
                        option = option.alias(alias);
                    }
                }

                if let Some(Some(description)) = description {
                    option = option.description(description);
                }

                if let Some(is_required) = is_required {
                    option = option.required(is_required);
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
        state.serialize_field("about", &self.get_about())?;
        state.serialize_field("subcommands", &self.get_children().cloned().collect::<Vec<Command>>())?;
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
            Name, Description, About, Subcommands, Options, Args
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where
                D: Deserializer<'de> {
                struct FieldVisitor;
                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                        formatter.write_str("`name`, `description`, `about`, `subcommands`, `options` or `args`")
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where
                        E: de::Error, {
                        match v {
                            "name" => Ok(Field::Name),
                            "description" => Ok(Field::Description),
                            "about" => Ok(Field::About),
                            "subcommands" => Ok(Field::Subcommands),
                            "options" => Ok(Field::Options),
                            "args" => Ok(Field::Args),
                            _ => return Err(de::Error::unknown_field(v, FIELDS))
                        }
                    }

                    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E> where
                        E: de::Error, {
                        match v {
                            b"name" => Ok(Field::Name),
                            b"description" => Ok(Field::Description),
                            b"about" => Ok(Field::About),
                            b"subcommands" => Ok(Field::Subcommands),
                            b"options" => Ok(Field::Options),
                            b"args" => Ok(Field::Args),
                            _ => {
                                let value = String::from_utf8_lossy(v);
                                return Err(de::Error::unknown_field(&value, FIELDS))
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

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where
                A: SeqAccess<'de>, {
                let name : String = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;

                let description : Option<String> = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                let about : Option<String> = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;

                let subcommands : Vec<Command> = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;

                let options : OptionList = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(4, &self))?;

                let args : ArgumentList = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(5, &self))?;

                let mut command = Command::new(name)
                    .options(options)
                    .args(args);

                for subcommand in subcommands {
                    command = command.subcommand(subcommand);
                }

                if let Some(description) = description {
                    command = command.description(description);
                }

                if let Some(about) = about {
                    command = command.about(about);
                }

                Ok(command)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error> where
                A: MapAccess<'de>, {

                let mut name : Option<String> = None;
                let mut description : Option<Option<String>> = None;
                let mut about : Option<Option<String>> = None;
                let mut subcommands : Option<Vec<Command>> = None;
                let mut options : Option<OptionList> = None;
                let mut args : Option<ArgumentList> = None;

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

                let mut command = Command::new(
                    name.ok_or_else(|| de::Error::missing_field("name"))?
                );

                if let Some(Some(description)) = description {
                    command = command.description(description);
                }

                if let Some(Some(about)) = about {
                    command = command.about(about);
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

mod utils {
    use serde::de::Visitor;
    use serde::export::{fmt, Formatter};
    use serde::{de, Deserialize, Deserializer};

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
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct AnyToString(pub String);

    impl<'de> Deserialize<'de> for AnyToString {
        fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
        where
            D: Deserializer<'de>,
        {
            struct StringWrapperVisitor;
            impl<'de> Visitor<'de> for StringWrapperVisitor {
                type Value = AnyToString;

                fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                    formatter.write_str("`bool`, `char`, `integer`, `float` or `string`")
                }

                fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    Ok(AnyToString(value))
                }

                visit_to_string! {
                    visit_bool bool, visit_char char, visit_str &str, visit_borrowed_str &'de str,
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

            deserializer.deserialize_any(StringWrapperVisitor)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::Token;

    #[test]
    fn arg_count_test() {
        let arg_count = ArgCount::new(2, 10);
        serde_test::assert_tokens(
            &arg_count,
            &[
                Token::Struct {
                    name: "ArgCount",
                    len: 2,
                },
                Token::Str("min_count"),
                Token::U64(2),
                Token::Str("max_count"),
                Token::U64(10),
                Token::StructEnd,
            ],
        )
    }

    #[test]
    fn argument_test() {
        let arg1 = Argument::new("numbers")
            .description("A set of numbers")
            .arg_count(1..=10)
            .valid_values(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9])
            .defaults(&[1, 2, 3]);

        serde_test::assert_tokens(
            &arg1,
            &[
                Token::Struct {
                    name: "Argument",
                    len: 6,
                },
                Token::Str("name"),
                Token::String("numbers"),
                Token::Str("description"),
                Token::Some,
                Token::String("A set of numbers"),
                Token::Str("min_count"),
                Token::U64(1),
                Token::Str("max_count"),
                Token::U64(10),
                Token::Str("valid_values"),
                Token::Seq { len: Some(10) },
                Token::Str("0"),
                Token::Str("1"),
                Token::Str("2"),
                Token::Str("3"),
                Token::Str("4"),
                Token::Str("5"),
                Token::Str("6"),
                Token::Str("7"),
                Token::Str("8"),
                Token::Str("9"),
                Token::SeqEnd,
                Token::Str("default_values"),
                Token::Seq { len: Some(3) },
                Token::Str("1"),
                Token::Str("2"),
                Token::Str("3"),
                Token::SeqEnd,
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn argument_missing_fields_test1() {
        let arg1 = Argument::new("numbers");

        serde_test::assert_tokens(
            &arg1,
            &[
                Token::Struct {
                    name: "Argument",
                    len: 6,
                },
                Token::Str("name"),
                Token::String("numbers"),
                Token::Str("description"),
                Token::None,
                Token::Str("min_count"),
                Token::U64(1),
                Token::Str("max_count"),
                Token::U64(1),
                Token::Str("valid_values"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("default_values"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn argument_missing_fields_test2() {
        let arg1 = Argument::new("numbers")
            .valid_values(&[0, 1, 2, 3])
            .default(0);

        serde_test::assert_tokens(
            &arg1,
            &[
                Token::Struct {
                    name: "Argument",
                    len: 6,
                },
                Token::Str("name"),
                Token::String("numbers"),
                Token::Str("description"),
                Token::None,
                Token::Str("min_count"),
                Token::U64(1),
                Token::Str("max_count"),
                Token::U64(1),
                Token::Str("valid_values"),
                Token::Seq { len: Some(4) },
                Token::Str("0"),
                Token::Str("1"),
                Token::Str("2"),
                Token::Str("3"),
                Token::SeqEnd,
                Token::Str("default_values"),
                Token::Seq { len: Some(1) },
                Token::Str("0"),
                Token::SeqEnd,
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn argument_from_json_test() {
        let arg = serde_json::from_str::<Argument>(
            r#"
        {
            "name": "numbers",
            "valid_values": [1,2,3]
        }
        "#,
        )
        .unwrap();

        assert_eq!(arg.get_name(), "numbers");
        assert_eq!(
            arg.get_valid_values(),
            &["1".to_owned(), "2".to_owned(), "3".to_owned()]
        );
    }
}