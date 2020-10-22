use crate::command::Command;
use crate::context::Context;
use crate::error::{Error, ErrorKind, ParseError, Result};
use crate::help::{DefaultHelpCommand, HelpCommand};
use crate::parser::{DefaultParser, Parser};
use crate::root_command::RootCommand;
use crate::suggestion::{SingleSuggestionProvider, SuggestionProvider};
use crate::utils::OptionExt;
use std::fmt::{Debug, Formatter};
use std::ops::DerefMut;

/// Represents a command-line app.
pub struct CommandLine {
    context: Context,
    help: Option<Box<dyn HelpCommand>>,
    suggestions: Option<Box<dyn SuggestionProvider>>,
    show_help_when_not_handler: bool,
}

impl CommandLine {
    /// Constructs a new `CommandLine` with the provided `RootCommand`.
    #[inline]
    pub fn new(root: RootCommand) -> Self {
        CommandLine::with_context(Context::new(root))
    }

    /// Constructs a new `CommandLine` with default values with the provided `RootCommand`.
    #[inline]
    pub fn default_with_root(root: RootCommand) -> Self{
        Self::new(root)
            .use_default_suggestions()
            .use_default_help()
    }

    /// Constructs a new `CommandLine` with default values with the provided `Context`.
    #[inline]
    pub fn default_with_context(context: Context) -> Self{
        Self::with_context(context)
            .use_default_suggestions()
            .use_default_help()
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
    pub fn root(&self) -> &RootCommand {
        &self.context.root()
    }

    /// Returns the `HelpCommand` used by this command-line or `None` if not set.
    pub fn help(&self) -> Option<&Box<dyn HelpCommand>> {
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
        self.set_help(DefaultHelpCommand)
    }

    /// Sets the specified `HelpCommand`.
    pub fn set_help(mut self, help_command: impl HelpCommand + 'static) -> Self {
        assert_eq!(
            self.context.root().get_child(help_command.name()),
            None,
            "Command `{}` already exists",
            help_command.name()
        );

        let command = Command::new(help_command.name())
            .set_description(help_command.description());

        self.context.root_mut().as_mut().add_command(command);
        self.help = Some(Box::new(help_command));
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
            Err(error) => return self.handle_parse_error(error)
        };

        let command = parse_result.command();

        // Check if the command is a 'help' command
        if self.is_help_command(command) {
            return self.display_help(parse_result.args().values());
        }

        let handler = command.handler();

        // We borrow the value from the Option to avoid create temporary
        if let Some(mut handler) = handler {
            let options = parse_result.options();
            let args = parse_result.args();
            handler.deref_mut()(options, args)
        } else {
            if self.show_help_when_not_handler {
                self.display_help(&[])
            } else {
                Err(Error::new(
                    ErrorKind::Unknown,
                    format!("No handler specified for `{}`", command.name()),
                ))
            }
        }
    }

    /// Runs this command-line app.
    ///
    /// This is equivalent to `cmd_line.exec(std::env::args().skip(1))`.
    #[inline]
    pub fn run(&self) -> Result<()>{
        self.exec(std::env::args().skip(1))
    }

    fn handle_parse_error(&self, error: Error) -> Result<()>{
        if *error.kind() == ErrorKind::EmptyExpression && self.help.is_some() {
            return self.display_help(&[]);
        }

        if self.suggestions.is_some() {
            let parse_error = error.try_into_parse_error()?;
            if self.is_help_command(parse_error.command()) {
                return self.display_help(parse_error.command_args().unwrap_or_default());
            }

            return Err(self.display_suggestions(parse_error));
        }

        return Err(error);
    }

    fn is_help_command(&self, command: &Command) -> bool {
        if let Some(help_provider) = &self.help {
            help_provider.name() == command.name()
        } else {
            false
        }
    }

    fn display_help(&self, args: &[String]) -> Result<()> {
        let help_command = self.help.as_ref().expect("help command is not set");

        fn find_command<'a>(root: &'a RootCommand, children: &[String]) -> Result<&'a Command> {
            debug_assert!(children.len() > 0);

            let mut current = root.as_ref();

            for i in 0..children.len() {
                let child_name = children[i].as_str();
                if let Some(cmd) = current.get_child(child_name) {
                    current = cmd;
                } else {
                    return Err(Error::from(ErrorKind::UnrecognizedCommand(
                        child_name.to_string(),
                    )));
                }
            }

            Ok(current)
        }

        let output = match args.len() {
            0 => help_command.help(&self.context, self.context.root().as_ref()),
            _ => {
                let root = self.context.root();
                let subcommand = find_command(root, args)?;
                help_command.help(&self.context, subcommand)
            }
        };

        print!("{}", output);
        Ok(())
    }

    fn display_suggestions(&self, error: ParseError) -> Error {
        let kind = error.kind();
        let message = match kind {
            ErrorKind::UnrecognizedCommand(s) => {
                let command_names = error
                    .command()
                    .children()
                    .map(|c| c.name().to_string())
                    .collect::<Vec<String>>();

                self.suggestions
                    .as_ref()
                    .unwrap()
                    .suggestion_message_for(&s, &command_names)
            }
            ErrorKind::UnrecognizedOption(s) => {
                let option_names = error
                    .command()
                    .options()
                    .iter()
                    .map(|o| o.name().to_string())
                    .collect::<Vec<String>>();

                self.suggestions
                    .as_ref()
                    .unwrap()
                    .suggestion_message_for(&s, &option_names)
            }
            _ => {
                // Forwards the error
                return Error::from(error.kind().clone());
            }
        };

        if let Some(msg) = message {
            Error::new(kind.clone(), msg)
        } else {
            Error::from(kind.clone())
        }
    }
}

impl Debug for CommandLine {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandLine")
            .field("context", &self.context)
            .field(
                "help",
                if self.help.is_some() {
                    &"Some(HelpCommand)"
                } else {
                    &"None"
                },
            )
            .field(
                "suggestions",
                if self.help.is_some() {
                    &"Some(SuggestionProvider)"
                } else {
                    &"None"
                },
            )
            .finish()
    }
}

impl From<Context> for CommandLine {
    fn from(context: Context) -> Self {
        CommandLine::with_context(context)
    }
}

impl From<RootCommand> for CommandLine {
    fn from(root: RootCommand) -> Self {
        CommandLine::new(root)
    }
}

impl From<Command> for CommandLine {
    fn from(command: Command) -> Self {
        CommandLine::new(RootCommand::from(command))
    }
}

/// Split the given value `&str` into command-line args.
///
/// # Example
/// ```rust
/// use clapi::command_line::into_arg_iterator;
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
/// use clapi::command_line::into_platform_arg_iterator;
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
pub fn into_platform_arg_iterator(value: &str) -> Vec<String> {
    #[cfg(target_os = "windows")]
    const QUOTE_ESCAPE: char = '^';
    #[cfg(not(target_os = "windows"))]
    const QUOTE_ESCAPE: char = '\\';

    into_arg_iterator_with_quote_escape(value, QUOTE_ESCAPE)
}

/// Split the given value `&str` into command-line args
/// using the specified `quote_escape`.
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
}
