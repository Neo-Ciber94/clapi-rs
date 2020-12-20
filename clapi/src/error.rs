use crate::command::Command;
use crate::error::Inner::{Custom, Parsed, Simple};
use crate::{ArgumentList, OptionList, ParseResult, Argument};
use std::fmt::{Debug, Display, Formatter};
use std::result;

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
    /// - `parse_result`: the current result of the parse operation.
    pub fn new_parse_error(inner: Error, parse_result: ParseResult) -> Self {
        Error {
            inner: Parsed(Box::new(ParseError::new(inner, parse_result))),
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
            Inner::Parsed(error) => Display::fmt(error.error(), f),
            Inner::Custom(custom) => {
                if matches!(custom.kind, ErrorKind::Other) {
                    write!(f, "{}", custom.error)
                } else {
                    write!(f, "{}: {}", custom.kind, custom.error)
                }
            }
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
        Error {
            inner: Inner::Parsed(Box::new(parse_error)),
        }
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
    parse_result: ParseResult,
    inner: Error,
}

impl ParseError {
    fn new(inner: Error, parse_result: ParseResult) -> Self {
        ParseError {
            parse_result,
            inner,
        }
    }

    /// Returns the inner error.
    pub fn error(&self) -> &Error {
        &self.inner
    }

    /// Returns the type of the error.
    pub fn kind(&self) -> &ErrorKind {
        &self.inner.kind()
    }

    /// Returns the `ParseResult` before this error.
    pub fn parse_result(&self) -> &ParseResult {
        &self.parse_result
    }

    /// Returns the `Command` where the error occurred.
    pub fn command(&self) -> &Command {
        &self.parse_result.command()
    }

    /// Returns the `OptionList`s of the executing command.
    pub fn options(&self) -> &OptionList {
        self.parse_result.options()
    }

    /// Returns the `Argument` of this error if any or `None` if there is more than 1 argument.
    pub fn arg(&self) -> Option<&Argument>{
        self.parse_result.arg()
    }

    /// Returns the arguments values of the command.
    pub fn args(&self) -> &ArgumentList {
        self.parse_result.args()
    }
}
