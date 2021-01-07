#![allow(clippy::len_zero)]
use std::fmt::{Write, Display, Formatter};
use crate::{Context, Command, CommandOption, Argument};
use crate::utils::Then;

/// A trait for provide help information about a `Command`.
pub trait Help {
    /// Provides help information about the command like:
    /// name, description, options, subcommands and usage
    fn help(&self, buf: &mut Buffer, context: &Context, command: &Command) -> std::fmt::Result;

    /// Provides information about the usage of the command.
    ///
    /// By default this delegates the call to `Help::help`.
    fn usage(&self, buf: &mut Buffer, context: &Context, command: &Command) -> std::fmt::Result {
        self.help(buf, context, command)
    }

    /// Type of the `HelpProvider`, the default is `HelpKind::Any`.
    fn kind(&self) -> HelpKind {
        HelpKind::Any
    }

    /// Returns the name of this help command, the default is: `help`.
    #[inline]
    fn name(&self) -> &str {
        "help"
    }

    /// Returns the alias of the help command, the default is: `None`.
    #[inline]
    fn alias(&self) -> Option<&str>{
        None
    }

    /// Returns the description of this help command.
    #[inline]
    fn description(&self) -> &str {
        "Provides information about a command"
    }
}

/// Type of the `HelpProvider`.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum HelpKind {
    /// The help is a command, for example:
    ///
    /// `command help [args]`.
    Subcommand,
    /// The help is an option, for example:
    ///
    /// `command --help`.
    Option,
    /// The help is both a command or option.
    Any
}

/// A buffer of bytes to write to.
#[derive(Default, Debug, Clone)]
pub struct Buffer {
    buffer: Vec<u8>
}

impl Buffer {
    /// Constructs a new `Buffer`.
    pub fn new() -> Self {
        Buffer {
            buffer: Default::default()
        }
    }

    /// Constructs a new `Buffer` with the specified initial capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Buffer {
            buffer: Vec::with_capacity(capacity)
        }
    }

    /// Reserve the specified amount of bytes in this buffer.
    pub fn reserve(&mut self, additional: usize) {
        self.buffer.reserve(additional)
    }

    /// Returns a reference to the buffer.
    pub fn buffer(&self) -> &Vec<u8>{
        &self.buffer
    }

    /// Returns a mutable reference to the buffer.
    pub fn buffer_mut(&mut self) -> &mut Vec<u8>{
        &mut self.buffer
    }
}

impl Write for Buffer {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.buffer.extend_from_slice(s.as_bytes());
        Ok(())
    }
}

impl Display for Buffer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", std::str::from_utf8(&self.buffer).unwrap())
    }
}

// todo: Add additional config for uppercase `args`, indents
/// A default implementation of the `Help` trait.
pub struct DefaultHelp(pub HelpKind);

impl Default for DefaultHelp {
    fn default() -> Self {
        DefaultHelp(HelpKind::Subcommand)
    }
}

impl Help for DefaultHelp {
    fn help(&self, buf: &mut Buffer, context: &Context, command: &Command) -> std::fmt::Result {
        // Command name
        writeln!(buf, "{}", command.get_name())?;

        // Command description
        if let Some(description) = command.get_description() {
            write_indent(buf);
            writeln!(buf, "{}", description)?;
        }

        // Command usage
        if command.take_args() || command.get_options().len() > 0 || command.get_children().len() > 0 {
            // We check again for args, options and children to add a newline
            writeln!(buf)?;
            self.usage(buf, context, command)?;
        }

        // Options
        if command.get_options().len() > 0 {
            writeln!(buf)?;
            writeln!(buf, "OPTIONS:")?;

            for option in command.get_options() {
                write_indent(buf);
                writeln!(buf, "{}", option_to_string(context, option))?;
            }
        }

        // Subcommands
        if command.get_children().len() > 0 {
            writeln!(buf)?;
            writeln!(buf, "SUBCOMMANDS:")?;

            for command in command.get_children() {
                write_indent(buf);
                writeln!(buf, "{}", command_to_string(command))?;
            }
        }

        if let Some(about) = command.get_about() {
            writeln!(buf)?;
            writeln!(buf, "{}", about)?;
        }

        // Help usage message
        if let Some(msg) = use_help_for_more_info_msg(context) {
            writeln!(buf)?;
            writeln!(buf, "{}", msg)?;
        }

        Ok(())
    }

    fn usage(&self, buf: &mut Buffer, _: &Context, command: &Command) -> std::fmt::Result {
        if command.take_args() || command.get_options().len() > 0 || command.get_children().len() > 0 {
            writeln!(buf, "USAGE:")?;
            // command [OPTIONS] [ARGS]...
            {
                write_indent(buf);
                write!(buf, "{}", command.get_name())?;

                if command.get_options().len() > 1 {
                    if command.get_options().len() == 1 {
                        write!(buf, " [OPTION]")?;
                    } else {
                        write!(buf, " [OPTIONS]")?;
                    }
                }

                for arg in command.get_args() {
                    let arg_name = arg.get_name().to_uppercase();
                    if arg.get_value_count().max() > 1 {
                        write!(buf, " [{}]...", arg_name)?;
                    } else {
                        write!(buf, " [{}] ", arg_name)?;
                    }
                }

                writeln!(buf)?;
            }

            // command [SUBCOMMAND] [OPTIONS] [ARGS]...
            if command.get_children().len() > 0 {
                write_indent(buf);
                write!(buf, "{} [SUBCOMMAND]", command.get_name())?;

                if command.get_children().any(|c| c.get_options().len() > 0) {
                    write!(buf, " [OPTIONS]")?;
                }

                if command.get_children().any(|c| c.take_args()) {
                    write!(buf, " [ARGS]")?;
                }

                writeln!(buf)?;
            }
        }

        Ok(())
    }

    fn kind(&self) -> HelpKind {
        self.0
    }
}

// Add indentation to the buffer
fn write_indent(buf: &mut Buffer) {
    // 3 spaces
    write!(buf, "   ").unwrap()
}

// -v, --version        Shows the version
fn option_to_string(context: &Context, option: &CommandOption) -> String {
    let names = if let Some(alias) = option.get_aliases().next() {
        let alias_prefix = context.alias_prefixes().next().unwrap();
        let name_prefix = context.name_prefixes().next().unwrap();
        format!("{}{}, {}{}", alias_prefix, alias, name_prefix, option.get_name())
    } else {
        let name_prefix = context.name_prefixes().next().unwrap();
        // Normally there is 4 spaces if the `alias prefix` and `name` is 1 char
        format!("    {}{}", name_prefix, option.get_name())
    };

    if let Some(description) = option.get_description() {
        format!("{:25} {}", names, description)
    } else {
        names
    }
}

// version              Shows the version
fn command_to_string(command: &Command) -> String {
    if let Some(description) = command.get_description() {
        format!("{:25} {}", command.get_name(), description)
    } else {
        command.get_name().to_owned()
    }
}

// Use '' for see more information about a command
pub(crate) fn use_help_for_more_info_msg(context: &Context) -> Option<String> {
    if let Some(help) = context.help() {
        match help.kind() {
            HelpKind::Any | HelpKind::Subcommand => {
                let command = context.root().get_name();
                Some(format!("Use '{} {} <subcommand>' for more information about a command.", command, help.name()))
            }
            HelpKind::Option => {
                // SAFETY: `name_prefixes` is never empty
                let prefix = context.name_prefixes().next().unwrap();
                let command = context.root().get_name();
                Some(format!("Use '{} <subcommand> {}{}' for more information about a command.", command, prefix, help.name()))
            }
        }
    } else {
        None
    }
}

pub(crate) fn to_command<H: Help + ?Sized>(help: &H) -> Command {
    Command::new(help.name())
        .arg(Argument::new("subcommand").value_count(0..=1))
        .description(help.description())
}

pub(crate) fn to_option<H: Help + ?Sized>(help: &H) -> CommandOption {
    CommandOption::new(help.name())
        .arg(Argument::new("subcommand").value_count(0..=1))
        .description(help.description())
        .then_apply(|opt| {
            if let Some(alias) = help.alias() {
                opt.alias(alias)
            } else {
                opt
            }
        })
}