
/// Represents a command-line symbol.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Symbol {
    /// A command symbol.
    Command(String),
    /// A option symbol.
    Option(String),
}

impl Symbol {
    /// Gets the name of the symbol.
    pub fn name(&self) -> &str {
        match self {
            Symbol::Command(s) | Symbol::Option(s) => s,
        }
    }

    /// Checks if this symbol is a command.
    pub fn is_command(&self) -> bool {
        match self {
            Symbol::Command(_) => true,
            _ => false,
        }
    }

    /// Checks if this symbol is an option.
    pub fn is_option(&self) -> bool {
        match self {
            Symbol::Option(_) => true,
            _ => false,
        }
    }
}
