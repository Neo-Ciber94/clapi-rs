use crate::serde::internal::StringOrList;
use crate::{Argument, ArgumentList, Command, CommandOption, OptionList};
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::export::{Formatter, Result};
use serde::ser::{SerializeSeq, SerializeStruct};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

//  Argument
impl Serialize for Argument {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer {
        #[cfg(feature = "valid_type")]
        use crate::validator::Validator;

        #[cfg(feature = "valid_type")]
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

        let mut state = serializer.serialize_struct("Argument", 8)?;
        state.serialize_field("name", &self.get_name())?;
        state.serialize_field("description", &self.get_description())?;
        state.serialize_field("min_values", &self.get_values_count().min())?;
        state.serialize_field("max_values", &self.get_values_count().max())?;
        #[cfg(feature = "valid_type")]
        {
            state.serialize_field("type", &get_valid_type(self.get_validator()))?;
        }
        state.serialize_field("error", &self.get_validation_error())?;
        state.serialize_field("valid_values", &self.get_valid_values())?;
        state.serialize_field("default_values", &self.get_default_values())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Argument {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>, {
        deserializer.deserialize_struct("Argument", argument::FIELDS, argument::ArgumentVisitor)
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
                    args.add(next_arg).expect("duplicated argument");
                }
                Ok(args)
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, <A as MapAccess<'de>>::Error>
                where A: MapAccess<'de>, {
                let mut args = ArgumentList::new();
                args.add(argument::ArgumentVisitor.visit_map(map)?)
                    .expect("duplicated argument");

                Ok(args)
            }
        }

        deserializer.deserialize_any(ArgumentListVisitor)
    }
}

// CommandOption
impl Serialize for CommandOption {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("CommandOption", 8)?;
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
        state.serialize_field("args", self.get_args())?;
        state.serialize_field("required", &self.is_required())?;
        state.serialize_field("hidden", &self.is_hidden())?;
        state.serialize_field("multiple", &self.allow_multiple())?;
        state.serialize_field("requires_assign", &self.is_assign_required())?;
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
            "args",
            "required",
            "hidden",
            "multiple",
            "requires_assign",
        ];

        enum Field {
            Name,
            Aliases,
            Description,
            Args,
            Required,
            Hidden,
            Multiple,
            RequiresAssign,
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
                            "args" => Ok(Field::Args),
                            "required" => Ok(Field::Required),
                            "hidden" => Ok(Field::Hidden),
                            "multiple" => Ok(Field::Multiple),
                            "requires_assign" => Ok(Field::RequiresAssign),
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
                            b"args" => Ok(Field::Args),
                            b"required" => Ok(Field::Required),
                            b"hidden" => Ok(Field::Hidden),
                            b"multiple" => Ok(Field::Multiple),
                            b"requires_assign" => Ok(Field::RequiresAssign),
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

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, <A as MapAccess<'de>>::Error>
            where
                A: MapAccess<'de>,
            {
                let mut name: Option<String> = None;
                let mut aliases: Option<Vec<String>> = None;
                let mut description: Option<Option<String>> = None;
                let mut args: Option<ArgumentList> = None;
                let mut required: Option<bool> = None;
                let mut hidden : Option<bool> = None;
                let mut multiple : Option<bool> = None;
                let mut requires_assign: Option<bool> = None;

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
                        Field::Args => {
                            if args.is_some() {
                                return Err(de::Error::duplicate_field("args"));
                            }

                            args = Some(map.next_value()?);
                        }
                        Field::Required => {
                            if required.is_some() {
                                return Err(de::Error::duplicate_field("required"));
                            }

                            required = Some(map.next_value()?);
                        }
                        Field::Hidden => {
                            if hidden.is_some() {
                                return Err(de::Error::duplicate_field("hidden"));
                            }

                            hidden = Some(map.next_value()?);
                        }
                        Field::Multiple => {
                            if multiple.is_some() {
                                return Err(de::Error::duplicate_field("multiple"));
                            }

                            multiple = Some(map.next_value()?);
                        }
                        Field::RequiresAssign => {
                            if requires_assign.is_some() {
                                return Err(de::Error::duplicate_field("requires_assign"));
                            }

                            requires_assign = Some(map.next_value()?);
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

                if let Some(args) = args {
                    option = option.args(args);
                }

                if let Some(required) = required {
                    option = option.required(required);
                }

                if let Some(hidden) = hidden {
                    option = option.hidden(hidden);
                }

                if let Some(multiple) = multiple {
                    option = option.multiple(multiple);
                }

                if let Some(requires_assign) = requires_assign {
                    option = option.requires_assign(requires_assign);
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
        let mut state = serializer.serialize_struct("Command", 8)?;
        state.serialize_field("name", self.get_name())?;
        state.serialize_field("description", &self.get_description())?;
        state.serialize_field("usage", &self.get_usage())?;
        state.serialize_field("help", &self.get_help())?;
        state.serialize_field("subcommands", &self.get_subcommands().cloned().collect::<Vec<Command>>())?;
        state.serialize_field("options", &self.get_options())?;
        state.serialize_field("args", &self.get_args())?;
        state.serialize_field("hidden", &self.is_hidden())?;
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
            "usage",
            "help",
            "subcommands",
            "options",
            "args",
            "hidden",
        ];

        enum Field {
            Name,
            Description,
            Usage,
            Help,
            Subcommands,
            Options,
            Args,
            Hidden,
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
                            "usage" => Ok(Field::Usage),
                            "help" => Ok(Field::Help),
                            "subcommands" => Ok(Field::Subcommands),
                            "options" => Ok(Field::Options),
                            "args" => Ok(Field::Args),
                            "hidden" => Ok(Field::Hidden),
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
                            b"usage" => Ok(Field::Usage),
                            b"help" => Ok(Field::Help),
                            b"subcommands" => Ok(Field::Subcommands),
                            b"options" => Ok(Field::Options),
                            b"args" => Ok(Field::Args),
                            b"hidden" => Ok(Field::Hidden),
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

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut name: Option<String> = None;
                let mut description: Option<Option<String>> = None;
                let mut usage: Option<Option<String>> = None;
                let mut help: Option<Option<String>> = None;
                let mut subcommands: Option<Vec<Command>> = None;
                let mut options: Option<OptionList> = None;
                let mut args: Option<ArgumentList> = None;
                let mut hidden : Option<bool> = None;

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
                        Field::Usage => {
                            if usage.is_some() {
                                return Err(de::Error::duplicate_field("usage"));
                            }

                            usage = Some(map.next_value()?);
                        }
                        Field::Help => {
                            if help.is_some() {
                                return Err(de::Error::duplicate_field("help"));
                            }

                            help = Some(map.next_value()?);
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
                        },
                        Field::Hidden => {
                            if hidden.is_some() {
                                return Err(de::Error::duplicate_field("hidden"));
                            }

                            hidden = Some(map.next_value()?);
                        }
                    }
                }

                let mut command =
                    Command::new(name.ok_or_else(|| de::Error::missing_field("name"))?);

                if let Some(Some(description)) = description {
                    command = command.description(description);
                }

                if let Some(Some(usage)) = usage {
                    command = command.usage(usage);
                }

                if let Some(Some(help)) = help {
                    command = command.help(help);
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

                if let Some(hidden) = hidden {
                    command = command.hidden(hidden)
                }

                Ok(command)
            }
        }

        deserializer.deserialize_struct("Command", FIELDS, CommandVisitor)
    }
}

mod internal {
    use serde::de::Visitor;
    use serde::{de, Deserialize, Deserializer};
    use serde::export::{fmt, Formatter};

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
}

#[cfg(feature = "valid_type")]
mod valid_type {
    use crate::args::validator::{Type, validate_type};
    use std::any::TypeId;
    use std::net::{IpAddr, SocketAddr};
    use crate::Argument;
    use std::fmt::Display;

    /// Declares the enum `ValidType` used for serialize the type of an argument.
    macro_rules! declare_impl_valid_type {
        ('primitives: $($ty:ty => $variant:ident),+
            'other: $($ty2:ty => $variant2:ident $name:literal),* $(,)?) => {

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
        'primitives:
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

        'other:
        String => String "string",
        IpAddr => IpAddress "ip_address",
        SocketAddr => SocketAddress "socket_address",
    }
}

mod argument {
    use serde::{Deserialize, Deserializer, de};
    use serde::de::{Visitor, MapAccess};
    use crate::{Argument, ArgCount};
    use crate::serde::internal::AnyToString;
    use std::fmt;
    use std::fmt::Formatter;

    #[cfg(feature = "valid_type")]
    use crate::serde::valid_type::ValidType;

    pub const FIELDS: &'static [&'static str] = &[
        "name",
        "description",
        "min_values",
        "max_values",
        "error",
        "valid_values",
        "default_values",

        #[cfg(feature = "valid_type")]
        "type",
    ];

    pub enum Field {
        Name,
        Description,
        MinCount,
        MaxCount,
        Error,
        ValidValues,
        DefaultValues,

        #[cfg(feature = "valid_type")]
        Type,
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
                    #[cfg(feature = "valid_type")]
                    {
                        formatter.write_str("`name`, `description`, `min_values`, `max_values`, `type`, `valid_values` or `default_values`")
                    }
                    #[cfg(not(feature = "valid_type"))]
                    {
                        formatter.write_str("`name`, `description`, `min_values`, `max_values`, `valid_values` or `default_values`")
                    }
                }

                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                {
                    match v {
                        "name" => Ok(Field::Name),
                        "description" => Ok(Field::Description),
                        "min_values" => Ok(Field::MinCount),
                        "max_values" => Ok(Field::MaxCount),
                        "error" => Ok(Field::Error),
                        "valid_values" => Ok(Field::ValidValues),
                        "default_values" => Ok(Field::DefaultValues),

                        #[cfg(feature = "valid_type")]
                        "type" => Ok(Field::Type),
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
                        b"min_values" => Ok(Field::MinCount),
                        b"max_values" => Ok(Field::MaxCount),
                        b"error" => Ok(Field::Error),
                        b"valid_values" => Ok(Field::ValidValues),
                        b"default_values" => Ok(Field::DefaultValues),

                        #[cfg(feature = "valid_type")]
                        b"type" => Ok(Field::Type),
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

    pub struct ArgumentVisitor;
    impl<'de> Visitor<'de> for ArgumentVisitor {
        type Value = Argument;

        fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
            formatter.write_str("struct Argument")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, <A as MapAccess<'de>>::Error>
            where
                A: MapAccess<'de>,
        {
            let mut name: Option<String> = None;
            let mut description: Option<Option<String>> = None;
            let mut min_values: Option<Option<usize>> = None;
            let mut max_values: Option<Option<usize>> = None;
            let mut validation_error: Option<Option<String>> = None;
            let mut valid_values: Option<Vec<String>> = None;
            let mut default_values: Option<Vec<String>> = None;

            #[cfg(feature = "valid_type")]
            let mut valid_type : Option<Option<ValidType>> = None;

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
                        if min_values.is_some() {
                            return Err(de::Error::duplicate_field("min_values"));
                        }

                        min_values = Some(map.next_value()?);
                    }
                    Field::MaxCount => {
                        if max_values.is_some() {
                            return Err(de::Error::duplicate_field("max_values"));
                        }

                        max_values = Some(map.next_value()?);
                    }
                    #[cfg(feature = "valid_type")]
                    Field::Type => {
                        if valid_type.is_some() {
                            return Err(de::Error::duplicate_field("type"));
                        }

                        valid_type = Some(map.next_value()?);
                    },
                    Field::Error => {
                        if validation_error.is_some() {
                            return Err(de::Error::duplicate_field("error"));
                        }

                        validation_error = Some(map.next_value()?);
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

            let mut argument = match name {
                Some(name) => Argument::with_name(name),
                None => Argument::new()
            };

            if let Some(Some(description)) = description {
                argument = argument.description(description);
            }

            match (min_values.flatten(), max_values.flatten()) {
                (None, None) => { /*By default an `Argument` takes 1 value */ },
                (min, max) => {
                    argument = argument.values_count(ArgCount::new(min, max))
                }
            }

            #[cfg(feature = "valid_type")]
            if let Some(Some(valid_type)) = valid_type {
                argument = valid_type.set_validator(argument);
            }

            if let Some(Some(validation_error)) = validation_error {
                argument = argument.validation_error(validation_error);
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
}

#[cfg(test)]
mod tests {
    #[cfg(test)]
    mod args_tests {
        use crate::{Argument, ArgumentList};
        use serde_test::Token;
        use crate::serde::test_utils::ArgTokens;
        use crate::args::validator::validate_type;

        #[cfg(feature = "valid_type")]
        use {
            crate::args::ty::Type,
            crate::serde::valid_type::ValidType
        };

        #[test]
        fn argument_test() {
            let arg = Argument::with_name("numbers")
                .description("A set of numbers")
                .values_count(1..=10)
                .validator(validate_type::<i64>())
                .validation_error("expected integer")
                .valid_values(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9])
                .defaults(&[1, 2, 3]);

            #[cfg(feature = "valid_type")]
            {
                serde_test::assert_tokens(
                    &arg,
                    ArgTokens::new("numbers")
                        .description("A set of numbers")
                        .min_values(1)
                        .max_values(10)
                        .valid_type(ValidType::I64)
                        .validation_error("expected integer")
                        .valid_values(vec!["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"])
                        .default_values(vec!["1", "2", "3"])
                        .to_tokens()
                        .as_slice(),
                );
            }

            #[cfg(not(feature = "valid_type"))]
            {
                serde_test::assert_tokens(
                    &arg,
                    ArgTokens::new("numbers")
                        .description("A set of numbers")
                        .min_values(1)
                        .max_values(10)
                        .validation_error("expected integer")
                        .valid_values(vec!["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"])
                        .default_values(vec!["1", "2", "3"])
                        .to_tokens()
                        .as_slice(),
                );
            }
        }

        #[test]
        fn argument_missing_fields_test1() {
            let arg = Argument::with_name("numbers");

            serde_test::assert_tokens(
                &arg,
                ArgTokens::new("numbers")
                    .value_count(1)
                    .to_tokens()
                    .as_slice()
            );
        }

        #[test]
        fn argument_missing_fields_test2() {
            let arg = Argument::with_name("numbers")
                .valid_values(&[0, 1, 2, 3])
                .default(0);

            serde_test::assert_tokens(
                &arg,
                ArgTokens::new("numbers")
                    .value_count(1)
                    .valid_values(vec!["0", "1", "2", "3"])
                    .default_values(vec!["0"])
                    .to_tokens()
                    .as_slice()
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
                arg.get_valid_values(),
                &["1".to_owned(), "2".to_owned(), "3".to_owned()]
            );
            #[cfg(feature = "valid_type")]
            assert_eq!(
                arg.get_validator().unwrap().valid_type(),
                Some(Type::of::<i32>())
            );
        }

        #[test]
        fn argument_list_test() {
            let mut args = ArgumentList::new();
            args.add(Argument::with_name("A").description("a")).unwrap();
            args.add(Argument::with_name("B").values_count(2)).unwrap();
            args.add(Argument::with_name("C").valid_values(&['a', 'b', 'c']))
                .unwrap();

            let mut tokens = Vec::new();
            tokens.push(Token::Seq { len: Some(3) });
            tokens.extend(ArgTokens::new("A").value_count(1).description("a").to_tokens());
            tokens.extend(ArgTokens::new("B").min_values(2).max_values(2).to_tokens());
            tokens.extend(ArgTokens::new("C").value_count(1).valid_values(vec!["a", "b", "c"]).to_tokens());
            tokens.push(Token::SeqEnd);

            serde_test::assert_tokens(&args, tokens.as_slice());
        }

        #[test]
        fn argument_list_from_json_test(){
            let json = r#"{
                "description" : "From 2 up to 10 numbers",
                "min_values" : 2,
                "max_values": 10
            }"#;

            let args = serde_json::from_str::<ArgumentList>(json).unwrap();
            assert_eq!(args.len(), 1);

            let arg = &args[0];
            assert_eq!(arg.get_name(), crate::args::ARGUMENT_DEFAULT_NAME);
            assert_eq!(arg.get_description(), Some("From 2 up to 10 numbers"));
            assert_eq!(arg.get_values_count().min(), Some(2));
            assert_eq!(arg.get_values_count().max(), Some(10));
        }
    }

    #[cfg(test)]
    mod options_tests {
        use crate::{ArgCount, Argument, CommandOption, OptionList};
        use serde_test::Token;
        use crate::serde::test_utils::{OptionTokens, ArgTokens};

        #[test]
        fn option_test() {
            let opt = CommandOption::new("time")
                .alias("t")
                .alias("T")
                .description("Number of times")
                .required(false)
                .multiple(true)
                .hidden(false)
                .arg(Argument::with_name("N"))
                .requires_assign(false);

            serde_test::assert_tokens(&opt,
            OptionTokens::new("time")
                .alias("t")
                .alias("T")
                .description("Number of times")
                .required(false)
                .multiple(true)
                .hidden(false)
                .arg(ArgTokens::new("N").value_count(1))
                .requires_assign(false)
                .to_tokens()
                .as_slice());
        }

        #[test]
        fn option_missing_fields_test() {
            let option = CommandOption::new("color")
                .required(true)
                .arg(Argument::with_name("color").valid_values(vec!["red", "blue", "green"]));

            serde_test::assert_tokens(&option,
            OptionTokens::new("color")
                .required(true)
                .arg(ArgTokens::new("color")
                    .value_count(1)
                    .valid_values(vec!["red", "blue", "green"]))
                .to_tokens()
                .as_slice());
        }

        #[test]
        fn option_from_json_test() {
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
            assert_eq!(arg.get_values_count(), ArgCount::one());
            assert_eq!(
                arg.get_valid_values(),
                &["red".to_owned(), "green".to_owned(), "blue".to_owned()]
            );
        }

        #[test]
        fn option_list_test() {
            let mut option_list = OptionList::new();
            option_list
                .add(CommandOption::new("A").description("generic description"))
                .unwrap();
            option_list
                .add(CommandOption::new("B")
                        .alias("b")
                        .arg(Argument::with_name("value")),
                )
                .unwrap();

            let mut tokens = Vec::new();
            tokens.push(Token::Seq { len: Some(2) });
            tokens.extend(OptionTokens::new("A").description("generic description").to_tokens());
            tokens.extend(OptionTokens::new("B")
                .alias("b")
                .arg(ArgTokens::new("value")
                    .value_count(1))
                .to_tokens());
            tokens.push(Token::SeqEnd);

            serde_test::assert_tokens(&option_list, &tokens);
        }
    }

    #[cfg(test)]
    mod command_tests {
        use crate::{ArgCount, Argument, Command, CommandOption};
        use crate::serde::test_utils::{CommandTokens, OptionTokens, ArgTokens};

        #[test]
        fn command_test() {
            let command = Command::new("echo")
                .description("Prints a value")
                .usage("echo [VALUES]...")
                .help("
                echo 1.0
                Prints value to the console
                ")
                .subcommand(Command::new("version").description("Shows the version of the app"))
                .option(
                    CommandOption::new("color")
                        .arg(Argument::with_name("color").valid_values(vec!["red", "green", "blue"])),
                )
                .arg(Argument::with_name("values").values_count(1..))
                .hidden(false);

            serde_test::assert_tokens(
                &command,
                CommandTokens::new("echo")
                    .description("Prints a value")
                    .usage("echo [VALUES]...")
                    .help("
                echo 1.0
                Prints value to the console
                ")
                    .subcommand(CommandTokens::new("version")
                        .description("Shows the version of the app"))
                    .option(OptionTokens::new("color")
                        .arg(ArgTokens::new("color")
                            .value_count(1)
                            .valid_values(vec!["red", "green", "blue"])))
                    .arg(ArgTokens::new("values").min_values(1))
                    .hidden(false)
                    .to_tokens()
                    .as_slice()
            );
        }

        #[test]
        fn command_missing_fields_test() {
            let command = Command::new("echo").arg(Argument::with_name("value"));

            serde_test::assert_tokens(
                &command,
                CommandTokens::new("echo")
                    .arg(ArgTokens::new("value").value_count(1))
                    .to_tokens()
                    .as_slice()
            );
        }

        #[test]
        fn command_from_json_test() {
            let command = serde_json::from_str::<Command>(
                r#"
                {
                    "name": "echo",
                    "description" : "Prints a value",
                    "usage" : "echo [VALUES]...",
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
                            "min_values" : 1
                        }
                    ]
                }
                "#,
            )
            .unwrap();

            assert_eq!(command.get_name(), "echo");
            assert_eq!(command.get_description(), Some("Prints a value"));
            assert_eq!(command.get_usage(), Some("echo [VALUES]..."));

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
            assert_eq!(arg.get_values_count(), ArgCount::more_than(1));
        }
    }
}

#[cfg(test)]
mod test_utils {
    use serde_test::Token;
    use crate::serde::internal::ValidType;

    #[derive(Debug, Clone)]
    pub struct ArgTokens {
        name: &'static str,
        description: Option<&'static str>,
        min_values: Option<u64>,
        max_values: Option<u64>,
        validation_error: Option<&'static str>,
        valid_values: Vec<&'static str>,
        default_values: Vec<&'static str>,

        #[cfg(feature = "valid_type")]
        valid_type: Option<ValidType>,
    }

    impl ArgTokens {
        pub fn new(name: &'static str) -> Self {
            ArgTokens {
                name,
                description: None,
                min_values: None,
                max_values: None,
                validation_error: None,
                valid_values: vec![],
                default_values: vec![],

                #[cfg(feature = "valid_type")]
                valid_type: None,
            }
        }

        pub fn description(mut self, description: &'static str) -> Self {
            self.description = Some(description);
            self
        }

        pub fn min_values(mut self, min_values: u64) -> Self {
            self.min_values = Some(min_values);
            self
        }

        pub fn max_values(mut self, max_values: u64) -> Self {
            self.max_values = Some(max_values);
            self
        }

        pub fn value_count(mut self, value_count: u64) -> Self {
            self.min_values = Some(value_count);
            self.max_values = Some(value_count);
            self
        }

        #[cfg(feature = "valid_type")]
        pub fn valid_type(mut self, valid_type: ValidType) -> Self {
            self.valid_type = Some(valid_type);
            self
        }

        pub fn validation_error(mut self, error: &'static str) -> Self {
            self.validation_error = Some(error);
            self
        }

        pub fn valid_values(mut self, values: Vec<&'static str>) -> Self {
            self.valid_values = values;
            self
        }

        pub fn default_values(mut self, values: Vec<&'static str>) -> Self {
            self.default_values = values;
            self
        }

        pub fn to_tokens(&self) -> Vec<Token> {
            let mut tokens = Vec::new();
            tokens.push(Token::Struct {
                name: "Argument",
                len: 8,
            });

            // Argument name
            tokens.push(Token::Str("name"));
            tokens.push(Token::String(self.name));

            // Argument description
            tokens.push(Token::Str("description"));
            if let Some(description) = self.description {
                tokens.push(Token::Some);
                tokens.push(Token::String(description));
            } else {
                tokens.push(Token::None);
            }

            // Argument min values
            tokens.push(Token::Str("min_values"));
            if let Some(min_values) = self.min_values {
                tokens.push(Token::Some);
                tokens.push(Token::U64(min_values))
            } else {
                tokens.push(Token::None);
            }

            // Argument max values
            tokens.push(Token::Str("max_values"));
            if let Some(max_values) = self.max_values {
                tokens.push(Token::Some);
                tokens.push(Token::U64(max_values))
            } else {
                tokens.push(Token::None);
            }

            // Argument valid type
            #[cfg(feature = "valid_type")]
            {
                tokens.push(Token::Str("type"));
                if let Some(valid_type) = &self.valid_type {
                    tokens.push(Token::Some);
                    tokens.push(Token::UnitVariant { name: "ValidType", variant: valid_type.as_str() });
                } else {
                    tokens.push(Token::None);
                }
            }

            // Argument validation error
            tokens.push(Token::Str("error"));
            if let Some(validation_error) = self.validation_error {
                tokens.push(Token::Some);
                tokens.push(Token::String(validation_error));
            } else {
                tokens.push(Token::None);
            }

            // Argument valid values
            tokens.push(Token::Str("valid_values"));
            tokens.push(Token::Seq {
                len: Some(self.valid_values.len()),
            });
            for value in &self.valid_values {
                tokens.push(Token::Str(value));
            }
            tokens.push(Token::SeqEnd);

            // Argument default values
            tokens.push(Token::Str("default_values"));
            tokens.push(Token::Seq {
                len: Some(self.default_values.len()),
            });
            for value in &self.default_values {
                tokens.push(Token::Str(value));
            }
            tokens.push(Token::SeqEnd);

            // End
            tokens.push(Token::StructEnd);
            tokens
        }
    }

    #[derive(Debug, Clone)]
    pub struct OptionTokens {
        name: &'static str,
        aliases: Vec<&'static str>,
        description: Option<&'static str>,
        args: Vec<ArgTokens>,
        required: bool,
        hidden: bool,
        multiple: bool,
        requires_assign: bool,
    }

    impl OptionTokens {
        pub fn new(name: &'static str) -> Self {
            OptionTokens {
                name,
                aliases: vec![],
                description: None,
                args: vec![],
                required: false,
                hidden: false,
                multiple: false,
                requires_assign: false,
            }
        }

        pub fn alias(mut self, alias: &'static str) -> Self {
            self.aliases.push(alias);
            self
        }

        pub fn description(mut self, description: &'static str) -> Self {
            self.description = Some(description);
            self
        }

        pub fn arg(mut self, arg: ArgTokens) -> Self {
            self.args.push(arg);
            self
        }

        pub fn required(mut self, required: bool) -> Self {
            self.required = required;
            self
        }

        pub fn hidden(mut self, hidden: bool) -> Self {
            self.hidden = hidden;
            self
        }

        pub fn multiple(mut self, multiple: bool) -> Self {
            self.multiple = multiple;
            self
        }

        pub fn requires_assign(mut self, requires_assign: bool) -> Self {
            self.requires_assign = requires_assign;
            self
        }

        pub fn to_tokens(&self) -> Vec<Token> {
            let mut tokens = Vec::new();
            tokens.push(Token::Struct {
                name: "CommandOption",
                len: 8,
            });

            // Option name
            tokens.push(Token::Str("name"));
            tokens.push(Token::String(self.name));

            // Option aliases
            if self.aliases.len() == 1 {
                tokens.push(Token::Str("alias"));
                tokens.push(Token::String(self.aliases[0]))
            } else {
                tokens.push(Token::Str("aliases"));
                tokens.push(Token::Seq {
                    len: Some(self.aliases.len()),
                });
                for alias in &self.aliases {
                    tokens.push(Token::Str(alias));
                }
                tokens.push(Token::SeqEnd);
            }

            // Option description
            tokens.push(Token::Str("description"));
            if let Some(description) = self.description {
                tokens.push(Token::Some);
                tokens.push(Token::String(description));
            } else {
                tokens.push(Token::None);
            }

            // Option arguments
            tokens.push(Token::Str("args"));
            tokens.push(Token::Seq {
                len: Some(self.args.len()),
            });
            for arg in &self.args {
                tokens.extend(arg.to_tokens())
            }
            tokens.push(Token::SeqEnd);

            // Option required
            tokens.push(Token::Str("required"));
            tokens.push(Token::Bool(self.required));

            // Option hidden
            tokens.push(Token::Str("hidden"));
            tokens.push(Token::Bool(self.hidden));

            // Option multiple
            tokens.push(Token::Str("multiple"));
            tokens.push(Token::Bool(self.multiple));

            // Option assign required
            tokens.push(Token::Str("requires_assign"));
            tokens.push(Token::Bool(self.requires_assign));

            // End
            tokens.push(Token::StructEnd);
            tokens
        }
    }

    #[derive(Debug, Clone)]
    pub struct CommandTokens {
        name: &'static str,
        description: Option<&'static str>,
        usage: Option<&'static str>,
        help: Option<&'static str>,
        subcommands: Vec<CommandTokens>,
        options: Vec<OptionTokens>,
        args: Vec<ArgTokens>,
        hidden: bool,
    }

    impl CommandTokens {
        pub fn new(name: &'static str) -> Self {
            CommandTokens {
                name,
                description: None,
                usage: None,
                help: None,
                subcommands: vec![],
                options: vec![],
                args: vec![],
                hidden: false,
            }
        }

        pub fn description(mut self, description: &'static str) -> Self {
            self.description = Some(description);
            self
        }

        pub fn usage(mut self, usage: &'static str) -> Self {
            self.usage = Some(usage);
            self
        }

        pub fn help(mut self, help: &'static str) -> Self {
            self.help = Some(help);
            self
        }

        pub fn subcommand(mut self, subcommand: CommandTokens) -> Self {
            self.subcommands.push(subcommand);
            self
        }

        pub fn option(mut self, option: OptionTokens) -> Self {
            self.options.push(option);
            self
        }

        pub fn arg(mut self, arg: ArgTokens) -> Self {
            self.args.push(arg);
            self
        }

        pub fn hidden(mut self, hidden: bool) -> Self {
            self.hidden = hidden;
            self
        }

        pub fn to_tokens(&self) -> Vec<Token> {
            let mut tokens = Vec::new();
            tokens.push(Token::Struct {
                name: "Command",
                len: 8,
            });

            // Command name
            tokens.push(Token::Str("name"));
            tokens.push(Token::String(self.name));

            // Command description
            tokens.push(Token::Str("description"));
            if let Some(description) = self.description {
                tokens.push(Token::Some);
                tokens.push(Token::String(description));
            } else {
                tokens.push(Token::None);
            }

            // Command usage
            tokens.push(Token::Str("usage"));
            if let Some(usage) = self.usage {
                tokens.push(Token::Some);
                tokens.push(Token::String(usage));
            } else {
                tokens.push(Token::None);
            }

            // Command help
            tokens.push(Token::Str("help"));
            if let Some(help) = self.help {
                tokens.push(Token::Some);
                tokens.push(Token::String(help));
            } else {
                tokens.push(Token::None);
            }

            // Command children
            tokens.push(Token::Str("subcommands"));
            tokens.push(Token::Seq {
                len: Some(self.subcommands.len()),
            });

            for subcommand in &self.subcommands {
                tokens.extend(subcommand.to_tokens());
            }
            tokens.push(Token::SeqEnd);

            // Command options
            tokens.push(Token::Str("options"));
            tokens.push(Token::Seq {
                len: Some(self.options.len()),
            });
            for option in &self.options {
                tokens.extend(option.to_tokens());
            }
            tokens.push(Token::SeqEnd);

            // Command args
            tokens.push(Token::Str("args"));
            tokens.push(Token::Seq { len: Some(self.args.len()) });
            for arg in &self.args {
                tokens.extend(arg.to_tokens());
            }
            tokens.push(Token::SeqEnd);

            // Command hidden
            tokens.push(Token::Str("hidden"));
            tokens.push(Token::Bool(self.hidden));

            // End
            tokens.push(Token::StructEnd);
            tokens
        }
    }
}