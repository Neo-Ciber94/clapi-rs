use std::collections::hash_map::{Keys, Iter};
use std::collections::HashMap;
use std::iter::Peekable;
use syn::export::ToTokens;
use syn::{Attribute, AttributeArgs, Meta, MetaList, MetaNameValue, NestedMeta, Path, Result};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct AttributeData {
    path: String,
    data: HashMap<String, Value>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Value {
    /// No value: `#[attribute(key)]`
    Empty,
    /// A literal: `#[attribute(key=literal)]`
    Literal(String),
    /// An array of literals: `#[attribute(key=1,2,3)]`
    Array(Vec<String>),
    /// A map of values: `#[attribute(key(x, y=1))]`
    Nested(HashMap<String, Value>),
}

impl Value {
    pub fn is_empty(&self) -> bool{
        match self{
            Value::Empty => true,
            _ => false
        }
    }

    pub fn is_literal(&self) -> bool{
        match self{
            Value::Literal(_) => true,
            _ => false
        }
    }

    pub fn is_array(&self) -> bool{
        match self{
            Value::Array(_) => true,
            _ => false
        }
    }

    pub fn is_nested(&self) -> bool{
        match self{
            Value::Nested(_) => true,
            _ => false
        }
    }

    pub fn into_literal(self) -> Option<String>{
        match self{
            Value::Literal(x) => Some(x),
            _ => None
        }
    }

    pub fn into_array(self) -> Option<Vec<String>>{
        match self{
            Value::Array(x) => Some(x),
            _ => None
        }
    }

    pub fn into_nested(self) -> Option<HashMap<String, Value>>{
        match self{
            Value::Nested(x) => Some(x),
            _ => None
        }
    }

    pub fn parse_literal<T: FromStr>(&self) -> Option<T> {
        match self{
            Value::Literal(x) => T::from_str(x).ok(),
            _ => None
        }
    }

    pub fn parse_array<T: FromStr>(&self) -> Option<Vec<T>>{
        match self{
            Value::Array(array) => {
                let mut ret = Vec::new();
                for arg in array {
                    let n = T::from_str(arg).ok()?;
                    ret.push(n);
                }
                Some(ret)
            },
            _ => None
        }
    }
}

impl AttributeData {
    pub fn new(att: Attribute) -> Self {
        AttributeVisitor::new(att).visit()
    }

    pub fn from_attribute_args(path: String, args: AttributeArgs) -> Self {
        AttributeVisitor::from_args(path, args).visit()
    }

    pub fn path(&self) -> &str {
        self.path.as_str()
    }

    pub fn data(&self) -> &HashMap<String, Value> {
        &self.data
    }

    pub fn keys(&self) -> Keys<'_, String, Value> {
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

    pub fn iter(&self) -> Iter<'_, String, Value> {
        self.data.iter()
    }
}

impl IntoIterator for AttributeData {
    type Item =(String, Value);
    type IntoIter = std::collections::hash_map::IntoIter<String, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'a> IntoIterator for &'a AttributeData {
    type Item =(&'a String, &'a Value);
    type IntoIter = std::collections::hash_map::Iter<'a, String, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

#[derive(Debug)]
struct AttributeVisitor {
    path: String,
    args: AttributeArgs,
}

impl AttributeVisitor {
    pub fn new(att: Attribute) -> Self {
        let path = att.path.to_token_stream().to_string();
        let args = get_attribute_args(&att).unwrap();

        AttributeVisitor { path, args }
    }

    pub fn from_args(path: String, args: AttributeArgs) -> Self{
        AttributeVisitor { path, args }
    }

    pub fn visit(self) -> AttributeData {
        let mut data = HashMap::new();
        let mut iter = self.args.iter().peekable();

        while let Some(nested_meta) = iter.next() {
            if let NestedMeta::Meta(meta) = nested_meta {
                match meta {
                    Meta::Path(path) => self.visit_path(&mut data, path),
                    Meta::List(list) => self.visit_list(&mut data, list),
                    Meta::NameValue(name_value) => {
                        self.visit_name_value(&mut data, &mut iter, name_value)
                    }
                }
            } else {
                panic!(
                    "invalid token: `{}`",
                    nested_meta.to_token_stream().to_string()
                );
            }
        }

        AttributeData {
            path: self.path,
            data,
        }
    }

    fn visit_path(&self, data: &mut HashMap<String, Value>, path: &Path) {
        let key = path.to_token_stream().to_string();
        data.insert(key, Value::Empty);
    }

    fn visit_name_value<'a, I>(
        &self,
        data: &mut HashMap<String, Value>,
        iter: &mut Peekable<I>,
        name_value: &MetaNameValue,
    ) where
        I: Iterator<Item = &'a NestedMeta>,
    {
        let key = name_value.path.to_token_stream().to_string();
        let mut values = Vec::new();

        values.push(name_value.lit.to_token_stream().to_string());

        while let Some(NestedMeta::Lit(lit)) = iter.peek() {
            values.push(lit.to_token_stream().to_string());
            iter.next();
        }

        debug_assert!(values.len() > 0);

        if values.len() == 1 {
            data.insert(key, Value::Literal(values[0].clone()));
        } else {
            data.insert(key, Value::Array(values));
        }
    }

    fn visit_list(&self, data: &mut HashMap<String, Value>, list: &MetaList) {
        let key = list.path.to_token_stream().to_string();
        let mut map = HashMap::new();
        let mut iter = list.nested.iter().peekable();

        while let Some(nested_meta) = iter.next() {
            if let NestedMeta::Meta(meta) = nested_meta {
                match meta {
                    Meta::Path(path) => self.visit_path(&mut map, path),
                    Meta::List(list) => self.visit_list(&mut map, list),
                    Meta::NameValue(name_value) => {
                        self.visit_name_value(&mut map, &mut iter, name_value)
                    }
                }
            } else {
                panic!(
                    "invalid token: `{}`",
                    nested_meta.to_token_stream().to_string()
                );
            }
        }

        if map.is_empty() {
            data.insert(key, Value::Empty);
        } else {
            data.insert(key, Value::Nested(map));
        }
    }
}

pub fn get_attribute_args(att: &Attribute) -> Result<AttributeArgs> {
    let mut token_tree = att.tokens.clone().into_iter();
    if let Some(proc_macro2::TokenTree::Group(group)) = token_tree.next() {
        let stream = group.stream().into();
        syn::parse_macro_input::parse::<AttributeArgs>(stream)
    } else {
        unreachable!()
    }
}
