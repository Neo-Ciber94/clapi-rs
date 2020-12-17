use crate::command::Command;
use crate::option::CommandOption;
use linked_hash_set::LinkedHashSet;

/// Provides common values used for a command-line parsing.
#[derive(Debug)]
pub struct Context {
    root: Command,
    name_prefixes: LinkedHashSet<String>,
    alias_prefixes: LinkedHashSet<String>,
    arg_assign: LinkedHashSet<char>,
    delimiter: char,
}

impl Context {
    /// Constructs a new `Context` with the `RootCommand`.
    pub fn new(root: Command) -> Self {
        ContextBuilder::new(root).build()
    }

    /// Returns `true` if the value is a name prefix.
    pub fn is_name_prefix(&self, value: &str) -> bool {
        self.name_prefixes.contains(value)
    }

    /// Returns `true` if the value is an alias prefix.
    pub fn is_alias_prefix(&self, value: &str) -> bool {
        self.alias_prefixes.contains(value)
    }

    /// Returns an `Iterator` over the option name prefixes of this context.
    pub fn name_prefixes(&self) -> impl ExactSizeIterator<Item = &String> {
        self.name_prefixes.iter()
    }

    /// Returns an `Iterator` over the option alias prefixes of this context.
    pub fn alias_prefixes(&self) -> impl ExactSizeIterator<Item = &String> {
        self.alias_prefixes.iter()
    }

    /// Returns an `Iterator` over the argument assign `char`s.
    pub fn arg_assign(&self) -> impl ExactSizeIterator<Item = &char> {
        self.arg_assign.iter()
    }

    /// Returns the delimiter used in this context.
    pub fn delimiter(&self) -> char {
        self.delimiter
    }

    /// Returns the `CommandOption` by the specified name or alias or `None` if not found.
    pub fn get_option(&self, name_or_alias: &str) -> Option<&CommandOption> {
        if let Some(opt) = self.root().get_options().get(name_or_alias) {
            return Some(opt);
        }

        for child in self.root().get_children() {
            if let Some(opt) = child.get_options().get(name_or_alias) {
                return Some(opt);
            }
        }

        None
    }

    /// Returns the `Command` by the specified name or `None` if not found.
    pub fn get_command(&self, name: &str) -> Option<&Command> {
        self.root().get_children().find(|c| c.get_name() == name)
    }

    /// Returns `true` if the specified value starts with an option prefix.
    pub fn is_option_prefixed(&self, value: &str) -> bool {
        self.name_prefixes
            .iter()
            .any(|prefix| value.starts_with(prefix))
            || self
                .alias_prefixes
                .iter()
                .any(|prefix| value.starts_with(prefix))
    }

    /// Split the option and returns the `option` name.
    pub fn trim_prefix<'a>(&self, value: &'a str) -> &'a str {
        self.trim_and_get_prefix(value).1
    }

    /// Split the option and returns its `prefix` (if any) and the `option` name.
    pub fn trim_and_get_prefix<'a>(&self, value: &'a str) -> (Option<&'a str>, &'a str) {
        if let Some(prefix) = self
            .name_prefixes()
            .find(|prefix| value.starts_with(prefix.as_str()))
        {
            if let Some(index) = value.find(prefix) {
                let (prefix, value) = value.split_at(index + prefix.len());
                return (Some(prefix), value);
            }
        }

        if let Some(prefix) = self
            .alias_prefixes()
            .find(|prefix| value.starts_with(prefix.as_str()))
        {
            if let Some(index) = value.find(prefix) {
                let (prefix, value) = value.split_at(index + prefix.len());
                return (Some(prefix), value);
            }
        }

        (None, value)
    }

    /// Returns `true` if there is a `CommandOption` with the specified name or alias in this context.
    pub fn has_option(&self, name_or_alias: &str) -> bool {
        self.get_option(name_or_alias).is_some()
    }

    /// Returns `true` if there is a `Command` with the specified name in this context.
    pub fn has_command(&self, name: &str) -> bool {
        self.root().get_children().any(|c| c.get_name() == name)
    }

    /// Returns the `RootCommand` used by this context.
    pub fn root(&self) -> &Command {
        &self.root
    }

    pub(crate) fn root_mut(&mut self) -> &mut Command {
        &mut self.root
    }
}

#[derive(Clone)]
pub struct ContextBuilder {
    root: Command,
    name_prefixes: LinkedHashSet<String>,
    alias_prefixes: LinkedHashSet<String>,
    arg_assign: LinkedHashSet<char>,
    delimiter: Option<char>,
}

impl ContextBuilder {
    pub fn new(root: Command) -> Self {
        ContextBuilder {
            root,
            name_prefixes: Default::default(),
            alias_prefixes: Default::default(),
            arg_assign: Default::default(),
            delimiter: None,
        }
    }

    pub fn name_prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.name_prefixes.insert(prefix.into());
        self
    }

    pub fn alias_prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.alias_prefixes.insert(prefix.into());
        self
    }

    pub fn arg_assign(mut self, value: char) -> Self {
        self.arg_assign.insert(value);
        self
    }

    pub fn delimiter(mut self, value: char) -> Self {
        self.delimiter = Some(value);
        self
    }

    pub fn build(mut self) -> Context {
        let name_prefixes = {
            if self.name_prefixes.is_empty() {
                self.name_prefixes.insert("--".to_owned());
            }
            self.name_prefixes
        };

        let alias_prefixes = {
            if self.alias_prefixes.is_empty() {
                self.alias_prefixes.insert("-".to_owned());
            }
            self.alias_prefixes
        };

        let arg_assign = {
            if self.arg_assign.is_empty() {
                self.arg_assign.insert('=');
                self.arg_assign.insert(':');
            }
            self.arg_assign
        };

        let delimiter = self.delimiter.unwrap_or(',');

        Context {
            root: self.root,
            name_prefixes,
            alias_prefixes,
            arg_assign,
            delimiter,
        }
    }
}
