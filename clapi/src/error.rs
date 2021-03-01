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
            inner: Custom(
                CustomError::new(
                    kind,
                    error.into(),
                    None
                )
            ),
        }
    }

    /// Returns the `ErrorKind` of this error.
    pub fn kind(&self) -> &ErrorKind {
        match &self.inner {
            Inner::Simple(kind) => kind,
            Inner::Custom(custom) => &custom.kind,
        }
    }

    /// Returns this error with the given message.
    ///
    /// # Example
    /// ```
    /// use clapi::{Error, ErrorKind};
    ///
    /// let error = Error::from(ErrorKind::InvalidArgument("xyz".to_string()));
    /// let new_error = error.with_message("expected a number");
    /// assert_eq!(new_error.to_string(), "invalid value for argument 'xyz': expected a number".to_string())
    /// ```
    pub fn with_message<S: Into<AnyError>>(&self, msg: S) -> Self {
        match &self.inner {
            Simple(kind) => {
                Error::new(kind.clone(), msg.into())
            }
            Custom(custom) => {
                Error {
                    inner: Custom(CustomError::new(
                        custom.kind.clone(),
                        custom.error.to_string().into(),
                        Some(msg.into().to_string())
                    ))
                }
            }
        }
    }

    /// Prints this error in the `stderr` and exit this process with status 1.
    pub fn exit(self) -> ! {
        if matches!(self.kind(), ErrorKind::DisplayHelp(_) | ErrorKind::DisplayVersion(_)) {
            println!("{}", self);
            std::process::exit(0)
        } else {
            eprintln!("Error: {}", self);
            std::process::exit(1)
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
            Inner::Custom(custom) =>  Display::fmt(custom, f)
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
    /// The value passed to the argument is invalid.
    InvalidArgument(String),
    /// Invalid number of arguments being passed.
    InvalidArgumentCount,
    /// The expression is invalid.
    InvalidExpression,
    /// The option wasn't expected in the current context
    UnexpectedOption(String),
    /// The command wasn't expected in the current context
    UnexpectedCommand(String),
    /// The option is required.
    MissingOption(String),
    /// An error no listed.
    Other,

    /// *Not an actual error used for convenience*.
    ///
    /// Display a help message.
    DisplayHelp(String),

    /// *Not an actual error used for convenience*.
    ///
    /// Display a version message.
    DisplayVersion(String),

    /// Indicates to the caller to show a help message. This should not be used as an `Error`.
    #[doc(hidden)]
    FallthroughHelp
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::InvalidArgument(s) => write!(f, "invalid value for argument '{}'", s),
            ErrorKind::InvalidArgumentCount => write!(f, "invalid argument count"),
            ErrorKind::InvalidExpression => write!(f, "invalid expression"),
            ErrorKind::UnexpectedOption(s) => write!(f, "unexpected option: '{}'", s),
            ErrorKind::UnexpectedCommand(s) => write!(f, "unexpected command: '{}'", s),
            ErrorKind::MissingOption(s) => write!(f, "'{}' is required", s),
            ErrorKind::Other => write!(f, "unexpected error"),
            ErrorKind::DisplayHelp(s) => write!(f, "{}", s),
            ErrorKind::DisplayVersion(s) => write!(f, "{}", s),
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
    info: Option<String>
}

impl CustomError {
    pub fn new(kind: ErrorKind, error: AnyError, info: Option<String>) -> Self {
        CustomError {
            kind,
            error,
            info
        }
    }
}

impl Display for CustomError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(info) = &self.info {
            write!(f, "{}: {}\n{}", self.kind, self.error, info)
        } else {
            write!(f, "{}: {}", self.kind, self.error)
        }
    }
}