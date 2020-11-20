use crate::command::Command;
use crate::option::CommandOption;
use linked_hash_set::LinkedHashSet;

/// Provides common values used for a command-line parsing.
#[derive(Debug)]
pub struct Context {
    root: Command,
    name_prefixes: LinkedHashSet<String>,
    alias_prefixes: LinkedHashSet<String>,
    arg_delimiters: LinkedHashSet<char>,
}

impl Context {
    fn empty(root: Command) -> Self {
        Context {
            root,
            name_prefixes: LinkedHashSet::new(),
            alias_prefixes: LinkedHashSet::new(),
            arg_delimiters: LinkedHashSet::new(),
        }
    }

    /// Constructs a new `Context` with the `RootCommand`.
    pub fn new(root: Command) -> Self {
        let mut context = Context::empty(root);
        context.add_name_prefix("--");
        context.add_alias_prefix("-");
        context.add_arg_delimiter(':');
        context.add_arg_delimiter('=');
        context
    }

    /// Constructs a new `Context` with the given values.
    ///
    /// # Arguments
    /// - `root`: the `RootCommand` used.
    /// - `name_prefixes`: the name prefixes used for the options, by default `"--"` is used.
    /// - `alias_prefixes`: the alias prefixed used for the options, by default `"-"` is used.
    /// - `arg_delimiter`: the delimiter to declare option arguments.
    pub fn with<'a, I, U>(
        root: Command,
        name_prefixes: I,
        alias_prefixes: I,
        arg_delimiters: U,
    ) -> Self
    where
        I: IntoIterator<Item = &'a str>,
        U: IntoIterator<Item = char>,
    {
        let mut context = Context::empty(root);

        for prefix in name_prefixes {
            context.add_name_prefix(prefix);
        }

        for prefix in alias_prefixes {
            context.add_alias_prefix(prefix);
        }

        for delimiter in arg_delimiters {
            context.add_arg_delimiter(delimiter);
        }

        // The context require both 1 option name and alias prefix
        // but the arguments delimiters aren't required

        assert!(
            context.name_prefixes().len() > 0,
            "context require at least 1 option name prefix"
        );
        assert!(
            context.alias_prefixes().len() > 0,
            "context require at least 1 option alias prefix"
        );

        context
    }

    /// Returns `true` if the value is a name prefix.
    pub fn is_name_prefix(&self, value: &str) -> bool {
        self.name_prefixes.contains(value)
    }

    /// Returns `true` if the value is an alias prefix.
    pub fn is_alias_prefix(&self, value: &str) -> bool {
        self.alias_prefixes.contains(value)
    }

    /// Returns an `ExactSizeIterator` over the option name prefixes of this context.
    pub fn name_prefixes(&self) -> impl ExactSizeIterator<Item = &String> {
        self.name_prefixes.iter()
    }

    /// Returns an `ExactSizeIterator` over the option alias prefixes of this context.
    pub fn alias_prefixes(&self) -> impl ExactSizeIterator<Item = &String> {
        self.alias_prefixes.iter()
    }

    /// Returns an `ExactSizeIterator` over the option argument delimiter of this context.
    pub fn arg_delimiters(&self) -> impl ExactSizeIterator<Item = &char> {
        self.arg_delimiters.iter()
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

    /// Trims the option prefix of the given value, and returns the value without prefix.
    pub fn trim_prefix<'a>(&self, value: &'a str) -> &'a str {
        if let Some(prefix) = self
            .name_prefixes()
            .find(|prefix| value.starts_with(prefix.as_str()))
        {
            return value.trim_start_matches(prefix);
        }

        if let Some(prefix) = self
            .alias_prefixes()
            .find(|prefix| value.starts_with(prefix.as_str()))
        {
            return value.trim_start_matches(prefix);
        }

        value
    }

    pub fn trim_and_get_prefix<'a>(&self, value: &'a str) -> (Option<&'a str>, &'a str) {
        if let Some(prefix) = self
            .name_prefixes()
            .find(|prefix| value.starts_with(prefix.as_str()))
        {
            if let Some(index) = value.find(prefix){
                let (prefix, value) = value.split_at(index + prefix.len());
                return (Some(prefix), value);
            }
        }

        if let Some(prefix) = self
            .alias_prefixes()
            .find(|prefix| value.starts_with(prefix.as_str()))
        {
            if let Some(index) = value.find(prefix){
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

    fn add_name_prefix(&mut self, prefix: &str) {
        self.assert_valid_prefix(prefix, "prefix");
        self.name_prefixes.insert(prefix.to_string());
    }

    fn add_alias_prefix(&mut self, prefix: &str) {
        self.assert_valid_prefix(prefix, "prefix");
        self.alias_prefixes.insert(prefix.to_string());
    }

    fn add_arg_delimiter(&mut self, delimiter: char) {
        assert!(
            !delimiter.is_whitespace(),
            "delimiter cannot be a whitespace"
        );
        self.arg_delimiters.insert(delimiter);
    }

    #[inline(always)]
    fn assert_valid_prefix(&self, prefix: &str, symbol_name: &str) {
        if prefix.is_empty() {
            panic!("{} cannot be empty", symbol_name);
        }

        if prefix.chars().any(|c| c.is_whitespace()) {
            panic!("{} cannot contains whitespaces", symbol_name)
        }

        if prefix.chars().count() == 1 {
            let c = &prefix.chars().next().unwrap();
            assert!(
                !self.arg_delimiters.contains(&c),
                "`{}` is an arg delimiter",
                prefix
            );
        }
    }
}
