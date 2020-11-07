use crate::{attribute_data_to_string, literal_to_string, MetaItem};
use std::fmt::{Debug, Formatter};
use std::str::FromStr;
use syn::Lit;

type Map<K, V> = linked_hash_map::LinkedHashMap<K, V>;
type Iter<'a, K, V> = linked_hash_map::Iter<'a, K, V>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NameValueAttributeArgs {
    path: Option<String>,
    data: Map<String, Value>,
}

impl NameValueAttributeArgs {
    pub fn new(path: Option<String>, values: Vec<MetaItem>) -> Result<Self, NameValueError> {
        if let Some(index) = values.iter().position(|n| !n.is_name_value()) {
            return Err(NameValueError::InvalidValue(values[index].clone()));
        }

        let mut data = Map::new();

        for name_value in values.into_iter().map(|n| n.into_name_value().unwrap()) {
            if data.contains_key(&name_value.key) {
                return Err(NameValueError::DuplicatedKey(name_value.key));
            } else {
                data.insert(name_value.key, name_value.value);
            }
        }

        Ok(NameValueAttributeArgs { path, data })
    }

    pub fn path(&self) -> Option<&str> {
        self.path.as_ref().map(|n| n.as_str())
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.data.keys()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.data.iter()
    }
}

impl<'a> IntoIterator for &'a NameValueAttributeArgs{
    type Item = (&'a String, &'a Value);
    type IntoIter = Iter<'a, String, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
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
                write!(f, "`{}` is not a name-value", attribute_data_to_string(&x))
            }
            NameValueError::DuplicatedKey(x) => write!(f, "duplicated key: `{}`", x),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NameValue {
    pub key: String,
    pub value: Value,
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

    pub fn is_str(&self) -> bool {
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

    pub fn as_string_literal(&self) -> Option<String> {
        if let Value::Literal(lit) = self {
            return match lit {
                Lit::Str(x) => Some(x.value()),
                Lit::ByteStr(x) => unsafe { Some(String::from_utf8_unchecked(x.value())) },
                _ => None,
            };
        }

        None
    }

    pub fn as_char_literal(&self) -> Option<char> {
        if let Value::Literal(lit) = self {
            return match lit {
                Lit::Char(x) => Some(x.value()),
                _ => None,
            };
        }

        None
    }

    pub fn as_bool_literal(&self) -> Option<bool> {
        if let Value::Literal(lit) = self {
            return match lit {
                Lit::Bool(x) => Some(x.value),
                _ => None,
            };
        }

        None
    }

    pub fn as_byte_literal(&self) -> Option<u8> {
        if let Value::Literal(lit) = self {
            return match lit {
                Lit::Byte(x) => Some(x.value()),
                _ => None,
            };
        }

        None
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
                let value = literal_to_string(x);
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
                    let value = literal_to_string(arg);
                    let n = T::from_str(&value).ok()?;
                    ret.push(n);
                }
                Some(ret)
            }
            _ => None,
        }
    }
}
