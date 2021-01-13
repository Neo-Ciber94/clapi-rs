#![allow(clippy::len_zero)]
use std::fmt::{Write, Display, Formatter};
use crate::{Context, Command, CommandOption, Argument, OptionList};
use crate::utils::Then;

/// A trait for provide help information about a `Command`.
pub trait Help {
    /// Provides help information about the command like:
    /// name, description, options, subcommands and usage
    fn help(&self, buf: &mut Buffer, context: &Context, command: &Command);

    /// Provides information about the usage of the command.
    ///
    /// By default this delegates the call to `Help::help`.
    fn usage(&self, buf: &mut Buffer, context: &Context, command: &Command) {
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

/// A default implementation of the `Help` trait.
#[derive(Debug, Clone)]
pub struct DefaultHelp<'a> {
    indent: &'a [u8],
    kind: HelpKind,
    letter_case: LetterCase,
    help_message_source: MessageSource,
    usage_message_source: MessageSource,
    show_after_help_message: bool,
    after_help_message: Option<&'a str>,
}

impl<'a> Default for DefaultHelp<'a>{
    #[inline]
    fn default() -> Self {
        DefaultHelp::new()
    }
}

impl<'a> DefaultHelp<'a> {
    #[inline]
    pub const fn new() -> Self {
        Self::with_kind(HelpKind::Any)
    }

    #[inline]
    pub const fn with_kind(kind: HelpKind) -> Self {
        Self::with_indent(kind, "   ".as_bytes())
    }

    #[inline]
    pub const fn with_indent(kind: HelpKind, indent: &'a [u8]) -> Self {
        DefaultHelp {
            indent,
            kind,
            letter_case: LetterCase::Upper,
            help_message_source: MessageSource::Overwrite,
            usage_message_source: MessageSource::Overwrite,
            show_after_help_message: true,
            after_help_message: None,
        }
    }

    pub const fn letter_case(mut self, letter_case: LetterCase) -> Self {
        self.letter_case = letter_case;
        self
    }

    pub const fn help_message_source(mut self, help_message_source: MessageSource) -> Self {
        self.help_message_source = help_message_source;
        self
    }

    pub const fn usage_message_source(mut self, usage_message_source: MessageSource) -> Self {
        self.usage_message_source = usage_message_source;
        self
    }

    pub const fn after_help_message(mut self, after_help_message: &'a str) -> Self {
        self.after_help_message = Some(after_help_message);
        self
    }

    pub const fn show_after_help_message(mut self, show_after_help_message: bool) -> Self {
        self.show_after_help_message = show_after_help_message;
        self
    }
}

impl<'a> Help for DefaultHelp<'a> {
    fn help(&self, buf: &mut Buffer, context: &Context, command: &Command) {
        match self.help_message_source {
            source @ MessageSource::UseCommand |
            source @ MessageSource::Overwrite => {
                if let Some(help) = command.get_help() {
                    buf.write_str(help).unwrap();
                }

                if source == MessageSource::UseCommand {
                    return;
                }
            }
            MessageSource::UseHelp => {}
        }

        // Help letter case
        let letter_case = self.letter_case;

        // Command name
        writeln!(buf, "{}", command.get_name()).unwrap();

        // Command description
        if let Some(description) = command.get_description() {
            write_indent(buf, self.indent);
            writeln!(buf, "{}", description).unwrap();
        }

        // Number of no-hidden options and subcommands
        let option_count = count_options(command.get_options());
        let subcommand_count = count_subcommands(&command);

        // Command usage
        if command.take_args() || subcommand_count > 0 || option_count > 0 {
            // We check again for args, options and children to add a newline
            writeln!(buf).unwrap();
            self.usage(buf, context, command);
        }

        // Options
        if option_count > 0 {
            writeln!(buf).unwrap();
            writeln!(buf, "{}:", letter_case.format("OPTIONS")).unwrap();

            for option in command.get_options().iter().filter(|o| !o.is_hidden()) {
                write_indent(buf, self.indent);
                writeln!(buf, "{}", option_to_string(context, option)).unwrap();
            }
        }

        // Subcommands
        if subcommand_count > 0 {
            writeln!(buf).unwrap();
            writeln!(buf, "{}:", letter_case.format("SUBCOMMANDS")).unwrap();

            for command in command.get_children().filter(|c| !c.is_hidden()) {
                write_indent(buf, self.indent);
                writeln!(buf, "{}", command_to_string(command)).unwrap();
            }
        }

        if let Some(about) = command.get_usage() {
            writeln!(buf).unwrap();
            writeln!(buf, "{}", about).unwrap();
        }

        // Help usage message
        if self.show_after_help_message {
            match self.after_help_message {
                Some(msg) => {
                    writeln!(buf).unwrap();
                    writeln!(buf, "{}", msg).unwrap();
                }
                None => {
                    if let Some(msg) = after_help_message(context) {
                        writeln!(buf).unwrap();
                        writeln!(buf, "{}", msg).unwrap();
                    }
                }
            }
        }
    }

    fn usage(&self, buf: &mut Buffer, _: &Context, command: &Command) {
        match self.usage_message_source {
            source @ MessageSource::UseCommand |
            source @ MessageSource::Overwrite => {
                if let Some(usage) = command.get_usage() {
                    buf.write_str(usage).unwrap();
                }

                if source == MessageSource::UseCommand {
                    return;
                }
            }
            MessageSource::UseHelp => {}
        }

        // Number of no-hidden options and subcommands
        let option_count = count_options(command.get_options());
        let subcommand_count = count_subcommands(&command);
        let letter_case = self.letter_case;

        if command.take_args() || subcommand_count > 0 || option_count > 0 {
            writeln!(buf, "{}:", self.letter_case.format("USAGE")).unwrap();
            // command [OPTIONS] [ARGS]...
            {
                write_indent(buf, self.indent);
                write!(buf, "{}", command.get_name()).unwrap();

                if option_count > 1 {
                    if option_count == 1 {
                        write!(buf, " [{}]", letter_case.format("OPTION")).unwrap();
                    } else {
                        write!(buf, " [{}]", letter_case.format("OPTIONS")).unwrap();
                    }
                }

                for arg in command.get_args() {
                    if arg.get_value_count().max() > 1 {
                        write!(buf, " [{}]...", letter_case.format(arg.get_name())).unwrap();
                    } else {
                        write!(buf, " [{}] ", letter_case.format(arg.get_name())).unwrap();
                    }
                }

                writeln!(buf).unwrap();
            }

            // command [SUBCOMMAND] [OPTIONS] [ARGS]...
            if subcommand_count > 0 {
                write_indent(buf, self.indent);
                write!(buf, "{} [{}]", command.get_name(), letter_case.format("SUBCOMMAND")).unwrap();

                if command.get_children().any(|c| count_options(c.get_options()) > 0) {
                    write!(buf, " [{}]", letter_case.format("OPTIONS")).unwrap();
                }

                if command.get_children()
                    .filter(|c| !c.is_hidden())
                    .any(|c| c.take_args()) {
                    write!(buf, " [{}]", letter_case.format("ARGS")).unwrap();
                }

                writeln!(buf).unwrap();
            }
        }
    }

    fn kind(&self) -> HelpKind {
        self.kind
    }
}

/// Represents the source of a `usage` or `help` message.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MessageSource {
    /// Use the command as the source of the message.
    UseCommand,
    /// Use the `Help` implementation message.
    UseHelp,
    /// Use the command message if available otherwise the `Help`.
    Overwrite
}

/// Represents the letter case of a `String`.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LetterCase {
    /// Uppercase
    Upper,
    /// Lowercase
    Lower,
    /// No change case
    None
}

impl LetterCase {
    pub fn format(&self, s: &str) -> String {
        match self {
            LetterCase::Upper => s.to_uppercase(),
            LetterCase::Lower => s.to_lowercase(),
            LetterCase::None => s.to_owned()
        }
    }
}

// Number of no-hidden options
fn count_options(options: &OptionList) -> usize {
    options.iter()
        .filter(|opt| !opt.is_hidden())
        .count()
}

// Number of no-hidden subcommands
fn count_subcommands(parent: &Command) -> usize {
    parent.get_children()
        .filter(|c| !c.is_hidden())
        .count()
}

// Add indentation to the buffer
fn write_indent(buf: &mut Buffer, indent: &[u8]) {
    buf.buffer_mut().extend_from_slice(indent);
    // 3 spaces
    //write!(buf, "{}", std::str::from_utf8(indent).unwrap()).unwrap()
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
        //format!("    {}{}", name_prefix, option.get_name())
        format!("{}{:4}", name_prefix, option.get_name())
    };

    if let Some(description) = option.get_description() {
        format!("{:20} {}", names, description)
    } else {
        names
    }
}

// version              Shows the version
fn command_to_string(command: &Command) -> String {
    if let Some(description) = command.get_description() {
        format!("{:20} {}", command.get_name(), description)
    } else {
        command.get_name().to_owned()
    }
}

// Use '' for see more information about a command
pub(crate) fn after_help_message(context: &Context) -> Option<String> {
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
        .arg(Argument::with_name("subcommand").value_count(0..=1))
        .hidden(true)
        .description(help.description())
}

pub(crate) fn to_option<H: Help + ?Sized>(help: &H) -> CommandOption {
    CommandOption::new(help.name())
        .arg(Argument::with_name("subcommand").value_count(0..=1))
        .description(help.description())
        .hidden(true)
        .then_apply(|opt| {
            if let Some(alias) = help.alias() {
                opt.alias(alias)
            } else {
                opt
            }
        })
}