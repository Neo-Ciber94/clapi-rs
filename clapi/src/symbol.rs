/// Represents a command-line symbol.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Symbol {
    /// A command symbol.
    Cmd(String),
    /// A option symbol.
    Opt(String),
}

impl Symbol {
    /// Gets the name of the symbol.
    pub fn name(&self) -> &str {
        match self {
            Symbol::Cmd(s) | Symbol::Opt(s) => s,
        }
    }

    /// Checks if this symbol is a command.
    pub fn is_command(&self) -> bool {
        match self {
            Symbol::Cmd(_) => true,
            _ => false,
        }
    }

    /// Checks if this symbol is an option.
    pub fn is_option(&self) -> bool {
        match self {
            Symbol::Opt(_) => true,
            _ => false,
        }
    }
}
