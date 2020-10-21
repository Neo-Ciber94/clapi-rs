use crate::command::Command;
use crate::option::CommandOption;
use crate::root_command::RootCommand;
use linked_hash_set::LinkedHashSet;

/// Provides common values used for a command-line operation.
#[derive(Debug)]
pub struct Context {
    root: RootCommand,
    name_prefixes: LinkedHashSet<String>,
    alias_prefixes: LinkedHashSet<String>,
    arg_delimiters: LinkedHashSet<char>,
}

impl Context {
    fn empty(root: RootCommand) -> Self {
        Context {
            root,
            name_prefixes: LinkedHashSet::new(),
            alias_prefixes: LinkedHashSet::new(),
            arg_delimiters: LinkedHashSet::new(),
        }
    }

    /// Constructs a new `Context` with the `RootCommand`.
    pub fn new(root: RootCommand) -> Self {
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
        root: RootCommand,
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
        if let Some(opt) = self.root().options().get(name_or_alias) {
            return Some(opt);
        }

        for child in self.root().children() {
            if let Some(opt) = child.options().get(name_or_alias) {
                return Some(opt);
            }
        }

        None
    }

    /// Returns the `Command` by the specified name or `None` if not found.
    pub fn get_command(&self, name: &str) -> Option<&Command> {
        self.root().children().find(|c| c.name() == name)
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

    /// Returns `true` if there is a `CommandOption` with the specified name or alias in this context.
    pub fn has_option(&self, name_or_alias: &str) -> bool {
        self.get_option(name_or_alias).is_some()
    }

    /// Returns `true` if there is a `Command` with the specified name in this context.
    pub fn has_command(&self, name: &str) -> bool {
        self.root().children().any(|c| c.name() == name)
    }

    /// Returns the `RootCommand` used by this context.
    pub fn root(&self) -> &RootCommand {
        &self.root
    }

    pub(crate) fn root_mut(&mut self) -> &mut RootCommand {
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
