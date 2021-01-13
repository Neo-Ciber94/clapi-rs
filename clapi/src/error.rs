use crate::error::Inner::{Custom, Simple};
use std::fmt::{Debug, Display, Formatter};

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

    /// Returns the `ErrorKind` of this error.
    pub fn kind(&self) -> &ErrorKind {
        match &self.inner {
            Inner::Simple(kind) => kind,
            Inner::Custom(custom) => &custom.kind,
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self.inner {
            Simple(_) => None,
            Custom(ref custom) => Some(custom.error.as_ref()),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            Inner::Simple(kind) => Display::fmt(kind, f),
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
    /// Indicates to the caller to show a help message. This should not be used as an `Error`.
    #[doc(hidden)]
    FallthroughHelp
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
            ErrorKind::FallthroughHelp => panic!("`ErrorKind::FallthroughHelp` should not be used as an error")
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