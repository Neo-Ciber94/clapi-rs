use crate::{NameValue, NameValueAttributeArgs, NameValueError, Value};
use std::iter::Peekable;
use syn::{Attribute, AttributeArgs, Lit, Meta, MetaList, MetaNameValue, NestedMeta, Path};

/// Represents the arguments in a macro attribute like:
///
/// `#[attribute(key="value")]`
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MacroAttributeArgs {
    path: Option<String>,
    data: Vec<MetaItem>,
}

impl MacroAttributeArgs {
    pub fn new(attribute: Attribute) -> Self {
        let name = join_path_to_string(&attribute.path);
        let args = get_attribute_args(&attribute).expect("invalid attribute");
        let data = AttributeVisitor::visit(args);

        MacroAttributeArgs {
            path: Some(name),
            data,
        }
    }

    pub fn from_attribute_args(attribute_args: AttributeArgs) -> Self {
        let data = AttributeVisitor::visit(attribute_args);
        MacroAttributeArgs {
            path: None,
            data
        }
    }

    pub fn path(&self) -> Option<&str> {
        self.path.as_ref().map(|n| n.as_str())
    }

    pub fn data(&self) -> &[MetaItem] {
        self.data.as_slice()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &MetaItem> {
        self.data.iter()
    }

    pub fn into_name_values(self) -> Result<NameValueAttributeArgs, NameValueError> {
        NameValueAttributeArgs::new(self.path, self.data)
    }

    pub fn into_inner(self) -> Vec<MetaItem> {
        self.data
    }
}

impl<'a> IntoIterator for &'a MacroAttributeArgs {
    type Item = &'a MetaItem;
    type IntoIter = std::slice::Iter<'a, MetaItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

/// Represents the data in a attribute.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MetaItem {
    /// A path like: `#[attribute]`
    Path(String),
    /// A literal like: `#[attribute("hello world")]`
    Literal(Lit),
    /// A key-value like: `#[attribute(key="value")]` or `#[attribute(array=1,2,3,4)]`
    NameValue(NameValue),
    /// Nested data like: `#[attribute(inner("hello"))]`
    Nested(Box<MacroAttributeArgs>),
}

impl MetaItem {
    pub fn is_path(&self) -> bool {
        matches!(self, MetaItem::Path(_))
    }

    pub fn is_literal(&self) -> bool {
        matches!(self, MetaItem::Literal(_))
    }

    pub fn is_name_value(&self) -> bool {
        matches!(self, MetaItem::NameValue(_))
    }

    pub fn is_nested(&self) -> bool {
        matches!(self, MetaItem::Nested(_))
    }

    pub fn into_path(self) -> Option<String> {
        match self {
            MetaItem::Path(x) => Some(x),
            _ => None,
        }
    }

    pub fn into_literal(self) -> Option<Lit> {
        match self {
            MetaItem::Literal(x) => Some(x),
            _ => None,
        }
    }

    pub fn into_name_value(self) -> Option<NameValue> {
        match self {
            MetaItem::NameValue(x) => Some(x),
            _ => None,
        }
    }

    pub fn into_nested(self) -> Option<Box<MacroAttributeArgs>> {
        match self {
            MetaItem::Nested(x) => Some(x),
            _ => None,
        }
    }
}

struct AttributeVisitor;

impl AttributeVisitor {
    pub fn visit(attribute_args: AttributeArgs) -> Vec<MetaItem> {
        let mut data = Vec::new();
        let mut iter = attribute_args.iter().peekable();

        while let Some(next) = iter.next() {
            match next {
                NestedMeta::Lit(lit) => AttributeVisitor::visit_lit(&mut data, lit),
                NestedMeta::Meta(meta) => AttributeVisitor::visit_meta(&mut iter, &mut data, meta),
            }
        }

        data
    }

    fn visit_lit(ret: &mut Vec<MetaItem>, lit: &Lit) {
        ret.push(MetaItem::Literal(lit.clone()))
    }

    fn visit_meta<'a, I>(iter: &mut Peekable<I>, ret: &mut Vec<MetaItem>, meta: &Meta)
    where
        I: Iterator<Item = &'a NestedMeta>,
    {
        match meta {
            Meta::Path(path) => AttributeVisitor::visit_path(ret, path),
            Meta::List(list) => AttributeVisitor::visit_list(ret, list),
            Meta::NameValue(name_value) => {
                AttributeVisitor::visit_name_value(iter, ret, name_value)
            }
        }
    }

    fn visit_path(ret: &mut Vec<MetaItem>, path: &Path) {
        let name = join_path_to_string(path);
        ret.push(MetaItem::Path(name))
    }

    fn visit_list(ret: &mut Vec<MetaItem>, list: &MetaList) {
        let path = join_path_to_string(&list.path);
        let mut values = Vec::new();
        let mut iter = list.nested.iter().peekable();

        while let Some(next) = iter.next() {
            match next {
                NestedMeta::Meta(meta) => {
                    AttributeVisitor::visit_meta(&mut iter, &mut values, meta)
                }
                NestedMeta::Lit(lit) => AttributeVisitor::visit_lit(&mut values, lit),
            }
        }

        ret.push(MetaItem::Nested(Box::new(MacroAttributeArgs {
            path: Some(path),
            data: values,
        })));
    }

    fn visit_name_value<'a, I>(
        iter: &mut Peekable<I>,
        ret: &mut Vec<MetaItem>,
        name_value: &MetaNameValue,
    ) where
        I: Iterator<Item = &'a NestedMeta>,
    {
        let key = join_path_to_string(&name_value.path);
        let mut values = Vec::new();
        values.push(name_value.lit.clone());

        while let Some(NestedMeta::Lit(lit)) = iter.peek() {
            values.push(lit.clone());
            iter.next();
        }

        debug_assert!(values.len() >= 1);
        match values.len() {
            1 => {
                let value = Value::Literal(values.remove(0));
                ret.push(MetaItem::NameValue(NameValue { key, value }))
            }
            _ => ret.push(MetaItem::NameValue(NameValue {
                key,
                value: Value::Array(values),
            })),
        }
    }
}

fn get_attribute_args(att: &Attribute) -> syn::Result<AttributeArgs> {
    let mut token_tree = att.tokens.clone().into_iter();
    if let Some(proc_macro2::TokenTree::Group(group)) = token_tree.next() {
        // let stream = group.stream().into();
        // return syn::parse_macro_input::parse::<AttributeArgs>(stream);
        use syn::parse_macro_input::ParseMacroInput;
        let tokens = group.stream();
        return syn::parse::Parser::parse2(AttributeArgs::parse, tokens);
    } else {
        Ok(AttributeArgs::new())
    }
}

fn join_path_to_string(path: &Path) -> String {
    path.segments.iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<String>>()
        .join("::")
}

pub fn attribute_data_to_string(data: &MetaItem) -> String {
    match data {
        MetaItem::Path(path) => path.to_owned(),
        MetaItem::Literal(lit) => literal_to_string(lit),
        MetaItem::NameValue(data) => match &data.value {
            Value::Literal(x) => format!("{} = {}", data.key, literal_to_string(x)),
            Value::Array(x) => {
                let s = x.iter().map(literal_to_string).collect::<Vec<String>>();

                format!("{} = {:?}", data.key, s)
            }
        },
        MetaItem::Nested(data) => {
            if data.len() > 0 {
                format!("{}", data.path.clone().unwrap())
            } else {
                format!("{}(...)", data.clone().path.unwrap())
            }
        }
    }
}

pub fn literal_to_string(lit: &Lit) -> String {
    match lit {
        Lit::Str(s) => s.value(),
        Lit::ByteStr(s) => unsafe { String::from_utf8_unchecked(s.value()) },
        Lit::Byte(s) => s.value().to_string(),
        Lit::Char(s) => s.value().to_string(),
        Lit::Int(s) => s.to_string(),
        Lit::Float(s) => s.to_string(),
        Lit::Bool(s) => s.value.to_string(),
        Lit::Verbatim(s) => s.to_string(),
    }
}