use std::fmt::{Display, Formatter};

/// The path of an `Item`.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NamePath {
    path: Vec<String>
}

impl NamePath {
    pub fn new(name: String) -> Self {
        assert_valid_path(&name);
        NamePath { path: vec![name] }
    }

    pub fn from_path(path: Vec<String>) -> Self {
        assert!(path.len() > 0, "path is empty");

        for s in &path {
            assert_valid_path(s);
        }

        NamePath { path }
    }

    pub fn name(&self) -> &str {
        self.path.last()
            .map(|s| s.as_str())
            .unwrap()
    }

    pub fn path(&self) -> &[String]{
        self.path.as_slice()
    }
}

impl Display for NamePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.join("::"))
    }
}

fn assert_valid_path(path: &str){
    if path.trim().is_empty() {
        panic!("path is empty");
    }

    if !path.chars().all(|c|c.is_ascii_alphanumeric() || c == '_') {
        panic!("invalid path: `{}`, only `ascii alphanumeric` and `_` are valid", path);
    }
}