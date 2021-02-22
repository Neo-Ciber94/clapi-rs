#![allow(clippy::len_zero)]
use crate::command::Command;
use crate::context::Context;
use crate::error::{Error, ErrorKind, Result};
use crate::parser::Parser;
use crate::suggestion::SuggestionSource;
use crate::{Argument, ParseResult, CommandOption, OptionList};
use std::borrow::Borrow;
use std::fmt::Debug;
use std::result::Result as StdResult;
use crate::help::HelpSource;

/// Represents a command-line app.
#[derive(Debug)]
pub struct CommandLine {
    context: Context,
}

impl CommandLine {
    /// Constructs a new `CommandLine` with the provided `Command`.
    #[inline]
    pub fn new(root: Command) -> Self {
        CommandLine::with_context(Context::new(root))
    }

    /// Constructs a new `CommandLine` with the provided `Context`.
    pub fn with_context(mut context: Context) -> Self {
        // Adds a default `version` option if the command or any child have a version defined
        if contains_version_recursive(context.root()) {
            context.set_version_option(crate::default_version_option());
        }

        CommandLine { context }
    }

    /// Returns the `Context` used by this command-line.
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// Returns the `Command` used by this command-line.
    pub fn root(&self) -> &Command {
        &self.context.root()
    }

    /// Returns the `SuggestionProvider` used by this command-line.
    pub fn suggestions(&self) -> Option<&SuggestionSource> {
        self.context.suggestions()
    }

    /// Sets the default `Help`.
    pub fn use_default_help(mut self) -> Self {
        self.context.set_help_option(crate::context::default_help_option());
        self.context.set_help_command(crate::context::default_help_command());
        self
    }

    /// Sets the default `SuggestionProvider`.
    pub fn use_default_suggestions(self) -> Self {
        self.use_suggestions(SuggestionSource::new())
    }

    /// Sets the `SuggestionSource` of this command-line context.
    pub fn use_suggestions(mut self, suggestions: SuggestionSource) -> Self {
        self.context.set_suggestions(suggestions);
        self
    }

    /// Sets the `HelpSource` of this command-line context.
    pub fn use_help(mut self, help: HelpSource) -> Self {
        self.context.set_help(help);
        self
    }

    /// Sets the help option for this command-line context.
    pub fn use_help_option(mut self, option: CommandOption) -> Self {
        self.context.set_help_option(option);
        self
    }

    /// Sets the help command for this command-line context.
    pub fn use_help_command(mut self, command: Command) -> Self {
        self.context.set_help_command(command);
        self
    }

    /// Sets the version option for this command-line context.
    pub fn use_version_option(mut self, option: CommandOption) -> Self {
        self.context.set_version_option(option);
        self
    }

    /// Sets the version command for this command-line context.
    pub fn use_version_command(mut self, command: Command) -> Self {
        self.context.set_version_command(command);
        self
    }

    /// Parse the program arguments and runs the app.
    ///
    /// This is equivalent to `cmd_line.exec(std::env::args().skip(1))`.
    #[inline]
    pub fn parse_args(&mut self) -> Result<()> {
        // We skip the first element that may be the path of the executable
        self.parse_from(std::env::args().skip(1))
    }

    /// Parses the given arguments and runs the app.
    pub fn parse_from<S, I>(&mut self, args: I) -> Result<()>
        where
            S: Borrow<str>,
            I: IntoIterator<Item = S> {
        let mut parser = Parser::new(&self.context);
        let result = parser.parse(args);
        let parse_result = match result {
            Ok(r) => r,
            Err(error) => return self.handle_error(&parser, error),
        };

        // Checks if the command requires to display help
        if self.requires_help(Ok(&parse_result)) {
            return self.handle_help(&parse_result);
        }

        // Checks if the command requires to display the version
        if self.requires_version(&parse_result) {
            self.show_version(&parse_result);
            return Ok(());
        }

        // We borrow the value from the Option to avoid create a temporary
        let handler = parse_result.executing_command().get_handler();

        if let Some(mut handler) = handler {
            let options = parse_result.options();
            let args = parse_result.args();
            // Calls the handler and pass the arguments
            match (*handler)(options, args) {
                Ok(_) => Ok(()),
                Err(error) => {
                    // Special case, the caller can returns `ErrorKind::FallthroughHelp`
                    // to indicates the `CommandLine` to show a help message about the current command.
                    if matches!(error.kind(), ErrorKind::FallthroughHelp) {
                        self.display_help(None)
                    } else {
                        Err(error)
                    }
                }
            }
        } else {
            // Shows a help message if there is no handler
            self.display_help(None)
        }
    }

    fn requires_version(&self, result: &ParseResult) -> bool {
        if let Some(version_option) = self.context.version_option() {
            if result.options().contains(version_option.get_name()) {
                return true;
            }
        }

        if let Some(version_command) = self.context.help_command() {
            if result.executing_command().get_name() == version_command.get_name() {
                return true;
            }
        }

        false
    }

    fn show_version(&self, result: &ParseResult) {
        match result.executing_command().get_version() {
            Some(version) => {
                let name = result.executing_command().get_name();
                println!("{} {}", name, version);
            },
            None => unreachable!()
        }
    }

    fn handle_error(&self, parser: &Parser<'_>, error: Error) -> Result<()> {
        // todo: Returns the help message as a `Err` or `Ok`?
        // `Err` was decided initially due using an invalid `command` or `argument` is an error
        match error.kind() {
            ErrorKind::InvalidArgumentCount | ErrorKind::InvalidArgument(_)
            if self.context.help_option().is_some() || self.context.help_command().is_some() => {
                let usage_message = get_help_message(&self.context, None, MessageKind::Usage)?;
                Err(error.join(&format!("\n{}", &usage_message)))
            },
            ErrorKind::UnexpectedOption(_) if self.suggestions().is_some() => {
                self.display_option_suggestions(parser, error)
            },
            ErrorKind::UnexpectedCommand(_) if self.suggestions().is_some() => {
                self.display_command_suggestions(parser, error)
            },
            _ => {
                Err(error)
            }
        }
    }

    // Checks if the `ParseResult` or `Parser` requires to show a help message.
    // We use `std::result::Result` where `Ok` is a completed parse operation
    // and `Err` is a failed one.
    fn requires_help(&self, result: StdResult<&ParseResult, &Parser<'_>>) -> bool {
        let context = &self.context;

        if context.help_option().is_none() && context.help_command().is_none() {
            return false;
        }

        if let Some(help_option) = self.context.help_option() {
            let options = match result {
                Ok(parse_result) => parse_result.options(),
                Err(parser) => parser.options().unwrap()
            };

            if options.contains(help_option.get_name()) {
                return true;
            }
        }

        if let Some(help_command) = self.context.help_command() {
            return match result {
                Ok(parse_result) => help_command.get_name() == parse_result.executing_command().get_name(),
                Err(parser) => help_command.get_name() == parser.command().unwrap().get_name()
            }
        }

        false
    }

    fn handle_help(&self, parse_result: &ParseResult) -> Result<()> {
        // handler for either:
        // * --help [subcommand]
        // * [subcommand] --help
        if let Some(help_option) = self.context.help_option() {
            if parse_result.options().contains(help_option.get_name()) {
                let arg = parse_result.options().get(help_option.get_name())
                    .unwrap()
                    .get_arg();

                return self.display_help(arg);
            }
        }

        // handler for: help [subcommand]
        if let Some(help_command) = self.context.help_command() {
            if parse_result.executing_command().get_name() == help_command.get_name() {
                return self.display_help(parse_result.arg());
            }
        }

        // We check before enter is `ParseResult` contains a help flag,
        // so 1 of the 2 cases should be picked
        unreachable!()
    }

    fn display_help(&self, args: Option<&Argument>) -> Result<()> {
        let values = args.map(|s| s.get_values());
        print!("{}", get_help_message(&self.context, values, MessageKind::Help)?);
        Ok(())
    }

    fn display_option_suggestions(&self, parser: &Parser<'_>, error: Error) -> Result<()> {
        let unprefixed_option = match error.kind() {
            ErrorKind::UnexpectedOption(s) => self.context.trim_prefix(s),
            _ => unreachable!()
        };

        // SAFETY: We ensure `suggestions` is some before calling this method
        // check `CommandLine::handle_error`
        let suggestion_source = self.suggestions().unwrap();
        let command_options = parser.command()
            .unwrap()
            .get_options()
            .iter()
            .map(|o| o.get_name().to_string())
            .collect::<Vec<String>>();

        // Options suggestions
        let mut suggestions = suggestion_source.suggestions_for(unprefixed_option, &command_options);

        // Prefix all the suggested options
        let context = self.context();
        let options = parser.command().unwrap().get_options();

        for s in &mut suggestions {
            prefix_option(context, options, &mut s.value);
        }

        // Suggestion message
        let msg = suggestion_source
            .message_for(suggestions)
            .map(|s| format!("\n\n{}\n", s));

        // Returns the suggestion message
        match msg {
            Some(ref msg) => Err(error.join(msg)),
            None => Err(error)
        }
    }

    fn display_command_suggestions(&self, parser: &Parser<'_>, error: Error) -> Result<()> {
        let command_name = match error.kind() {
            ErrorKind::UnexpectedCommand(s) => s,
            _ => unreachable!()
        };

        // SAFETY: We ensure `suggestions` is some before calling this method
        // check `CommandLine::handle_error`
        let suggestion_source = self.suggestions().unwrap();
        let subcommands = parser.command()
            .unwrap()
            .get_subcommands()
            .map(|c| c.get_name().to_string())
            .collect::<Vec<String>>();

        // Suggested subcommands
        let suggestions = suggestion_source.suggestions_for(command_name, &subcommands);

        let msg = suggestion_source
            .message_for(suggestions)
            .map(|s| format!("\n\n{}\n", s));

        // Returns the suggestion message
        match msg {
            Some(ref msg) => Err(error.join(msg)),
            None => Err(error)
        }
    }
}

/// Type of the help message.
pub enum MessageKind {
    /// A help message.
    Help,
    /// A usage message.
    Usage,
}

// Adds a prefix to the option name
fn prefix_option(context: &Context, options: &OptionList, name: &mut String) {
    if options.get_by_alias(&name).is_some() {
        let prefix = context.alias_prefixes().next().unwrap();
        name.insert_str(0, prefix);
    }

    if options.get_by_name(&name).is_some() {
        let prefix = context.name_prefixes().next().unwrap();
        name.insert_str(0, prefix);
    }
}

// Checks if the option or any of its children have `version`
pub(crate) fn contains_version_recursive(command: &Command) -> bool {
    for c in command {
        if contains_version_recursive(c) {
            return true;
        }
    }

    command.get_version().is_some()
}

/// Returns a help message.
///
/// # Arguments
/// * context - the context used for get the `HelpSource`.
/// * values - the subcommand tree. For example: `["data", "get"]`, if `None` the root command will be used instead.
/// * kind - the type of help message.
pub fn get_help_message(context: &Context, values: Option<&[String]>, kind: MessageKind) -> Result<String> {
    fn find_command<'a>(root: &'a Command, children: &[String]) -> Result<&'a Command> {
        let mut current = root;

        for child_name in children {
            if let Some(cmd) = current.find_subcommand(child_name) {
                current = cmd;
            } else {
                return Err(Error::from(ErrorKind::UnexpectedCommand(
                    child_name.to_string(),
                )));
            }
        }

        Ok(current)
    }

    let command = match values {
        None => context.root(),
        Some(values) => find_command(&context.root(), values)?
    };

    let mut buf = String::new();
    match kind {
        MessageKind::Help => (context.help().help)(&mut buf, &context, command, true),
        MessageKind::Usage => (context.help().usage)(&mut buf, &context, command, true)
    }

    Ok(buf)
}

/// Split the given value `&str` into command-line args.
///
/// # Example
/// ```rust
/// use clapi::split_into_args;
///
/// let result = split_into_args("echo \"Hello World\" 123");
/// assert_eq!(
/// vec![
///     "echo".to_string(),
///     "Hello World".to_string(),
///     "123".to_string()],
/// result);
/// ```
#[inline]
#[doc(hidden)]
pub fn split_into_args(value: &str) -> Vec<String> {
    split_into_args_with_quote_escape(value, '\\')
}

/// Split the given value `&str` into command-line args using the default
/// platform quote escape:
/// - `"^"` for windows.
/// - `"\"` for unix and other platforms.
///
/// # Example
/// ```rust
/// use clapi::split_into_platform_args;
///
/// // on windows
/// if cfg!(windows){
///     let result = split_into_platform_args("echo ^\"Hello^\"");
///     assert_eq!(vec!["echo".to_string(), "\"Hello\"".to_string()], result);
/// } else {
///     let result = split_into_platform_args("echo \\\"Hello\\\"");
///     assert_eq!(vec!["echo".to_string(), "\"Hello\"".to_string()], result);
/// }
/// ```
#[inline]
#[doc(hidden)]
pub fn split_into_platform_args(value: &str) -> Vec<String> {
    #[cfg(target_os = "windows")]
    const QUOTE_ESCAPE: char = '^';
    #[cfg(not(target_os = "windows"))]
    const QUOTE_ESCAPE: char = '\\';

    split_into_args_with_quote_escape(value, QUOTE_ESCAPE)
}

/// Split the given value `&str` into command-line args
/// using the specified `quote_escape`.
#[doc(hidden)]
pub fn split_into_args_with_quote_escape(value: &str, quote_escape: char) -> Vec<String> {
    const DOUBLE_QUOTE : char = '"';

    let mut result = Vec::new();
    let mut buffer = String::new();
    let mut chars = value.chars().peekable();
    let mut in_quote = false;

    while let Some(next_char) = chars.next() {
        match next_char {
            _ if next_char.is_whitespace() && in_quote => {
                buffer.push(next_char)
            },
            _ if next_char.is_whitespace() => {
                if buffer.len() > 0 {
                    result.push(buffer.clone());
                    buffer.clear();
                }
            },
            DOUBLE_QUOTE if in_quote => {
                in_quote = false;
                result.push(buffer.clone());
                buffer.clear();
            },
            DOUBLE_QUOTE => {
                in_quote = true;
            },
            _ if next_char == quote_escape && chars.peek() == Some(&DOUBLE_QUOTE) => {
                buffer.push(chars.next().unwrap());
            },
            _ => {
                buffer.push(next_char);
            }
        }
    }

    // Add the rest
    if buffer.len() > 0 {
        result.push(buffer);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_arg_iterator_test1() {
        let args = split_into_args("create file \"hello_world.txt\"");
        assert_eq!("create", args[0]);
        assert_eq!("file", args[1]);
        assert_eq!("hello_world.txt", args[2]);
    }

    #[test]
    fn into_arg_iterator_test2() {
        let args = split_into_args("echo --times 5 \\\"bla\\\"");
        assert_eq!("echo", args[0]);
        assert_eq!("--times", args[1]);
        assert_eq!("5", args[2]);
        assert_eq!("\"bla\"", args[3]);
    }

    #[test]
    fn into_arg_iterator_test3() {
        let args = split_into_args("print\t --times:3 \"hello world\"");
        assert_eq!("print", args[0]);
        assert_eq!("--times:3", args[1]);
        assert_eq!("hello world", args[2]);
    }
}
