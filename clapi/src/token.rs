use core::fmt::{Display, Formatter};

/// Represents the end of the options
pub const END_OF_OPTIONS: &str = "--";

/// Represents a command-line token.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Token {
    /// A command
    Cmd(String),
    /// A prefixed option
    Opt(String),
    /// An argument
    Arg(String),
    /// End of options
    EOO,
    /// Option assign operator
    AssignOp(char)
}

impl Token {
    /// Returns `true` if the token is a command.
    pub fn is_command(&self) -> bool {
        matches!(self, Token::Cmd(_))
    }

    /// Returns `true` if the token is an option.
    pub fn is_option(&self) -> bool {
        matches!(self, Token::Opt(_))
    }

    /// Returns `true` if the token is an argument.
    pub fn is_arg(&self) -> bool {
        matches!(self, Token::Arg(_))
    }

    /// Returns `true` if the token represents an `end of options`.
    pub fn is_eoo(&self) -> bool {
        matches!(self, Token::EOO)
    }

    /// Returns `true` if the token represents an assign operator.
    pub fn is_assign_op(&self) -> bool {
        matches!(self, Token::AssignOp(_))
    }

    /// Returns a `String` representation of this `Token`.
    pub fn into_string(self) -> String {
        match self {
            Token::Cmd(s) => s,
            Token::Opt(s) => s,
            Token::Arg(s) => s,
            Token::EOO => String::from(END_OF_OPTIONS),
            Token::AssignOp(c) => c.to_string(),
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Cmd(name) => write!(f, "{}", name),
            Token::Opt(name) => write!(f, "{}", name),
            Token::Arg(name) => write!(f, "{}", name),
            Token::EOO => write!(f, "{}", END_OF_OPTIONS),
            Token::AssignOp(c) => write!(f, "{}", c),
        }
    }
}
