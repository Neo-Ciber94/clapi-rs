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
        matches!(self, Symbol::Cmd(_))
    }

    /// Checks if this symbol is an option.
    pub fn is_option(&self) -> bool {
        matches!(self, Symbol::Opt(_))
    }
}
