use crate::command::Command;
use crate::option::CommandOption;
use linked_hash_set::LinkedHashSet;
use crate::help::{Help, HelpKind};
use crate::suggestion::SuggestionProvider;
use std::rc::Rc;
use std::fmt::{Debug, Formatter};
use crate::utils::debug_option;
use crate::{VersionProvider, DefaultVersionProvider};

/// Provides configuration info for parsing a command.
///
/// # Example
/// ```
/// use clapi::{Command, Argument, CommandOption, Context, Parser};
/// use clapi::validator::parse_validator;
///
/// let command = Command::new("MyApp")
///     .arg(Argument::one_or_more("values"))
///     .option(CommandOption::new("enable")
///         .alias("e")
///         .requires_assign(true)
///         .arg(Argument::new().validator(parse_validator::<bool>())));
///
/// let context = Context::builder(command)
///     .alias_prefix("/")      // An alias prefix
///     .assign_operator(':')   // An assign operator for options: `--option:value`
///     .build();
///
/// let result = Parser::new(&context)
///     .parse(vec!["/e:false", "--", "hello", "hola"])
///     .unwrap();
///
/// assert!(result.get_option_arg("enable").unwrap().contains("false"));
/// assert!(result.arg().unwrap().contains("hello"));
/// assert!(result.arg().unwrap().contains("hola"));
/// ```
#[derive(Clone)]
pub struct Context {
    root: Command,
    help: Option<Rc<dyn Help + 'static>>,
    suggestions: Option<Rc<dyn SuggestionProvider + 'static>>,
    version: Rc<dyn VersionProvider + 'static>,
    name_prefixes: LinkedHashSet<String>,
    alias_prefixes: LinkedHashSet<String>,
    assign_operators: LinkedHashSet<char>,
    delimiter: char,
}

impl Context {
    /// Constructs a default `Context` with the given command.
    #[inline]
    pub fn new(root: Command) -> Self {
        ContextBuilder::new(root).build()
    }

    /// Constructs a `ContextBuilder` with the given command.
    #[inline]
    pub fn builder(root: Command) -> ContextBuilder {
        ContextBuilder::new(root)
    }

    /// Returns the `Command` used by this context.
    pub fn root(&self) -> &Command {
        &self.root
    }

    /// Returns an iterator over the option name prefixes of this context.
    pub fn name_prefixes(&self) -> Prefixes<'_> {
        Prefixes {
            iter: self.name_prefixes.iter()
        }
    }

    /// Returns an iterator over the option alias prefixes of this context.
    pub fn alias_prefixes(&self) -> Prefixes<'_> {
        Prefixes {
            iter: self.alias_prefixes.iter()
        }
    }

    /// Returns an iterator over the assign operator `char`s.
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

    /// Returns the `VersionProvider` used by this context.
    pub fn version(&self) -> &Rc<dyn VersionProvider> {
        &self.version
    }

    /// Sets the `Help` provider of this context.
    pub fn set_help<H: Help + 'static>(&mut self, help: H) {
        self.help = Some(Rc::new(help));
    }

    /// Sets the `SuggestionProvider` of this context.
    pub fn set_suggestions<S: SuggestionProvider + 'static>(&mut self, suggestions: S) {
        self.suggestions = Some(Rc::new(suggestions));
    }

    /// Returns the `CommandOption` with the given name or alias or `None` if not found.
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

    /// Returns the `Command` with the given name or `None` if not found.
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

    /// Returns `true` if the name match with the `help` provider.
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

    /// Returns `true` if the name match with a `version` provider.
    pub fn is_version<S: AsRef<str>>(&self, name: S) -> bool {
        if let Some(alias) = self.version.alias() {
            self.version.name() == name.as_ref() || alias == name.as_ref()
        } else {
            self.version.name() == name.as_ref()
        }
    }

    /// Removes the prefix from the given option
    pub fn trim_prefix<'a>(&self, option: &'a str) -> &'a str {
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
            .field("version", &"VersionProvider")
            .field("name_prefixes", &self.name_prefixes)
            .field("alias_prefixes", &self.alias_prefixes)
            .field("arg_assign", &self.assign_operators)
            .field("delimiter", &self.delimiter)
            .finish()
    }
}

/// An iterator over option prefixes.
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

/// A builder for `Context`.
#[derive(Clone)]
pub struct ContextBuilder {
    root: Command,
    help: Option<Rc<dyn Help>>,
    suggestions: Option<Rc<dyn SuggestionProvider>>,
    version: Option<Rc<dyn VersionProvider + 'static>>,
    name_prefixes: LinkedHashSet<String>,
    alias_prefixes: LinkedHashSet<String>,
    assign_operators: LinkedHashSet<char>,
    delimiter: Option<char>,
}

impl ContextBuilder {
    /// Constructs a default `ContextBuilder` for the given command.
    pub fn new(root: Command) -> Self {
        ContextBuilder {
            root,
            help: None,
            suggestions: None,
            version: None,
            name_prefixes: Default::default(),
            alias_prefixes: Default::default(),
            assign_operators: Default::default(),
            delimiter: None,
        }
    }

    /// Adds an option name prefix to the context.
    pub fn name_prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        let prefix = prefix.into();
        assert_valid_symbol("prefixes", prefix.as_str());
        self.name_prefixes.insert(prefix);
        self
    }

    /// Adds a option alias prefix to the context.
    pub fn alias_prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        let prefix = prefix.into();
        assert_valid_symbol("prefixes", prefix.as_str());
        self.alias_prefixes.insert(prefix);
        self
    }

    /// Adds an assign operator for this context.
    pub fn assign_operator(mut self, value: char) -> Self {
        // A char is always 4 bytes
        assert_valid_symbol("assign chars", value.encode_utf8(&mut [0;4]));
        self.assign_operators.insert(value);
        self
    }

    /// Sets the delimiter for this context.
    pub fn delimiter(mut self, value: char) -> Self {
        // A char is always 4 bytes
        assert_valid_symbol("delimiters", value.encode_utf8(&mut [0;4]));
        self.delimiter = Some(value);
        self
    }

    /// Sets the `Help` for this context.
    pub fn help<H: Help + 'static>(mut self, help: H) -> Self {
        self.help = Some(Rc::new(help));
        self
    }

    /// Sets the `SuggestionProvider` for this context.
    pub fn suggestions<S: SuggestionProvider + 'static>(mut self, suggestions: S) -> Self {
        self.suggestions = Some(Rc::new(suggestions));
        self
    }

    /// Sets the `VersionProvider` for this context.
    pub fn version<V: VersionProvider + 'static>(mut self, version: V) -> Self {
        self.version = Some(Rc::new(version));
        self
    }

    /// Constructs a `Context` using this builder data.
    pub fn build(mut self) -> Context {
        Context {
            // Root Command
            root: self.root,

            // Help provider
            help: self.help,

            // Suggestion Provider
            suggestions: self.suggestions,

            // Version provider
            version: self.version.unwrap_or_else(|| Rc::new(DefaultVersionProvider)),

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

// Checks if the given string is a help command.
pub(crate) fn is_help_command(context: &Context, name: &str) -> bool {
    if let Some(help) = context.help() {
        matches!(help.kind(), HelpKind::Any | HelpKind::Subcommand) && help.name() == name
    } else {
        false
    }
}

// Checks if the given string is a help option.
pub(crate) fn is_help_option(context: &Context, name: &str) -> bool {
    if let Some(help) = context.help() {
        matches!(help.kind(), HelpKind::Any | HelpKind::Option)
            && (help.name() == name || help.alias().map_or(false, |s| s == name))
    } else {
        false
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