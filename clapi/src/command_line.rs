#![allow(clippy::len_zero)]
use crate::command::Command;
use crate::context::Context;
use crate::error::{Error, ErrorKind, Result};
use crate::help::{DefaultHelp, HelpKind, Help};
use crate::parser::Parser;
use crate::suggestion::{SingleSuggestionProvider, SuggestionProvider};
use crate::{Argument, ParseResult, OptionList};
use std::borrow::Borrow;
use std::fmt::Debug;
use std::rc::Rc;
use std::result::Result as StdResult;

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
    pub fn with_context(context: Context) -> Self {
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

    /// Returns the `HelpProvider` used by this command-line or `None` if not set.
    pub fn help(&self) -> Option<&Rc<dyn Help>> {
        self.context.help()
    }

    /// Returns the `SuggestionProvider` used by this command-line.
    pub fn suggestions(&self) -> Option<&Rc<dyn SuggestionProvider>> {
        self.context.suggestions()
    }

    /// Sets the default `Help`.
    pub fn use_default_help(self) -> Self {
        self.use_help(DefaultHelp::default())
    }

    /// Sets the specified `Help`.
    pub fn use_help<H: Help + 'static>(mut self, help: H) -> Self {
        self.context.set_help(help);
        self
    }

    /// Sets the default `SuggestionProvider`.
    pub fn use_default_suggestions(self) -> Self {
        self.use_suggestions(SingleSuggestionProvider)
    }

    /// Sets the specified `SuggestionProvider`.
    pub fn use_suggestions<S: SuggestionProvider + 'static>(mut self, suggestions: S) -> Self {
        self.context.set_suggestions(suggestions);
        self
    }

    /// Runs this command-line app.
    ///
    /// This is equivalent to `cmd_line.exec(std::env::args().skip(1))`.
    #[inline]
    pub fn run(&mut self) -> Result<()> {
        // We skip the first element that may be the path of the executable
        self.run_with(std::env::args().skip(1))
    }

    /// Executes this command-line app passing the specified arguments.
    #[inline]
    pub fn run_with<S, I>(&mut self, args: I) -> Result<()>
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
        result.contains_option(self.context.version().name())
            && result.executing_command().get_version().is_some()
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
        match error.kind() {
            ErrorKind::InvalidArgumentCount | ErrorKind::InvalidArgument(_) if self.context.help().is_some() => {
                let usage_message = self.get_help_message(None, MessageKind::Usage)?;
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
        let help = match self.context.help() {
            Some(h) => h.as_ref(),
            None => return false,
        };

        match result {
            Ok(parse_result) => {
                match help.kind() {
                    HelpKind::Subcommand => {
                        is_help_subcommand(help, parse_result.executing_command())
                    },
                    HelpKind::Option => {
                        is_help_option(help, parse_result.options())
                    },
                    HelpKind::Any => {
                        is_help_subcommand(help, parse_result.executing_command())
                            || is_help_option(help, parse_result.options())
                    }
                }
            },
            Err(parser) => {
                match help.kind() {
                    HelpKind::Subcommand => {
                        is_help_subcommand(help, parser.command().unwrap())
                    }
                    HelpKind::Option => {
                        is_help_option(help, parser.options().unwrap())
                    }
                    HelpKind::Any => {
                        is_help_subcommand(help, parser.command().unwrap())
                            || is_help_option(help, parser.options().unwrap())
                    }
                }
            }
        }
    }

    fn handle_help(&self, parse_result: &ParseResult) -> Result<()> {
        let help = self.help().unwrap().as_ref();

        if is_help_option(help, parse_result.options()) {
            // handler for either:
            // * --help [subcommand]
            // * [subcommand] --help
            let arg = parse_result.get_option(help.name())
                .unwrap()
                .get_arg();

            self.display_help(arg)
        } else {
            // handler for: help [subcommand]
            self.display_help(parse_result.arg())
        }
    }

    fn display_help(&self, args: Option<&Argument>) -> Result<()> {
        println!("{}", self.get_help_message(args, MessageKind::Help)?);
        Ok(())
    }

    fn get_help_message(&self, args: Option<&Argument>, kind: MessageKind) -> Result<String> {
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

        match args {
            None => Ok(self.get_help_message_for_command(&self.context.root(), kind)),
            Some(arg) => {
                let subcommand = find_command(&self.context.root(), arg.get_values())?;
                Ok(self.get_help_message_for_command(subcommand, kind))
            }
        }
    }

    fn get_help_message_for_command(&self, command: &Command, kind: MessageKind) -> String {
        let help = self.help().expect("help command is not set");
        let mut buffer = crate::help::Buffer::new();

        match kind {
            MessageKind::Help => help.help(&mut buffer, &self.context, command),
            MessageKind::Usage => help.usage(&mut buffer, &self.context, command)
        };

        buffer.to_string()
    }

    fn display_option_suggestions(&self, parser: &Parser<'_>, error: Error) -> Result<()> {
        let prefixed_option = match error.kind() {
            ErrorKind::UnexpectedOption(s) => s,
            _ => unreachable!()
        };
        let unprefixed_option = self.context.trim_prefix(prefixed_option);

        // SAFETY: We ensure `suggestions` is some before calling this method
        // check `CommandLine::handle_error`
        let suggestions = self.suggestions().unwrap();
        let command_options = parser.command()
            .unwrap()
            .get_options()
            .iter()
            .map(|o| o.get_name().to_string())
            .collect::<Vec<String>>();

        let msg = suggestions
            .suggestions_for(unprefixed_option, &command_options)
            .map(|result| {
                suggestions.suggestion_message_for(result.map(|s| {
                    let context = self.context();
                    let options = parser.command().unwrap().get_options();
                    prefix_option(context, options, s)
                }))
            })
            .flatten()
            .map(|s| format!("\n\n{}", s));

        if let Some(msg) = msg {
            Err(error.join(&msg))
        } else {
            Err(error)
        }
    }

    fn display_command_suggestions(&self, parser: &Parser<'_>, error: Error) -> Result<()> {
        let prefixed_option = match error.kind() {
            ErrorKind::UnexpectedCommand(s) => s,
            _ => unreachable!()
        };
        let unprefixed_option = self.context.trim_prefix(prefixed_option);

        // SAFETY: We ensure `suggestions` is some before calling this method
        // check `CommandLine::handle_error`
        let suggestions = self.suggestions().unwrap();
        let command_options = parser.command()
            .unwrap()
            .get_subcommands()
            .map(|c| c.get_name().to_string())
            .collect::<Vec<String>>();

        let msg = suggestions
            .suggestions_for(unprefixed_option, &command_options)
            .map(|result| {
                suggestions.suggestion_message_for(result.map(|s| {
                    let context = self.context();
                    let options = parser.command().unwrap().get_options();
                    prefix_option(context, options, s)
                }))
            })
            .flatten()
            .map(|s| format!("\n\n{}\n", s));

        if let Some(msg) = msg {
            Err(error.join(&msg))
        } else {
            Err(error)
        }
    }
}

/// Type help message.
enum MessageKind {
    /// A help message.
    Help,
    /// A usage message.
    Usage,
}

fn prefix_option(context: &Context, options: &crate::option::OptionList, name: String) -> String {
    if options.get_by_alias(&name).is_some() {
        let prefix: String = context.alias_prefixes().next().cloned().unwrap();
        return format!("{}{}", prefix, name);
    }

    if options.get_by_name(&name).is_some() {
        let prefix: String = context.name_prefixes().next().cloned().unwrap();
        return format!("{}{}", prefix, name);
    }

    name
}

fn is_help_option<H: Help + ?Sized>(help: &H, options: &OptionList) -> bool{
    if let Some(alias) = help.alias() {
        if options.contains(alias){
            return true;
        }
    }

    options.contains(help.name())
}

fn is_help_subcommand<H: Help + ?Sized>(help: &H, command: &Command) -> bool {
    help.name() == command.get_name()
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
