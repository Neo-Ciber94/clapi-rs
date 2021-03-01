use crate::command::Command;
use crate::option::CommandOption;
use crate::suggestion::SuggestionSource;
use std::fmt::{Debug, Formatter};
use crate::utils::debug_option;
use crate::Argument;
use crate::help::HelpSource;

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
/// assert!(result.options().get_arg("enable").unwrap().contains("false"));
/// assert!(result.arg().unwrap().contains("hello"));
/// assert!(result.arg().unwrap().contains("hola"));
/// ```
#[derive(Clone)]
pub struct Context {
    root: Command,
    suggestions: Option<SuggestionSource>,
    help: HelpSource,
    name_prefixes: Vec<String>,
    alias_prefixes: Vec<String>,
    assign_operators: Vec<char>,
    delimiter: char,
    help_option: Option<CommandOption>,
    help_command: Option<Command>,
    version_option: Option<CommandOption>,
    version_command: Option<Command>,
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

    /// Returns the `SuggestionProvider` or `None` if not set.
    pub fn suggestions(&self) -> Option<&SuggestionSource> {
        self.suggestions.as_ref()
    }

    /// Returns the `HelpSource` of this context.
    pub fn help(&self) -> &HelpSource {
        &self.help
    }

    /// Gets the help `CommandOption` of this context.
    pub fn help_option(&self) -> Option<&CommandOption> {
        self.help_option.as_ref()
    }

    /// Gets the help `Command` of this context.
    pub fn help_command(&self) -> Option<&Command> {
        self.help_command.as_ref()
    }

    /// Gets the version `CommandOption` of this context.
    pub fn version_option(&self) -> Option<&CommandOption> {
        self.version_option.as_ref()
    }

    /// Gets the version `Command` of this context.
    pub fn version_command(&self) -> Option<&Command> {
        self.version_command.as_ref()
    }

    /// Sets the `SuggestionSource` of this context.
    pub fn set_suggestions(&mut self, suggestions: SuggestionSource) {
        self.suggestions = Some(suggestions);
    }

    /// Sets the `HelpSource` of this context.
    pub fn set_help(&mut self, help: HelpSource) {
        self.help = help;
    }

    /// Sets the help `CommandOption` of this context.
    pub fn set_help_option(&mut self, option: CommandOption) {
        assert!(self.help_option.is_none(), "`Context` already contains a help option");
        self.help_option = Some(option);
        add_command_builtin_help_option(self);
    }

    /// Sets the help `Command` of this context.
    pub fn set_help_command(&mut self, command: Command) {
        assert!(self.help_command.is_none(), "`Context` already contains a help command");
        self.help_command = Some(command);
        add_command_builtin_help_command(self);
    }

    /// Sets the version `CommandOption` of this context.
    pub fn set_version_option(&mut self, option: CommandOption) {
        assert!(self.version_option.is_none(), "`Context` already contains a version option");
        self.version_option = Some(option);
        add_command_builtin_version_option(self);
    }

    /// Sets the version `Command` of this context.
    pub fn set_version_command(&mut self, command: Command) {
        assert!(self.version_command.is_none(), "`Context` already contains a version command");
        self.version_command = Some(command);
        add_command_builtin_version_command(self);
    }

    /// Returns the `CommandOption` with the given name or alias or `None` if not found.
    pub fn get_option(&self, name_or_alias: &str) -> Option<&CommandOption> {
        if let Some(opt) = self.root().get_options().get(name_or_alias) {
            return Some(opt);
        }

        for child in self.root().get_subcommands() {
            if let Some(opt) = child.get_options().get(name_or_alias) {
                return Some(opt);
            }
        }

        None
    }

    /// Returns the `Command` with the given name or `None` if not found.
    pub fn get_command(&self, name: &str) -> Option<&Command> {
        self.root().get_subcommands().find(|c| c.get_name() == name)
    }

    /// Returns `true` if the value is a name prefix.
    pub fn is_name_prefix(&self, value: &str) -> bool {
        self.name_prefixes.iter().any(|s| s == value)
    }

    /// Returns `true` if the value is an alias prefix.
    pub fn is_alias_prefix(&self, value: &str) -> bool {
        self.alias_prefixes.iter().any(|s| s == value)
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
            .field("suggestions", &debug_option(&self.suggestions, "SuggestionSource"))
            .field("help", &"HelpSource")
            .field("name_prefixes", &self.name_prefixes)
            .field("alias_prefixes", &self.alias_prefixes)
            .field("assign_operators", &self.assign_operators)
            .field("delimiter", &self.delimiter)
            .field("help_option", &self.help_option)
            .field("help_command", &self.help_command)
            .field("version_option", &self.version_option)
            .field("version_command", &self.version_command)
            .finish()
    }
}

/// An iterator over option prefixes.
#[derive(Debug, Clone)]
pub struct Prefixes<'a> {
    iter: std::slice::Iter<'a, String>
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
    suggestions: Option<SuggestionSource>,
    help: Option<HelpSource>,
    name_prefixes: Vec<String>,
    alias_prefixes: Vec<String>,
    assign_operators: Vec<char>,
    delimiter: Option<char>,
    help_option: Option<CommandOption>,
    help_command: Option<Command>,
    version_option: Option<CommandOption>,
    version_command: Option<Command>,
}

impl ContextBuilder {
    /// Constructs a default `ContextBuilder` for the given command.
    pub fn new(root: Command) -> Self {
        ContextBuilder {
            root,
            suggestions: None,
            help: None,
            name_prefixes: Default::default(),
            alias_prefixes: Default::default(),
            assign_operators: Default::default(),
            delimiter: None,
            help_option: None,
            help_command: None,
            version_option: None,
            version_command: None,
        }
    }

    /// Adds an option name prefix to the context.
    pub fn name_prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        let prefix = prefix.into();
        assert_valid_symbol("prefixes", prefix.as_str());
        self.name_prefixes.push(prefix);
        self
    }

    /// Adds a option alias prefix to the context.
    pub fn alias_prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        let prefix = prefix.into();
        assert_valid_symbol("prefixes", prefix.as_str());
        self.alias_prefixes.push(prefix);
        self
    }

    /// Adds an assign operator for this context.
    pub fn assign_operator(mut self, value: char) -> Self {
        // A char is always 4 bytes
        assert_valid_symbol("assign chars", value.encode_utf8(&mut [0;4]));
        self.assign_operators.push(value);
        self
    }

    /// Sets the delimiter for this context.
    pub fn delimiter(mut self, value: char) -> Self {
        // A char is always 4 bytes
        assert_valid_symbol("delimiters", value.encode_utf8(&mut [0;4]));
        self.delimiter = Some(value);
        self
    }

    /// Sets the `SuggestionSource` for this context.
    pub fn suggestions(mut self, suggestions: SuggestionSource) -> Self {
        self.suggestions = Some(suggestions);
        self
    }

    /// Sets the `HelpSource` for this context.
    pub fn help(mut self, help: HelpSource) -> Self {
        self.help = Some(help);
        self
    }

    /// Sets the help `CommandOption` for this context.
    pub fn help_option(mut self, option: CommandOption) -> Self {
        assert_is_help_option(&option);
        self.help_option = Some(option);
        self
    }

    /// Sets the help `Command` for this context.
    pub fn help_command(mut self, command: Command) -> Self {
        assert_is_help_command(&command);
        self.help_command = Some(command);
        self
    }

    /// Sets the version `CommandOption` for this context.
    pub fn version_option(mut self, option: CommandOption) -> Self {
        assert_is_version_option(&option);
        self.version_option = Some(option);
        self
    }

    /// Sets the version `Command` for this context.
    pub fn version_command(mut self, command: Command) -> Self {
        assert_is_version_command(&command);
        self.version_command = Some(command);
        self
    }

    /// Constructs a `Context` using this builder data.
    pub fn build(mut self) -> Context {
        let mut context = Context {
            // Root command
            root: self.root,

            // Suggestion provider
            suggestions: self.suggestions,

            // Help provider
            help: self.help.unwrap_or_else(|| HelpSource::default()),

            // Option name prefixes
            name_prefixes: {
                if self.name_prefixes.is_empty() {
                    self.name_prefixes.push("--".to_owned());
                }
                self.name_prefixes
            },

            // Option aliases prefixes
            alias_prefixes: {
                if self.alias_prefixes.is_empty() {
                    self.alias_prefixes.push("-".to_owned());
                }
                self.alias_prefixes
            },

            // Option argument assign
            assign_operators: {
                if self.assign_operators.is_empty() {
                    self.assign_operators.push('=');
                }
                self.assign_operators
            },

            // Argument values delimiter
            delimiter: self.delimiter.unwrap_or(','),

            // Help option
            help_option: self.help_option,

            // Help command
            help_command: self.help_command,

            // Version option
            version_option: self.version_option,

            // Version command
            version_command: self.version_command
        };

        add_command_builtin_help_option(&mut context);
        add_command_builtin_help_command(&mut context);
        add_command_builtin_version_option(&mut context);
        add_command_builtin_version_command(&mut context);
        context
    }
}

#[inline]
pub fn default_version_option() -> CommandOption {
    CommandOption::new("version")
        .alias("v")
        .description("Shows the version of the command")
}

#[inline]
pub fn default_version_command() -> Command {
    Command::new("version")
        .description("Shows the version of the command")
}

#[inline]
pub fn default_help_option() -> CommandOption {
    CommandOption::new("help")
        .alias("h")
        .description("Shows help information about a command")
        .hidden(true)
        .arg(Argument::zero_or_more("command"))
}

#[inline]
pub fn default_help_command() -> Command {
    Command::new("help")
        .description("Shows help information about a command")
        .arg(Argument::zero_or_more("command"))
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

#[inline]
fn assert_is_help_option(option: &CommandOption) {
    let arg = option.get_arg().expect("help option must take only 1 argument");
    assert_eq!(arg.get_values_count().min(), Some(0), "help option argument must take any count of values");
    assert_eq!(arg.get_values_count().max(), None, "help option argument must take any count of values");
}

#[inline]
fn assert_is_help_command(command: &Command) {
    let arg = command.get_arg().expect("help command must take only 1 argument");
    assert_eq!(arg.get_values_count().min(), Some(0), "help command argument must take any count of values");
    assert_eq!(arg.get_values_count().max(), None, "help command argument must take any count of values");
}

#[inline]
fn assert_is_version_option(option: &CommandOption) {
    if option.get_arg().is_some() {
        panic!("version option must take no arguments");
    }
}

#[inline]
fn assert_is_version_command(command: &Command) {
    if command.get_arg().is_some() {
        panic!("version command must take no arguments");
    }
}

#[inline]
fn add_command_builtin_help_option(context: &mut Context) {
    if context.root.get_subcommands().count() > 0 {
        if let Some(help_option) = context.help_option.as_ref().cloned() {
            let command = &mut context.root;
            add_option_recursive(command, help_option);
        }
    }
}

#[inline]
fn add_command_builtin_help_command(context: &mut Context) {
    if let Some(help_command) = context.help_command.as_ref().cloned() {
        context.root.add_command(help_command);
    }
}

#[inline]
fn add_command_builtin_version_option(context: &mut Context) {
    if context.root.get_subcommands().count() > 0 {
        if let Some(version_option) = context.version_option.as_ref().cloned() {
            let command = &mut context.root;
            add_option_recursive(command, version_option);
        }
    }
}

#[inline]
fn add_command_builtin_version_command(context: &mut Context) {
    if let Some(version_command) = context.version_command.as_ref().cloned() {
        context.root.add_command(version_command);
    }
}

fn add_option_recursive(command: &mut Command, option: CommandOption) {
    for subcommand in command.get_subcommands_mut() {
        add_option_recursive(subcommand, option.clone());
    }

    command.add_option(option);
}

// Checks if the given string is a help command.
pub(crate) fn is_help_command(context: &Context, name: &str) -> bool {
    if let Some(help_command) = context.help_command.as_ref() {
        help_command.get_name() == name
    } else {
        false
    }
}

// Checks if the given string is a help option.
pub(crate) fn is_help_option(context: &Context, name: &str) -> bool {
    if let Some(help_option) = context.help_option.as_ref() {
        help_option.get_name() == name || help_option.has_alias(name)
    } else {
        false
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