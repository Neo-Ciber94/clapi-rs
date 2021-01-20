#![allow(clippy::len_zero)]
use std::fmt::{Display, Formatter};

/// The path of an `Item`.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NamePath {
    path: Vec<String>
}

impl NamePath {
    /// Constructs a new `NamePath` with the given item name.
    pub fn new(name: String) -> Self {
        assert_valid_path(&name);
        NamePath { path: vec![name] }
    }

    /// Constructs a new `NamePath` from the given path.
    pub fn from_path(path: Vec<String>) -> Self {
        assert!(path.len() > 0, "path is invalid");

        for s in &path {
            assert_valid_path(s);
        }

        NamePath { path }
    }

    /// Returns the name of the item
    pub fn name(&self) -> &str {
        self.path.last()
            .map(|s| s.as_str())
            .unwrap()
    }

    /// Returns the path ignoring the item name.
    ///
    /// May be a empty slice if the item is a root item.
    pub fn item_path(&self) -> &[String]{
        &self.path[..self.path.len() - 1]
    }

    /// Returns the full path of the item.
    pub fn full_path(&self) -> &[String]{
        self.path.as_slice()
    }

    /// Returns a`Vec<String>` with full path of the `NamePath`.
    pub fn into_vec(self) -> Vec<String> {
        self.path
    }
}

impl Display for NamePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.join("::"))
    }
}

fn assert_valid_path(path: &str){
    if path.trim().is_empty() {
        panic!("path is invalid");
    }

    if !path.chars().all(|c|c.is_ascii_alphanumeric() || c == '_') {
        panic!("invalid path: `{}`, only `ascii alphanumeric` and `_` are valid", path);
    }
}