use crate::command::Command;
use crate::context::Context;
use crate::error::{Error, ErrorKind, ParseError, Result};
use crate::help::{DefaultHelpProvider, HelpProvider};
use crate::parser::{DefaultParser, Parser};
use crate::suggestion::{SingleSuggestionProvider, SuggestionProvider};
use crate::utils::{OptionExt, debug_option};
use std::fmt::{Debug, Formatter};
use crate::{Argument, HelpKind, CommandOption, ParseResult};

/// Represents a command-line app.
pub struct CommandLine {
    context: Context,
    //parser: P,
    help: Option<Box<dyn HelpProvider>>,
    suggestions: Option<Box<dyn SuggestionProvider>>,
    show_help_when_not_handler: bool,
}

impl CommandLine {
    /// Constructs a new `CommandLine` with the provided `RootCommand`.
    #[inline]
    pub fn new(root: Command) -> Self {
        CommandLine::with_context(Context::new(root))
    }

    /// Constructs a new `CommandLine` with the provided `Context`.
    pub fn with_context(context: Context) -> Self {
        CommandLine {
            context,
            help: None,
            suggestions: None,
            show_help_when_not_handler: true,
        }
    }

    /// Returns the `Context` used by this command-line.
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// Returns the `RootCommand` used by this command-line.
    pub fn root(&self) -> &Command {
        &self.context.root()
    }

    /// Returns the `HelpCommand` used by this command-line or `None` if not set.
    pub fn help(&self) -> Option<&Box<dyn HelpProvider>> {
        self.help.as_ref()
    }

    /// Returns `true` if this command-line will call the `HelpCommand` if the handler
    /// of the called command is not specified.
    pub fn show_help_when_not_handler(&self) -> bool {
        self.show_help_when_not_handler
    }

    /// Returns the `SuggestionProvider` used by this command-line.
    pub fn suggestions(&self) -> Option<&Box<dyn SuggestionProvider>> {
        self.suggestions.as_ref()
    }

    /// Sets the default `HelpCommand`.
    pub fn use_default_help(self) -> Self {
        self.set_help(DefaultHelpProvider(crate::HelpKind::Subcommand))
    }

    /// Sets the specified `HelpProvider`.
    pub fn set_help(mut self, help: impl HelpProvider + 'static) -> Self {
        match help.kind() {
            HelpKind::Subcommand => {
                assert_eq!(
                    self.context.root().find_subcommand(help.name()),
                    None, "Command `{}` already exists", help.name()
                );

                let command = Command::new(help.name())
                    .description(help.description())
                    .arg(Argument::zero_or_more("args"));

                self.context.root_mut().add_command(command);
            }
            HelpKind::Option => {
                let option = CommandOption::new(help.name())
                    .description(help.description())
                    .arg(Argument::zero_or_more("args"));

                self.context.root_mut().add_option(option);
            }
        }
        self.help = Some(Box::new(help));
        self
    }

    /// Specify if this command-line will call the `HelpCommand` when the called command
    /// handler is not specified.
    pub fn set_show_help_when_no_handler(mut self, show_help: bool) -> Self {
        self.show_help_when_not_handler = show_help;
        self
    }

    /// Sets the default `SuggestionProvider`.
    pub fn use_default_suggestions(self) -> Self {
        self.set_suggestions(SingleSuggestionProvider)
    }

    /// Sets the specified `SuggestionProvider`.
    pub fn set_suggestions(mut self, suggestions: impl SuggestionProvider + 'static) -> Self {
        self.suggestions = Some(Box::new(suggestions));
        self
    }

    /// Executes this command-line app and pass the specified arguments as `&str`.
    ///
    /// This forwards the call to `CommandLine::exec` by slit the `str`.
    pub fn exec_str(&self, args: &str) -> Result<()> {
        self.exec(into_arg_iterator(args))
    }

    /// Executes this command-line app and pass the specified arguments.
    pub fn exec<I: IntoIterator<Item = String>>(&self, args: I) -> Result<()> {
        let mut parser = DefaultParser::default();
        let args = args.into_iter().collect::<Vec<String>>();
        let result = parser.parse(&self.context, args);

        let parse_result = match result {
            Ok(r) => r,
            Err(error) => return self.handle_error(error),
        };

        let command = parse_result.command();

        // Check if the command is a 'help' command and display help
        if self.is_help(&parse_result) {
            let help = self.help.as_ref().unwrap();
            return match help.kind() {
                HelpKind::Subcommand => {
                    self.display_help(parse_result.arg())
                },
                HelpKind::Option => {
                    let args = parse_result
                        .get_option(help.name())
                        .unwrap()
                        .get_arg();

                    self.display_help(args)
                }
            }
        }

        // We borrow the value from the Option to avoid create a temporary
        let handler = command.get_handler();

        if let Some(mut handler) = handler {
            let options = parse_result.options();
            let args = parse_result.args();
            // Calls the handler and pass the arguments
            (*handler)(options, args)
        } else {
            if self.show_help_when_not_handler {
                self.display_help(None)
            } else {
                // todo: panics instead of return error?
                Err(Error::new(
                    ErrorKind::Other,
                    format!("No handler specified for `{}`", command.get_name()),
                ))
            }
        }
    }

    /// Runs this command-line app.
    ///
    /// This is equivalent to `cmd_line.exec(std::env::args().skip(1))`.
    #[inline]
    pub fn run(&self) -> Result<()> {
        self.exec(std::env::args().skip(1))
    }

    fn handle_error(&self, error: Error) -> Result<()> {
        // If is a parse error and `InvalidArgumentCount`
        // we show a message about the usage of the command
        if error.kind() == &ErrorKind::InvalidArgumentCount {
            let message = self.get_message(None, MessageKind::Usage)?;
            return Err(
                Error::new(
                    error.kind().clone(),
                    format!("{}\n{}", error, message)
                )
            );
        }

        if self.suggestions.is_some() {
            // todo: Error is produce here due to trying to read a unrecognized command
            let parse_error = error.try_into_parse_error()?;
            if self.is_help(parse_error.parse_result()) {
                let args = parse_error.parse_result().arg();
                return self.display_help(args);
            }

            return Err(self.display_suggestions(parse_error));
        }

        return Err(error);
    }

    fn is_help(&self, parse_result: &ParseResult) -> bool {
        if let Some(help_provider) = &self.help {
            match help_provider.kind(){
                HelpKind::Subcommand => {
                    help_provider.name() == parse_result.command().get_name()
                },
                HelpKind::Option => {
                    parse_result.options().iter()
                        .any(|s| s.get_name() == help_provider.name())
                }
            }
        } else {
            false
        }
    }

    fn display_help(&self, args: Option<&Argument>) -> Result<()>{
        print!("{}", self.get_message(args, MessageKind::Help)?);
        Ok(())
    }

    fn get_message(&self, args: Option<&Argument>, kind: MessageKind) -> Result<String> {
        let help_provider = self.help.as_ref().expect("help command is not set");

        fn find_command<'a>(root: &'a Command, children: &[String]) -> Result<&'a Command> {
            //debug_assert!(children.len() > 0);

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

        let output = match args {
            None => {
                match kind {
                    MessageKind::Help => {
                        help_provider.help(&self.context, self.context.root())
                    }
                    MessageKind::Usage => {
                        help_provider.usage(&self.context, self.context.root())
                    }
                }
            },
            Some(args) => {
                let root = self.context.root();
                let subcommand = find_command(root, args.get_values())?;
                match kind {
                    MessageKind::Help => {
                        help_provider.help(&self.context, subcommand)
                    }
                    MessageKind::Usage => {
                        help_provider.usage(&self.context, subcommand)
                    }
                }
            }
        };

        Ok(output)
    }

    fn display_suggestions(&self, parse_error: ParseError) -> Error {
        // SAFETY: We check if the method is `Some` before enter
        let provider = self.suggestions.as_ref().unwrap();
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

enum MessageKind{
    Help, Usage
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

impl Debug for CommandLine {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandLine")
            .field("context", &self.context)
            .field("help", &debug_option(&self.help, "HelpCommand"))
            .field("suggestions", &debug_option(&self.suggestions, "SuggestionProvider"))
            .finish()
    }
}

/// Split the given value `&str` into command-line args.
///
/// # Example
/// ```rust
/// use clapi::into_arg_iterator;
///
/// let result = into_arg_iterator("echo \"Hello World\" 123");
/// assert_eq!(
/// vec![
///     "echo".to_string(),
///     "Hello World".to_string(),
///     "123".to_string()],
/// result);
/// ```
#[inline]
#[doc(hidden)]
pub fn into_arg_iterator(value: &str) -> Vec<String> {
    into_arg_iterator_with_quote_escape(value, '\\')
}

/// Split the given value `&str` into command-line args using the default
/// platform quote escape:
/// - `"^"` for windows.
/// - `"\"` for unix and other platforms.
///
/// # Example
/// ```rust
/// use clapi::into_platform_arg_iterator;
///
/// // on windows
/// if cfg!(windows){
///     let result = into_platform_arg_iterator("echo ^\"Hello^\"");
///     assert_eq!(vec!["echo".to_string(), "\"Hello\"".to_string()], result);
/// } else {
///     let result = into_platform_arg_iterator("echo \\\"Hello\\\"");
///     assert_eq!(vec!["echo".to_string(), "\"Hello\"".to_string()], result);
/// }
/// ```
#[inline]
#[doc(hidden)]
pub fn into_platform_arg_iterator(value: &str) -> Vec<String> {
    #[cfg(target_os = "windows")]
    const QUOTE_ESCAPE: char = '^';
    #[cfg(not(target_os = "windows"))]
    const QUOTE_ESCAPE: char = '\\';

    into_arg_iterator_with_quote_escape(value, QUOTE_ESCAPE)
}

/// Split the given value `&str` into command-line args
/// using the specified `quote_escape`.
#[doc(hidden)]
pub fn into_arg_iterator_with_quote_escape(value: &str, quote_escape: char) -> Vec<String> {
    const DOUBLE_QUOTE: char = '"';

    let mut result = Vec::new();
    let mut iterator = value.chars().peekable();
    let mut buffer = String::new();

    while let Some(next_char) = iterator.next() {
        if next_char.is_whitespace() {
            if !buffer.is_empty() {
                result.push(buffer.clone());
                buffer.clear();
            }

            continue;
        }

        if next_char == quote_escape && iterator.peek().contains_some(&DOUBLE_QUOTE) {
            buffer.push(iterator.next().unwrap());
        } else if next_char == DOUBLE_QUOTE {
            while let Some(c) = iterator.peek().cloned() {
                if c == DOUBLE_QUOTE {
                    iterator.next();
                    if !buffer.is_empty() {
                        result.push(buffer.clone());
                        buffer.clear();
                    }
                    break;
                } else {
                    iterator.next();
                    buffer.push(c);
                }
            }
        } else {
            buffer.push(next_char);
        }
    }

    if !buffer.is_empty() {
        result.push(buffer);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_arg_iterator_test1() {
        let args = into_arg_iterator("create file \"hello_world.txt\"");
        assert_eq!("create", args[0]);
        assert_eq!("file", args[1]);
        assert_eq!("hello_world.txt", args[2]);
    }

    #[test]
    fn into_arg_iterator_test2() {
        let args = into_arg_iterator("echo --times 5 \\\"bla\\\"");
        assert_eq!("echo", args[0]);
        assert_eq!("--times", args[1]);
        assert_eq!("5", args[2]);
        assert_eq!("\"bla\"", args[3]);
    }

    #[test]
    fn into_arg_iterator_test3() {
        let args = into_arg_iterator("print\t --times:3 \"hello world\"");
        assert_eq!("print", args[0]);
        assert_eq!("--times:3", args[1]);
        assert_eq!("hello world", args[2]);
    }
}
