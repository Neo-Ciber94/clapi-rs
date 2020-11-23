use crate::command::Command;
use crate::error::Inner::{Custom, Parsed, Simple};
use crate::option::CommandOption;
use std::fmt::{Debug, Display, Formatter};
use std::result;
use crate::ArgumentList;

/// A convenient `Result` type.
pub type Result<T> = std::result::Result<T, Error>;

type AnyError = Box<dyn std::error::Error + Sync + Send>;

/// An error in a command-line operation.
pub struct Error {
    inner: Inner,
}

enum Inner {
    Simple(ErrorKind),
    Custom(CustomError),
    Parsed(Box<ParseError>),
}

impl Error {
    /// Constructs a new `Error` with the specified `ErrorKind` and extended error information.
    ///
    /// # Example
    /// ```rust
    /// use clapi::{Error, ErrorKind};
    ///
    /// let error = Error::new(ErrorKind::InvalidArgumentCount, "expect 1 or more arguments");
    /// assert!(matches!(error.kind(), ErrorKind::InvalidArgumentCount));
    /// ```
    pub fn new<E: Into<AnyError>>(kind: ErrorKind, error: E) -> Self {
        Error {
            inner: Custom(CustomError {
                kind,
                error: error.into(),
            }),
        }
    }

    /// Constructs a new parse error.
    ///
    /// # Parameters
    /// - `inner`: the inner error.
    /// - `command`: the command or parent command where the error occurred.
    /// - `option`: the option where the error occurred.
    /// - `args`: the args being passed to the command or option, if the `option` is not set
    /// the args will be considered part of the command.
    pub fn new_parse_error(
        inner: Error,
        command: Command,
        option: Option<CommandOption>,
        args: Option<ArgumentList>,
    ) -> Self {
        Error {
            inner: Parsed(Box::new(ParseError::new(inner, command, option, args))),
        }
    }

    /// Returns the `ErrorKind` of this error.
    pub fn kind(&self) -> &ErrorKind {
        match &self.inner {
            Inner::Simple(kind) => kind,
            Inner::Parsed(error) => &error.kind(),
            Inner::Custom(custom) => &custom.kind,
        }
    }

    /// Returns `true` if this is a `ParseError`.
    pub fn is_parse_error(&self) -> bool {
        match self.inner {
            Inner::Parsed(_) => true,
            _ => false,
        }
    }

    /// Try converts this `Error` into a `ParseError`.
    ///
    /// # Returns
    /// - `Ok(ParseError)` if this is a parse error.
    /// - `Err(Self)` if this is not a parse error.
    pub fn try_into_parse_error(self) -> result::Result<ParseError, Error> {
        match self.inner {
            Inner::Parsed(error) => Ok(*error),
            _ => Err(self),
        }
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            Inner::Simple(kind) => Display::fmt(kind, f),
            Inner::Parsed(error) => Display::fmt(error.inner(), f),
            Inner::Custom(custom) => {
                if matches!(custom.kind, ErrorKind::Other) {
                    write!(f, "{}", custom.error)
                } else {
                    write!(f, "{}: {}", custom.kind, custom.error)
                }
            },
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error {
            inner: Simple(kind),
        }
    }
}

impl From<ParseError> for Error {
    fn from(parse_error: ParseError) -> Self {
        Error { inner: Inner::Parsed(Box::new(parse_error)) }
    }
}

/// Types of errors.
#[derive(Clone, Eq, PartialEq)]
pub enum ErrorKind {
    /// The argument is invalid.
    InvalidArgument(String),
    /// Invalid number of arguments being passed.
    InvalidArgumentCount,
    /// The expression is invalid.
    InvalidExpression,
    /// The expression is empty.
    EmptyExpression,
    /// The option is not found in the command.
    UnrecognizedOption(String, String),
    /// The command is not found in the parent.
    UnrecognizedCommand(String),
    /// The option is required.
    MissingOption(String),
    /// An error no listed.
    Other,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::InvalidArgument(s) => write!(f, "invalid argument: '{}'", s),
            ErrorKind::InvalidArgumentCount => write!(f, "invalid argument count"),
            ErrorKind::InvalidExpression => write!(f, "invalid expression"),
            ErrorKind::EmptyExpression => write!(f, "empty expression"),
            ErrorKind::UnrecognizedOption(p, s) => write!(f, "unrecognized option: '{}{}'", p, s),
            ErrorKind::UnrecognizedCommand(s) => write!(f, "unrecognized command: '{}'", s),
            ErrorKind::MissingOption(s) => write!(f, "'{}' is required", s),
            ErrorKind::Other => write!(f, "unknown error"),
        }
    }
}

impl Debug for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

struct CustomError {
    kind: ErrorKind,
    error: AnyError,
}

/// Represents an error occurred in a parse operation.
pub struct ParseError {
    inner: Error,
    command: Command,
    option: Option<CommandOption>,
    args: Option<ArgumentList>,
}

impl ParseError {
    fn new(
        inner: Error,
        command: Command,
        option: Option<CommandOption>,
        args: Option<ArgumentList>,
    ) -> Self {
        ParseError {
            inner,
            command,
            option,
            args,
        }
    }

    /// Returns the inner error.
    pub fn inner(&self) -> &Error {
        &self.inner
    }

    /// Returns the type of the error.
    pub fn kind(&self) -> &ErrorKind {
        &self.inner.kind()
    }

    /// Returns the `Command` where the error occurred.
    pub fn command(&self) -> &Command {
        &self.command
    }

    /// Returns the `CommandOption` where the error occurred.
    pub fn option(&self) -> Option<&CommandOption> {
        self.option.as_ref()
    }

    /// Returns the argument values of the option if any.
    pub fn option_args(&self) -> Option<&ArgumentList> {
        if self.option.is_some() {
            self.args.as_ref()
        } else {
            None
        }
    }

    /// Returns the argument values of the command if any.
    pub fn command_args(&self) -> Option<&ArgumentList> {
        if self.option.is_none() {
            self.args.as_ref()
        } else {
            None
        }
    }
}
