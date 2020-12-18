use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ItemPath {
    path: Vec<String>
}

impl ItemPath {
    pub fn new(name: String) -> Self {
        ItemPath { path: vec![name] }
    }

    pub fn from_path(path: Vec<String>) -> Self {
        assert!(path.len() > 0);
        ItemPath { path }
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

impl Display for ItemPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.join("::"))
    }
}