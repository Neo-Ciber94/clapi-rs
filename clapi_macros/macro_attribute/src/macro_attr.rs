use crate::{NameValue, NameValueAttribute, NameValueError, Value};
use std::iter::Peekable;
use std::ops::Index;
use syn::{Attribute, AttributeArgs, Lit, Meta, MetaList, MetaNameValue, NestedMeta, Path};

/// Represents a macro attribute and its arguments like:
///
/// `#[attribute(key="value")]`
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MacroAttribute {
    path: String,
    args: Vec<MetaItem>,
}

impl MacroAttribute {
    pub fn new(attribute: Attribute) -> Self {
        let path = join_path_to_string(&attribute.path);
        let attr_args = get_attribute_args(&attribute).expect("invalid attribute");
        let args = AttributeArgsVisitor::visit(attr_args);

        MacroAttribute {
            path: path.to_string(),
            args
        }
    }

    pub fn from_attribute_args(path: &str, attribute_args: AttributeArgs) -> Self {
        let args = AttributeArgsVisitor::visit(attribute_args);
        MacroAttribute {
            path: path.to_string(),
            args
        }
    }

    pub fn path(&self) -> &str {
        self.path.as_str()
    }

    pub fn args(&self) -> &[MetaItem] {
        self.args.as_slice()
    }

    pub fn len(&self) -> usize {
        self.args.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &MetaItem> {
        self.args.iter()
    }

    pub fn into_name_values(self) -> Result<NameValueAttribute, NameValueError> {
        NameValueAttribute::new(self.path.as_str(), self.args)
    }

    pub fn into_inner(self) -> Vec<MetaItem> {
        self.args
    }
}

impl Index<usize> for MacroAttribute {
    type Output = MetaItem;

    fn index(&self, index: usize) -> &Self::Output {
        &self.args[index]
    }
}

impl<'a> IntoIterator for &'a MacroAttribute {
    type Item = &'a MetaItem;
    type IntoIter = std::slice::Iter<'a, MetaItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.args.iter()
    }
}

impl IntoIterator for MacroAttribute {
    type Item = MetaItem;
    type IntoIter = std::vec::IntoIter<MetaItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.args.into_iter()
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
    Nested(Box<MacroAttribute>),
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

    pub fn into_nested(self) -> Option<Box<MacroAttribute>> {
        match self {
            MetaItem::Nested(x) => Some(x),
            _ => None,
        }
    }
}

struct AttributeArgsVisitor;

impl AttributeArgsVisitor {
    pub fn visit(attribute_args: AttributeArgs) -> Vec<MetaItem> {
        let mut data = Vec::new();
        let mut iter = attribute_args.iter().peekable();

        while let Some(next) = iter.next() {
            match next {
                NestedMeta::Lit(lit) => AttributeArgsVisitor::visit_lit(&mut data, lit),
                NestedMeta::Meta(meta) => {
                    AttributeArgsVisitor::visit_meta(&mut iter, &mut data, meta)
                }
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
            Meta::Path(path) => AttributeArgsVisitor::visit_path(ret, path),
            Meta::List(list) => AttributeArgsVisitor::visit_list(ret, list),
            Meta::NameValue(name_value) => {
                AttributeArgsVisitor::visit_name_value(iter, ret, name_value)
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
                    AttributeArgsVisitor::visit_meta(&mut iter, &mut values, meta)
                }
                NestedMeta::Lit(lit) => AttributeArgsVisitor::visit_lit(&mut values, lit),
            }
        }

        ret.push(MetaItem::Nested(Box::new(MacroAttribute {
            path,
            args: values,
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
                ret.push(MetaItem::NameValue(NameValue { name: key, value }))
            }
            _ => ret.push(MetaItem::NameValue(NameValue {
                name: key,
                value: Value::Array(values),
            })),
        }
    }
}

fn get_attribute_args(attr: &Attribute) -> syn::Result<AttributeArgs> {
    let mut token_tree = attr.tokens.clone().into_iter();
    if let Some(proc_macro2::TokenTree::Group(group)) = token_tree.next() {
        use syn::parse_macro_input::ParseMacroInput;
        let tokens = group.stream();
        return syn::parse::Parser::parse2(AttributeArgs::parse, tokens);
    } else {
        Ok(AttributeArgs::new())
    }
}

fn join_path_to_string(path: &Path) -> String {
    path.segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<String>>()
        .join("::")
}

pub fn meta_item_to_string(data: &MetaItem) -> String {
    match data {
        MetaItem::Path(path) => path.to_owned(),
        MetaItem::Literal(lit) => literal_to_string(lit),
        MetaItem::NameValue(data) => match &data.value {
            Value::Literal(x) => format!("{} = {}", data.name, literal_to_string(x)),
            Value::Array(x) => {
                let s = x.iter().map(literal_to_string).collect::<Vec<String>>();

                format!("{} = {:?}", data.name, s)
            }
        },
        MetaItem::Nested(data) => {
            if data.len() > 0 {
                format!("{}", data.path.clone())
            } else {
                format!("{}(...)", data.clone().path)
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

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use quote::*;
    use syn::parse_quote::ParseQuote;

    fn parse_attr(tokens: TokenStream) -> Attribute {
        syn::parse::Parser::parse2(Attribute::parse, tokens).expect("invalid attribute")
    }

    #[test]
    fn new_macro_attr_test() {
        let tokens = quote! { #[person(name="Kaori", age=20, job(salary=200.0))] };
        let attr = MacroAttribute::new(parse_attr(tokens));

        assert_eq!(attr.path, "person".to_owned());
        assert_eq!(attr.len(), 3);
        assert!(attr[0].is_name_value());
        assert!(attr[1].is_name_value());
        assert!(attr[2].is_nested());
    }
}

