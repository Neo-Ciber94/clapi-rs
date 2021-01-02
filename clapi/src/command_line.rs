use crate::command::Command;
use crate::context::Context;
use crate::error::{Error, ErrorKind, ParseError, Result};
use crate::help::{DefaultHelp, HelpKind, Help};
use crate::parser::Parser;
use crate::suggestion::{SingleSuggestionProvider, SuggestionProvider};
use crate::utils::OptionExt;
use crate::{Argument, ParseResult};
use std::borrow::Borrow;
use std::fmt::Debug;
use std::rc::Rc;

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

    /// Sets the default `HelpCommand`.
    pub fn use_default_help(self) -> Self {
        self.set_help(DefaultHelp(HelpKind::Any))
    }

    /// Sets the specified `HelpProvider`.
    pub fn set_help<H: Help + 'static>(mut self, help: H) -> Self {
        self.context.set_help(help);
        self
    }

    /// Sets the default `SuggestionProvider`.
    pub fn use_default_suggestions(self) -> Self {
        self.set_suggestions(SingleSuggestionProvider)
    }

    /// Sets the specified `SuggestionProvider`.
    pub fn set_suggestions<S: SuggestionProvider + 'static>(mut self, suggestions: S) -> Self {
        self.context.set_suggestions(suggestions);
        self
    }

    /// Executes this command-line app passing the specified arguments.
    #[inline]
    pub fn exec<S, I>(&mut self, args: I) -> Result<()>
    where
        S: Borrow<str>,
        I: IntoIterator<Item = S>,
    {
        let result = Parser.parse(&self.context, args);
        let parse_result = match result {
            Ok(r) => r,
            Err(error) => return self.handle_error(error),
        };

        let command = parse_result.command();

        // Check if the command require to display help
        if self.contains_help(&parse_result) {
            return self.handle_help(&parse_result);
        }

        // We borrow the value from the Option to avoid create a temporary
        let handler = command.get_handler();

        if let Some(mut handler) = handler {
            let options = parse_result.options();
            let args = parse_result.args();
            // Calls the handler and pass the arguments
            (*handler)(options, args)
        } else {
            // Shows a help message if there is no handler
            self.display_help(None)
        }
    }

    /// Runs this command-line app.
    ///
    /// This is equivalent to `cmd_line.exec(std::env::args().skip(1))`.
    #[inline]
    pub fn run(&mut self) -> Result<()> {
        self.exec(std::env::args().skip(1))
    }

    /// Executes this command-line app and pass the specified arguments as `&str`.
    ///
    /// This forwards the call to `CommandLine::exec` by slit the `str`.
    #[inline]
    pub fn exec_str(&mut self, args: &str) -> Result<()> {
        self.exec(split_into_args(args))
    }

    fn handle_error(&self, error: Error) -> Result<()> {
        // Special case, the caller can returns `ErrorKind::FallthroughHelp`
        // to indicates the `CommandLine` to show a help message.
        if error.kind() == &ErrorKind::FallthroughHelp {
            return self.display_help(None);
        }

        match error.try_into_parse_error() {
            Ok(parse_error) => {
                if self.contains_help(parse_error.parse_result()) {
                    return self.handle_help(parse_error.parse_result())
                }

                if self.suggestions().is_some() {
                    return Err(self.display_suggestions(parse_error));
                }

                Err(Error::from(parse_error))
            }
            Err(error) => {
                // If is a parse error and `InvalidArgumentCount`
                // we show a message about the usage of the command
                if error.kind() == &ErrorKind::InvalidArgumentCount {
                    let message = self.get_message(None, MessageKind::Usage)?;
                    return Err(Error::new(
                        error.kind().clone(),
                        format!("{}\n{}", error, message),
                    ));
                }

                return Err(error);
            }
        }
    }

    fn handle_help(&self, parse_result: &ParseResult) -> Result<()> {
        let command = parse_result.command();
        let help = self.help().unwrap();

        if is_help_option(help.as_ref(), parse_result) {
            // handler for: subcommand --help [ignore args]
            if command.get_parent().is_some() {
                print!("{}", self.get_message_for_command(command, MessageKind::Help));
                Ok(())
            } else {
                // handler for: --help [subcommand]
                let args = parse_result.get_option(help.name())
                    .unwrap()
                    .get_arg();

                self.display_help(args)
            }
        } else {
            // handler for: help [subcommand]
            self.display_help(parse_result.arg())
        }
    }

    fn contains_help(&self, parse_result: &ParseResult) -> bool {
        if let Some(help) = &self.help() {
            match help.kind() {
                HelpKind::Subcommand => {
                    is_help_subcommand(help.as_ref(), parse_result)
                },
                HelpKind::Option => {
                    is_help_option(help.as_ref(), parse_result)
                },
                HelpKind::Any => {
                    is_help_subcommand(help.as_ref(), parse_result)
                        || is_help_option(help.as_ref(), parse_result)
                }
            }
        } else {
            false
        }
    }

    fn get_message(&self, args: Option<&Argument>, kind: MessageKind) -> Result<String> {
        fn find_command<'a>(root: &'a Command, children: &[String]) -> Result<&'a Command> {
            let mut current = root;

            for i in 0..children.len() {
                let child_name = children[i].as_str();
                if let Some(cmd) = current.find_subcommand(child_name) {
                    current = cmd;
                } else {
                    return Err(Error::from(ErrorKind::UnrecognizedCommand(
                        child_name.to_string(),
                    )));
                }
            }

            Ok(current)
        }

        match args {
            None => Ok(self.get_message_for_command(&self.context.root(), kind)),
            Some(arg) => {
                let subcommand = find_command(&self.context.root(), arg.get_values())?;
                Ok(self.get_message_for_command(subcommand, kind))
            }
        }
    }

    fn get_message_for_command(&self, command: &Command, kind: MessageKind) -> String {
        let help = self.help().expect("help command is not set");
        let mut buffer = crate::help::Buffer::new();

        match kind {
            MessageKind::Help => help.help(&mut buffer, &self.context, command).unwrap(),
            MessageKind::Usage => help.usage(&mut buffer, &self.context, command).unwrap(),
        };

        buffer.to_string()
    }

    fn display_help(&self, args: Option<&Argument>) -> Result<()> {
        print!("{}", self.get_message(args, MessageKind::Help)?);
        Ok(())
    }

    fn display_suggestions(&self, parse_error: ParseError) -> Error {
        // SAFETY: We check if the method is `Some` before enter
        let provider = self.suggestions().unwrap();
        let kind = parse_error.kind();

        let (value, source) = match kind {
            ErrorKind::UnrecognizedCommand(s) => (
                s,
                parse_error
                    .command()
                    .get_children()
                    .map(|c| c.get_name().to_string())
                    .collect::<Vec<String>>(),
            ),
            ErrorKind::UnrecognizedOption(_, s) => (
                s,
                parse_error
                    .command()
                    .get_options()
                    .iter()
                    .map(|o| o.get_name().to_string())
                    .collect::<Vec<String>>(),
            ),
            // Forwards the error
            _ => return Error::from(parse_error),
        };

        let suggestions = provider
            .suggestions_for(value, &source)
            .map(|result| {
                provider.suggestion_message_for(result.map(|s| {
                    let context = self.context();
                    let options = parse_error.command().get_options();
                    prefix_option(context, options, s)
                }))
            })
            .flatten();

        if let Some(msg) = suggestions {
            Error::new(kind.clone(), msg)
        } else {
            Error::from(parse_error)
        }
    }
}

/// Type of the suggestion message of the `CommandLine`.
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

fn is_help_option<H: Help + ?Sized>(help: &H, parse_result: &ParseResult) -> bool{
    if let Some(alias) = help.alias() {
        if parse_result.contains_option(alias){
            return true;
        }
    }

    parse_result.contains_option(help.name())
}

fn is_help_subcommand<H: Help + ?Sized>(help: &H, parse_result: &ParseResult) -> bool {
    help.name() == parse_result.command().get_name()
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
            _ if next_char == quote_escape && chars.peek().contains_some(&DOUBLE_QUOTE) => {
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
