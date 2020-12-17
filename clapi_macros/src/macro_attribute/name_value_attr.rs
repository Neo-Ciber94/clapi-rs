use std::fmt::{Debug, Display, Formatter, Write};
use std::ops::Index;
use std::str::FromStr;

use syn::{AttrStyle, Attribute, AttributeArgs, Lit};

use crate::macro_attribute::{
    display_lit, lit_to_string, meta_item_to_string, MacroAttribute, MetaItem,
};
use std::convert::TryFrom;

type Map<K, V> = linked_hash_map::LinkedHashMap<K, V>;
type Iter<'a, K, V> = linked_hash_map::Iter<'a, K, V>;
type IntoIter<K, V> = linked_hash_map::IntoIter<K, V>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NameValueAttribute {
    path: String,
    args: Map<String, Value>,
    style: AttrStyle,
}

impl NameValueAttribute {
    pub fn empty(path: String, style: AttrStyle) -> Self {
        NameValueAttribute {
            path,
            args: Default::default(),
            style,
        }
    }

    pub fn new(
        path: &str,
        meta_items: Vec<MetaItem>,
        style: AttrStyle,
    ) -> Result<Self, NameValueError> {
        let mut args = Map::new();

        for meta_item in meta_items.into_iter() {
            let name_value = meta_item
                .as_name_value()
                .cloned()
                .ok_or_else(|| NameValueError::InvalidValue(meta_item.clone()))?;

            if args.contains_key(&name_value.name) {
                return Err(NameValueError::DuplicatedKey(name_value.name));
            } else {
                args.insert(name_value.name, name_value.value);
            }
        }

        let mut name_value_attribute = NameValueAttribute::empty(path.to_string(), style);
        name_value_attribute.args = args;
        Ok(name_value_attribute)
    }

    pub fn from_attribute_args(
        path: &str,
        attribute_args: AttributeArgs,
        style: AttrStyle,
    ) -> Result<Self, NameValueError> {
        MacroAttribute::from_attribute_args(path, attribute_args, style).into_name_values()
    }

    pub fn path(&self) -> &str {
        self.path.as_str()
    }

    pub fn style(&self) -> &AttrStyle {
        &self.style
    }

    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.args.keys()
    }

    pub fn len(&self) -> usize {
        self.args.len()
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.args.get(name)
    }

    pub fn contains_name(&self, name: &str) -> bool {
        self.args.contains_key(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.args.iter()
    }

    pub fn into_inner(self) -> Map<String, Value> {
        self.args
    }
}

impl<'a> Index<&'a str> for NameValueAttribute {
    type Output = Value;

    fn index(&self, index: &'a str) -> &Self::Output {
        &self.args[index]
    }
}

impl<'a> IntoIterator for &'a NameValueAttribute {
    type Item = (&'a String, &'a Value);
    type IntoIter = Iter<'a, String, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.args.iter()
    }
}

impl IntoIterator for NameValueAttribute {
    type Item = (String, Value);
    type IntoIter = IntoIter<String, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.args.into_iter()
    }
}

impl TryFrom<Attribute> for NameValueAttribute {
    type Error = NameValueError;

    fn try_from(value: Attribute) -> Result<Self, Self::Error> {
        MacroAttribute::new(value).into_name_values()
    }
}

impl Display for NameValueAttribute {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let style = if matches!(self.style, AttrStyle::Outer) {
            "#".to_owned()
        } else {
            "#!".to_owned()
        };

        let meta = self
            .args
            .values()
            .map(|value| {
                let mut s = String::new();
                value.display(&mut s, false).unwrap();
                s
            })
            .collect::<Vec<String>>();

        if meta.is_empty() {
            write!(f, "{}[{}]", style, self.path())
        } else {
            write!(f, "{}[{}({})]", style, self.path(), meta.join(", "))
        }
    }
}

pub enum NameValueError {
    InvalidValue(MetaItem),
    DuplicatedKey(String),
}

impl Debug for NameValueError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NameValueError::InvalidValue(x) => {
                write!(f, "`{}` is not a name-value", meta_item_to_string(&x))
            }
            NameValueError::DuplicatedKey(x) => write!(f, "duplicated key: `{}`", x),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NameValue {
    pub name: String,
    pub value: Value,
}

impl Display for NameValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", self.name, self.value)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Value {
    Literal(Lit),
    Array(Vec<Lit>),
}

impl Value {
    pub fn is_literal(&self) -> bool {
        matches!(self, Value::Literal(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    pub fn is_string(&self) -> bool {
        match self {
            Value::Literal(lit) => match lit {
                Lit::Str(_) | Lit::ByteStr(_) => true,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn is_char(&self) -> bool {
        match self {
            Value::Literal(lit) => match lit {
                Lit::Char(_) => true,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn is_bool(&self) -> bool {
        match self {
            Value::Literal(lit) => match lit {
                Lit::Bool(_) => true,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn is_integer(&self) -> bool {
        match self {
            Value::Literal(lit) => match lit {
                Lit::Int(_) | Lit::Byte(_) => true,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn is_float(&self) -> bool {
        match self {
            Value::Literal(lit) => match lit {
                Lit::Float(_) => true,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn is_numeric(&self) -> bool {
        self.is_integer() || self.is_float()
    }

    pub fn to_string_literal(&self) -> Option<String> {
        if let Value::Literal(lit) = self {
            return match lit {
                Lit::Str(x) => Some(x.value()),
                Lit::ByteStr(x) => unsafe { Some(String::from_utf8_unchecked(x.value())) },
                _ => None,
            };
        }

        None
    }

    pub fn to_char_literal(&self) -> Option<char> {
        if let Value::Literal(lit) = self {
            return match lit {
                Lit::Char(x) => Some(x.value()),
                _ => None,
            };
        }

        None
    }

    pub fn to_bool_literal(&self) -> Option<bool> {
        if let Value::Literal(lit) = self {
            return match lit {
                Lit::Bool(x) => Some(x.value),
                _ => None,
            };
        }

        None
    }

    pub fn to_byte_literal(&self) -> Option<u8> {
        if let Value::Literal(lit) = self {
            return match lit {
                Lit::Byte(x) => Some(x.value()),
                _ => None,
            };
        }

        None
    }

    pub fn to_integer_literal<N>(&self) -> Option<N>
    where
        N: FromStr,
        N::Err: Display,
    {
        match self {
            Value::Literal(lit) => match lit {
                Lit::Byte(n) => {
                    let s = n.value().to_string();
                    N::from_str(s.as_str()).ok()
                }
                Lit::Int(n) => n.base10_parse().ok(),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn to_float_literal<N>(&self) -> Option<N>
    where
        N: FromStr,
        N::Err: Display,
    {
        match self {
            Value::Literal(lit) => match lit {
                Lit::Float(n) => n.base10_parse().ok(),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn as_literal(&self) -> Option<&Lit> {
        match self {
            Value::Literal(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Lit>> {
        match self {
            Value::Array(x) => Some(x),
            _ => None,
        }
    }

    pub fn parse_literal<T: FromStr>(&self) -> Option<T> {
        match self {
            Value::Literal(x) => {
                let value = lit_to_string(x);
                T::from_str(&value).ok()
            }
            _ => None,
        }
    }

    pub fn parse_array<T: FromStr>(&self) -> Option<Vec<T>> {
        match self {
            Value::Array(array) => {
                let mut ret = Vec::new();
                for arg in array {
                    let value = lit_to_string(arg);
                    let n = T::from_str(&value).ok()?;
                    ret.push(n);
                }
                Some(ret)
            }
            _ => None,
        }
    }

    pub fn display<W: Write>(
        &self,
        formatter: &mut W,
        use_array_brackets: bool,
    ) -> std::fmt::Result {
        match self {
            Value::Literal(lit) => display_lit(formatter, lit),
            Value::Array(array) => {
                let result = array
                    .iter()
                    .map(|s| lit_to_string(s))
                    .collect::<Vec<String>>();

                if use_array_brackets {
                    write!(formatter, "[{}]", result.join(", "))
                } else {
                    write!(formatter, "{}", result.join(", "))
                }
            }
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.display(f, true)
    }
}

#[cfg(test)]
mod tests {
    use crate::macro_attribute::MacroAttribute;
    use proc_macro2::TokenStream;
    use quote::*;
    use syn::parse_quote::ParseQuote;
    use syn::Attribute;

    fn parse_attr(tokens: TokenStream) -> Attribute {
        syn::parse::Parser::parse2(Attribute::parse, tokens).expect("invalid attribute")
    }

    #[test]
    fn into_name_value_test() {
        let tokens = quote! { #[person(name="Kaori", age=20)] };
        let raw_attr = MacroAttribute::new(parse_attr(tokens));
        assert!(raw_attr.into_name_values().is_ok());
    }

    #[test]
    fn into_name_value_error_test() {
        let tokens = quote! { #[person(name="Kaori", age=20, job(salary=200.0))] };
        let raw_attr = MacroAttribute::new(parse_attr(tokens));
        assert!(raw_attr.into_name_values().is_err());
    }

    #[test]
    fn into_name_value_duplicate_name_test() {
        let tokens = quote! { #[person(name="Kaori", age=20, age=21)] };
        let raw_attr = MacroAttribute::new(parse_attr(tokens));
        assert!(raw_attr.into_name_values().is_err());
    }

    #[test]
    fn new_name_value_attr_test() {
        let tokens = quote! { #[person(name="Kaori", age=20, fav_numbers=2,4,7)] };
        let raw_attr = MacroAttribute::new(parse_attr(tokens));
        let attr = raw_attr.into_name_values().unwrap();

        assert_eq!(attr.path, "person".to_owned());
        assert_eq!(attr.len(), 3);

        assert!(attr["name"].is_string());
        assert!(attr["age"].is_integer());
        assert!(attr["fav_numbers"].is_array());

        assert_eq!(attr["name"].to_string_literal(), Some("Kaori".to_owned()));
        assert_eq!(attr["age"].parse_literal::<u32>(), Some(20));
        assert_eq!(attr["fav_numbers"].parse_array(), Some(vec!(2, 4, 7)));
    }

    #[test]
    fn contains_name_test() {
        let tokens = quote! { #[person(name="Kaori", age=20, fav_numbers=2,4,7)] };
        let raw_attr = MacroAttribute::new(parse_attr(tokens));
        let attr = raw_attr.into_name_values().unwrap();

        assert!(attr.contains_name("name"));
        assert!(attr.contains_name("age"));
        assert!(attr.contains_name("fav_numbers"));
    }

    #[test]
    fn get_test() {
        let tokens = quote! { #[person(name="Kaori", age=20, fav_numbers=2,4,7)] };
        let raw_attr = MacroAttribute::new(parse_attr(tokens));
        let attr = raw_attr.into_name_values().unwrap();

        assert_eq!(
            attr.get("name").unwrap().to_string_literal(),
            Some("Kaori".to_owned())
        );
        assert_eq!(attr.get("age").unwrap().parse_literal::<u32>(), Some(20));
        assert_eq!(
            attr.get("fav_numbers").unwrap().parse_array(),
            Some(vec!(2, 4, 7))
        );
    }

    #[test]
    fn value_variant_check_test() {
        let tokens = quote! {
            #[values(
                str="hello",
                bytestr=b"world",
                byte=b'a',
                number=20,
                float=0.5,
                boolean=true,
                character='z',
                array=1,2,3
            )]
        };

        let raw_attr = MacroAttribute::new(parse_attr(tokens));
        let attr = raw_attr.into_name_values().unwrap();

        assert!(attr["str"].is_literal());
        assert!(attr["bytestr"].is_literal());
        assert!(attr["byte"].is_literal());
        assert!(attr["number"].is_literal());
        assert!(attr["float"].is_literal());
        assert!(attr["boolean"].is_literal());
        assert!(attr["character"].is_literal());
        assert!(attr["array"].is_array());
    }

    #[test]
    fn value_as_type_test() {
        let tokens = quote! {
            #[values(
                str="hello",
                bytestr=b"world",
                byte=b'a',
                number=20,
                float=0.5,
                boolean=true,
                character='z',
                array=1,2,3
            )]
        };

        let raw_attr = MacroAttribute::new(parse_attr(tokens));
        let attr = raw_attr.into_name_values().unwrap();

        assert!(attr["str"].as_literal().is_some());
        assert!(attr["array"].as_array().is_some());

        assert_eq!(attr["str"].to_string_literal(), Some("hello".to_string()));
        assert_eq!(
            attr["bytestr"].to_string_literal(),
            Some("world".to_string())
        );
        assert_eq!(attr["byte"].to_byte_literal(), Some(b'a'));
        assert_eq!(attr["boolean"].to_bool_literal(), Some(true));
        assert_eq!(attr["character"].to_char_literal(), Some('z'));
    }

    #[test]
    fn value_parse_test() {
        let tokens = quote! {
            #[values(
                str="hello",
                bytestr=b"world",
                byte=b'a',
                number=20,
                float=0.5,
                boolean=true,
                character='z',
                array=1,2,3
            )]
        };

        let raw_attr = MacroAttribute::new(parse_attr(tokens));
        let attr = raw_attr.into_name_values().unwrap();

        assert_eq!(
            attr["str"].parse_literal::<String>(),
            Some("hello".to_string())
        );
        assert_eq!(
            attr["bytestr"].parse_literal::<String>(),
            Some("world".to_string())
        );
        assert_eq!(attr["byte"].parse_literal::<u8>(), Some(b'a'));
        assert_eq!(attr["number"].parse_literal::<u32>(), Some(20));
        assert_eq!(attr["float"].parse_literal::<f64>(), Some(0.5));
        assert_eq!(attr["boolean"].parse_literal::<bool>(), Some(true));
        assert_eq!(attr["character"].parse_literal::<char>(), Some('z'));
        assert_eq!(attr["array"].parse_array::<usize>(), Some(vec!(1, 2, 3)));
    }

    #[test]
    fn to_type_test() {
        let tokens = quote! {
            #[values(
                str="hello",
                bytestr=b"world",
                byte=b'a',
                number=20,
                float=0.5,
                boolean=true,
                character='z',
                array=1,2,3
            )]
        };

        let raw_attr = MacroAttribute::new(parse_attr(tokens));
        let attr = raw_attr.into_name_values().unwrap();

        assert_eq!(attr["str"].to_string_literal(), Some("hello".to_string()));
        assert_eq!(
            attr["bytestr"].to_string_literal(),
            Some("world".to_string())
        );
        assert_eq!(attr["byte"].to_byte_literal(), Some(b'a'));
        assert_eq!(attr["number"].to_integer_literal::<u32>(), Some(20));
        assert_eq!(attr["byte"].to_integer_literal::<u32>(), Some(b'a' as u32));
        assert_eq!(attr["float"].to_float_literal::<f64>(), Some(0.5));
        assert_eq!(attr["boolean"].to_bool_literal(), Some(true));
        assert_eq!(attr["character"].to_char_literal(), Some('z'));

        assert_eq!(attr["str"].to_char_literal(), None);
        assert_eq!(attr["bytestr"].to_char_literal(), None);
        assert_eq!(attr["byte"].to_string_literal(), None);
        assert_eq!(attr["number"].to_char_literal(), None);
        assert_eq!(attr["byte"].to_bool_literal(), None);
        assert_eq!(attr["float"].to_integer_literal::<u32>(), None);
        assert_eq!(attr["boolean"].to_float_literal::<f64>(), None);
        assert_eq!(attr["character"].to_bool_literal(), None);
    }
}
