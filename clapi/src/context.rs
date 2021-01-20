use crate::command::Command;
use crate::option::CommandOption;
use linked_hash_set::LinkedHashSet;
use crate::help::{Help, HelpKind};
use crate::suggestion::SuggestionProvider;
use std::rc::Rc;
use std::fmt::{Debug, Formatter};
use crate::utils::debug_option;

/// Provides common values used for a command-line parsing.
#[derive(Clone)]
pub struct Context {
    root: Command,
    help: Option<Rc<dyn Help + 'static>>,
    suggestions: Option<Rc<dyn SuggestionProvider + 'static>>,
    name_prefixes: LinkedHashSet<String>,
    alias_prefixes: LinkedHashSet<String>,
    assign_operators: LinkedHashSet<char>,
    delimiter: char,
}

/*
struct Config {
    enum ContextOptions {
        AllowWhitespaces,
    }
}
*/

impl Context {
    /// Constructs a new `Context` with the `RootCommand`.
    #[inline]
    pub fn new(root: Command) -> Self {
        ContextBuilder::new(root).build()
    }

    #[inline]
    pub fn builder(root: Command) -> ContextBuilder {
        ContextBuilder::new(root)
    }

    /// Returns an `Iterator` over the option name prefixes of this context.
    pub fn name_prefixes(&self) -> Prefixes<'_> {
        Prefixes {
            iter: self.name_prefixes.iter()
        }
    }

    /// Returns an `Iterator` over the option alias prefixes of this context.
    pub fn alias_prefixes(&self) -> Prefixes<'_> {
        Prefixes {
            iter: self.alias_prefixes.iter()
        }
    }

    /// Returns an `Iterator` over the value assign `char`s.
    pub fn assign_operators(&self) -> impl ExactSizeIterator<Item = &char> {
        self.assign_operators.iter()
    }

    /// Returns the delimiter used in this context.
    pub fn delimiter(&self) -> char {
        self.delimiter
    }

    /// Returns the `Help` provider or `None` if not set.
    pub fn help(&self) -> Option<&Rc<dyn Help>> {
        self.help.as_ref()
    }

    /// Returns the `SuggestionProvider` or `None` if not set.
    pub fn suggestions(&self) -> Option<&Rc<dyn SuggestionProvider>> {
        self.suggestions.as_ref()
    }

    /// Sets the `Help` provider of this context.
    pub fn set_help<H: Help + 'static>(&mut self, help: H) {
        //todo: handle all HelpKind here, Any, Subcommand and Option?

        // If the `help` is a subcommand we add the subcommand to the root
        match help.kind() {
            HelpKind::Any | HelpKind::Subcommand => {
                self.root.add_command(
                    crate::help::to_command(&help)
                );
            }
            _ => {}
        }

        self.help = Some(Rc::new(help));
    }

    /// Sets the `SuggestionProvider` of this context.
    pub fn set_suggestions<S: SuggestionProvider + 'static>(&mut self, suggestions: S) {
        self.suggestions = Some(Rc::new(suggestions));
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

    /// Returns `true` if the value is a name prefix.
    pub fn is_name_prefix(&self, value: &str) -> bool {
        self.name_prefixes.contains(value)
    }

    /// Returns `true` if the value is an alias prefix.
    pub fn is_alias_prefix(&self, value: &str) -> bool {
        self.alias_prefixes.contains(value)
    }

    /// Returns `true` if the specified value starts with an option prefix.
    pub fn is_option_prefixed(&self, value: &str) -> bool {
        self.name_prefixes
            .iter()
            .chain(self.alias_prefixes.iter())
            .any(|prefix| value.starts_with(prefix))
    }

    /// Returns `true` if the `name` match with the `help` provider.
    pub fn is_help<S: AsRef<str>>(&self, name: S) -> bool {
        if let Some(help) = &self.help {
            if help.name() == name.as_ref() {
                return true;
            }

            if matches!(help.kind(), HelpKind::Option | HelpKind::Any) {
                if let Some(alias) = help.alias() {
                    return alias == name.as_ref();
                }
            }
        }

        false
    }

    /// Returns the `RootCommand` used by this context.
    pub fn root(&self) -> &Command {
        &self.root
    }

    /// Removes the prefix from the given option
    pub(crate) fn trim_prefix<'a>(&self, option: &'a str) -> &'a str {
        self.name_prefixes.iter()
            .chain(self.alias_prefixes.iter())
            .find(|prefix| option.starts_with(prefix.as_str()))
            .map(|prefix| option.strip_prefix(prefix))
            .flatten()
            .unwrap_or(option)
    }
}

impl Debug for Context {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("root", &self.root)
            .field("help", &debug_option(&self.help, "Help"))
            .field("suggestions", &debug_option(&self.suggestions, "SuggestionProvider"))
            .field("name_prefixes", &self.name_prefixes)
            .field("alias_prefixes", &self.alias_prefixes)
            .field("arg_assign", &self.assign_operators)
            .field("delimiter", &self.delimiter)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct Prefixes<'a> {
    iter: linked_hash_set::Iter<'a, String>
}

impl<'a> Iterator for Prefixes<'a> {
    type Item = &'a String;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a> ExactSizeIterator for Prefixes<'a>{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

#[derive(Clone)]
pub struct ContextBuilder {
    root: Command,
    help: Option<Rc<dyn Help>>,
    suggestions: Option<Rc<dyn SuggestionProvider>>,
    name_prefixes: LinkedHashSet<String>,
    alias_prefixes: LinkedHashSet<String>,
    assign_operators: LinkedHashSet<char>,
    delimiter: Option<char>,
}

impl ContextBuilder {
    pub fn new(root: Command) -> Self {
        ContextBuilder {
            root,
            help: None,
            suggestions: None,
            name_prefixes: Default::default(),
            alias_prefixes: Default::default(),
            assign_operators: Default::default(),
            delimiter: None,
        }
    }

    pub fn name_prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        let prefix = prefix.into();
        assert_valid_symbol("prefixes", prefix.as_str());
        self.name_prefixes.insert(prefix);
        self
    }

    pub fn alias_prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        let prefix = prefix.into();
        assert_valid_symbol("prefixes", prefix.as_str());
        self.alias_prefixes.insert(prefix);
        self
    }

    pub fn assign_operator(mut self, value: char) -> Self {
        // A char is always 4 bytes
        assert_valid_symbol("assign chars", value.encode_utf8(&mut [0;4]));
        self.assign_operators.insert(value);
        self
    }

    pub fn delimiter(mut self, value: char) -> Self {
        // A char is always 4 bytes
        assert_valid_symbol("delimiters", value.encode_utf8(&mut [0;4]));
        self.delimiter = Some(value);
        self
    }

    pub fn help<H: Help + 'static>(mut self, help: H) -> Self {
        self.help = Some(Rc::new(help));
        self
    }

    pub fn suggestions<S: SuggestionProvider + 'static>(mut self, suggestions: S) -> Self {
        self.suggestions = Some(Rc::new(suggestions));
        self
    }

    pub fn build(mut self) -> Context {
        // If the `help` is a subcommand we add the subcommand to the root
        if let Some(help) = &self.help {
            match help.kind() {
                HelpKind::Any | HelpKind::Subcommand => {
                    self.root.add_command(
                        crate::help::to_command(help.as_ref())
                    );
                }
                _ => {}
            }
        }

        Context {
            // Root Command
            root: self.root,

            // Help provider
            help: self.help,

            // Suggestion Provider
            suggestions: self.suggestions,

            // Delimiter
            delimiter: self.delimiter.unwrap_or(','),

            // Name prefixes
            name_prefixes: {
                if self.name_prefixes.is_empty() {
                    self.name_prefixes.insert("--".to_owned());
                }
                self.name_prefixes
            },

            // Alias prefixes
            alias_prefixes: {
                if self.alias_prefixes.is_empty() {
                    self.alias_prefixes.insert("-".to_owned());
                }
                self.alias_prefixes
            },

            // Assign operators
            assign_operators: {
                if self.assign_operators.is_empty() {
                    self.assign_operators.insert('=');
                }
                self.assign_operators
            },
        }
    }
}

#[inline]
fn assert_valid_symbol(source: &str, value: &str) {
    for c in value.chars() {
        if c.is_whitespace() {
            panic!("{} cannot contains whitespaces: `{}`", source, value);
        }

        if c.is_ascii_alphanumeric() {
            panic!("{} cannot contains numbers or letters: `{}`", source, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_test(){
        let context = Context::new(Command::root());
        assert!(context.is_name_prefix("--"));
        assert!(context.is_alias_prefix("-"));
        assert!(context.assign_operators().any(|c| *c == '='));
        assert_eq!(context.delimiter(), ',');
    }

    #[test]
    fn context_builder_test(){
        let context = Context::builder(Command::root())
            .name_prefix("#")
            .alias_prefix("##")
            .assign_operator('>')
            .delimiter('-')
            .build();

        assert!(context.is_name_prefix("#"));
        assert!(context.is_alias_prefix("##"));
        assert!(context.assign_operators().any(|c| *c == '>'));
        assert_eq!(context.delimiter(), '-');
    }

    #[test]
    #[should_panic(expected="prefixes cannot contains numbers or letters: `1`")]
    fn invalid_name_prefix_test() {
        Context::builder(Command::root()).name_prefix("1");
    }

    #[test]
    #[should_panic(expected="prefixes cannot contains whitespaces: `\t ab`")]
    fn invalid_alias_prefix_test() {
        Context::builder(Command::root()).alias_prefix("\t ab");
    }

    #[test]
    #[should_panic(expected="assign chars cannot contains whitespaces: `\t`")]
    fn invalid_assign_chars_test() {
        Context::builder(Command::root()).assign_operator('\t');
    }

    #[test]
    #[should_panic(expected="delimiters cannot contains whitespaces: `\t`")]
    fn invalid_delimiter_test() {
        Context::builder(Command::root()).delimiter('\t');
    }
}