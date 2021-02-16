#![allow(clippy::len_zero)]
use std::fmt::{Display, Formatter};

/// The path of an `Item`.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NamePath {
    path: Vec<String>
}

impl NamePath {
    /// Constructs a new `NamePath` with the given item name.
    pub fn new<S: Into<String>>(name: S) -> Self {
        let name = name.into();
        assert!(!name.trim().is_empty(), "path is empty");
        NamePath { path: vec![name] }
    }

    /// Constructs a new `NamePath` from the given path.
    pub fn from_path<S: Into<String>>(path: Vec<S>) -> Self {
        assert!(path.len() > 0, "path is invalid");

        let path = path.into_iter()
            .map(|s| s.into())
            .collect::<Vec<String>>();

        for s in &path {
            assert!(!s.trim().is_empty(), "path is empty");
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
        for (i, item) in self.path.iter().enumerate() {
            if i > 0 {
                write!(f, "::")?;
            }
            write!(f, "{}", item)?;
        }

        Ok(())
    }
}