use crate::{Command, Context, OptionList};
use std::fmt::Write;
use self::utils::*;

// Indentation used to write the help messages
const INDENT: &str = "   ";

// Provides a help message for the command
#[doc(hidden)]
pub fn command_help(buf: &mut String, context: &Context, command: &Command) {
    // If the command have a `help` message use that instead
    if let Some(msg) = command.get_help() {
        buf.push_str(msg);
        return;
    }

    // Command name
    writeln!(buf, "{}", command.get_name()).unwrap();

    // Command description
    if let Some(description) = command.get_description() {
        write_indent(buf);
        writeln!(buf, "{}", description).unwrap();
    }

    // Number of no-hidden options and subcommands
    let option_count = count_options(command.get_options());
    let subcommand_count = count_subcommands(&command);

    // Command usage
    // Write into the buffer the command usage
    command_usage(buf, context, command, false);

    // Command Options
    if option_count > 0 {
        writeln!(buf).unwrap();
        writeln!(buf, "OPTIONS:").unwrap();

        let width = calculate_required_options_width(context, command, true);
        for option in command.get_options().iter().filter(|o| !o.is_hidden()) {
            write_indent(buf);
            if width > MAX_WIDTH {
                writeln!(
                    buf,
                    "{}",
                    option_to_string(context, option, Align::Column, true)
                )
                .unwrap();
            } else {
                writeln!(
                    buf,
                    "{}",
                    option_to_string(context, option, Align::Row(width), true)
                )
                .unwrap();
            }
        }

        // Remove the last newline of the column
        if width > MAX_WIDTH {
            buf.pop();
        }
    }

    // Command Subcommands
    if subcommand_count > 0 {
        writeln!(buf).unwrap();
        writeln!(buf, "SUBCOMMANDS:").unwrap();

        let width = calculate_required_subcommands_width(command);

        for command in command.get_subcommands().filter(|c| !c.is_hidden()) {
            write_indent(buf);
            if width > MAX_WIDTH {
                writeln!(buf, "{}", command_to_string(command, Align::Column)).unwrap();
            } else {
                writeln!(buf, "{}", command_to_string(command, Align::Row(width))).unwrap();
            }
        }

        // Remove the last newline of the column
        if width > MAX_WIDTH {
            buf.pop();
        }
    }

    if let Some(msg) = get_after_help_message(context) {
        writeln!(buf).unwrap();
        writeln!(buf, "{}", msg).unwrap();
    }
}

// Provides a usage message for the command
#[doc(hidden)]
pub fn command_usage(
    buf: &mut String,
    context: &Context,
    command: &Command,
    after_help_message: bool,
) {
    // If the command have a `usage` message use that instead
    if let Some(msg) = command.get_usage() {
        buf.push_str(msg);
        return;
    }

    // Writes the usage from the `Command` if any
    if let Some(usage) = command.get_usage() {
        writeln!(buf).unwrap();
        writeln!(buf, "USAGE:").unwrap();
        buf.write_str(usage).unwrap();
        return;
    }

    // Number of no-hidden options and subcommands
    let option_count = count_options(command.get_options());
    let subcommand_count = count_subcommands(&command);

    if command.take_args() || subcommand_count > 0 || option_count > 0 {
        writeln!(buf).unwrap();
        writeln!(buf, "USAGE:").unwrap();

        // command [OPTIONS] [ARGS]...
        if command.take_args() || option_count > 0 {
            write_indent(buf);
            write!(buf, "{}", command.get_name()).unwrap();

            if option_count > 1 {
                if option_count == 1 {
                    write!(buf, " [OPTION]").unwrap();
                } else {
                    write!(buf, " [OPTIONS]").unwrap();
                }
            }

            for arg in command.get_args() {
                let arg_name = arg.get_name().to_uppercase();
                if arg.get_values_count().max_or_default() > 1 {
                    write!(buf, " [{}]...", arg_name).unwrap();
                } else {
                    write!(buf, " [{}] ", arg_name).unwrap();
                }
            }

            writeln!(buf).unwrap();
        }

        // command [SUBCOMMAND] [OPTIONS] [ARGS]...
        if subcommand_count > 0 {
            write_indent(buf);
            write!(buf, "{} [SUBCOMMAND]", command.get_name()).unwrap();

            if command
                .get_subcommands()
                .any(|c| count_options(c.get_options()) > 0)
            {
                write!(buf, " [OPTIONS]").unwrap();
            }

            if command
                .get_subcommands()
                .filter(|c| !c.is_hidden())
                .any(|c| c.take_args())
            {
                write!(buf, " [ARGS]").unwrap();
            }

            writeln!(buf).unwrap();
        }
    }

    if after_help_message {
        // After help message
        if let Some(msg) = get_after_help_message(context) {
            writeln!(buf).unwrap();
            writeln!(buf, "{}", msg).unwrap();
        }
    }
}

// Use '' for see more information about a command
pub(crate) fn get_after_help_message(context: &Context) -> Option<String> {
    if context.help_command().is_some() {
        let command = context.root().get_name();
        let help_command = context.help_command().unwrap();
        Some(format!(
            "Use '{} {} <subcommand>' for more information about a command.",
            command,
            help_command.get_name()
        ))
    } else if context.help_option().is_some() {
        // SAFETY: `name_prefixes` is never empty
        let prefix = context.name_prefixes().next().unwrap();
        let command = context.root().get_name();
        let help_option = context.help_option().unwrap();
        Some(format!(
            "Use '{} <subcommand> {}{}' for more information about a command.",
            command,
            prefix,
            help_option.get_name()
        ))
    } else {
        None
    }
}

// Add indentation to the buffer
#[inline]
fn write_indent(buf: &mut String) {
    buf.push_str(INDENT)
}

// Number of no-hidden options
fn count_options(options: &OptionList) -> usize {
    options.iter().filter(|opt| !opt.is_hidden()).count()
}

// Number of no-hidden subcommands
fn count_subcommands(parent: &Command) -> usize {
    parent.get_subcommands().filter(|c| !c.is_hidden()).count()
}

// Utilities for formatting command, options and args
#[doc(hidden)]
pub mod utils {
    use crate::{ArgumentList, Command, CommandOption, Context};
    use std::cmp;

    // Min width of the name
    pub const MIN_WIDTH: usize = 13;

    // Max width of the name, if the name exceed this will display as a column
    pub const MAX_WIDTH: usize = 50;

    // Min spacing required between an name and description
    pub const MIN_SPACING: usize = 5;

    // Left padding of the column for the description
    pub const COLUMN_DESCRIPTION_PADDING: usize = 6;

    // Align of the strings.
    #[derive(Debug, Eq, PartialEq)]
    pub enum Align {
        Row(usize),
        Column,
    }

    // Letter-case
    #[derive(Debug, Clone, Eq, PartialEq)]
    pub enum LetterCase {
        Ignore,
        Upper,
        Lower,
    }

    impl LetterCase {
        pub fn format<S: Into<String>>(&self, value: S) -> String {
            let mut string = value.into();
            match self {
                LetterCase::Ignore => {},
                LetterCase::Upper => { string.make_ascii_uppercase() },
                LetterCase::Lower => { string.make_ascii_lowercase() },
            }

            string
        }
    }

    // How display the option arguments
    #[derive(Debug, Clone)]
    pub struct DisplayArgs {
        pub letter_case: LetterCase, // Upper
        pub grouping: (char, char),  // ('<', '>')
        pub only_name: bool,         // false
        pub delimiter: char,         // |
    }

    impl DisplayArgs {
        pub fn new() -> Self {
            Default::default()
        }
    }

    impl Default for DisplayArgs {
        #[inline]
        fn default() -> Self {
            DisplayArgs {
                letter_case: LetterCase::Upper,
                grouping: ('<', '>'),
                only_name: false,
                delimiter: '|'
            }
        }
    }

    // -v, --version                        Shows the version
    // -t, --times <TIMES>                  Numbers of times to execute
    // -c, -C, --color <RED|GREEN|BLUE>     Color to use
    pub fn option_to_string(
        context: &Context,
        option: &CommandOption,
        align: Align,
        include_args: bool,
    ) -> String {
        // Option name and it's aliases
        let names = if option.get_aliases().next().is_some() {
            let alias_prefix = context.alias_prefixes().next().unwrap();
            let name_prefix = context.name_prefixes().next().unwrap();
            let names: String = option
                .get_aliases()
                .map(|n| format!("{}{}", alias_prefix, n))
                .collect::<Vec<String>>()
                .join(", ");

            format!("{}, {}{}", names, name_prefix, option.get_name())
        } else {
            let name_prefix = context.name_prefixes().next().unwrap();
            // Normally there is 4 spaces if the `alias prefix` and `name` is 1 char
            format!("{}{:4}", name_prefix, option.get_name())
        };

        // Option args
        let mut args = if include_args && option.get_args().len() > 0 {
            args_to_string(option.get_args(), DisplayArgs::default())
        } else {
            None
        };

        // Add left padding
        if let Some(args) = &mut args {
            args.insert(0, ' ');
        }

        match align {
            Align::Row(width) => {
                if let Some(description) = option.get_description() {
                    format!(
                        "{:width$}{}",
                        // format_args! is not working with the width
                        format!("{}{}", names, args.unwrap_or_default()),
                        description,
                        width = width
                    )
                } else {
                    format!("{}{}", names, args.unwrap_or_default())
                }
            }
            Align::Column => {
                // The next column
                if let Some(description) = option.get_description() {
                    format!(
                        // We add a left-padding of 6 spaces
                        "{}{:padding$}{}\n",
                        // format_args! is not working with the width
                        format!("{}{}\n", names, args.unwrap_or_default()),
                        "",
                        description,
                        padding = COLUMN_DESCRIPTION_PADDING
                    )
                } else {
                    format!("{}{}", names, args.unwrap_or_default())
                }
            }
        }
    }

    // version              Shows the version
    pub fn command_to_string(command: &Command, align: Align) -> String {
        match align {
            Align::Row(width) => {
                if let Some(description) = command.get_description() {
                    format!(
                        "{:width$} {}",
                        command.get_name(),
                        description,
                        width = width
                    )
                } else {
                    command.get_name().to_owned()
                }
            }
            Align::Column => {
                if let Some(description) = command.get_description() {
                    format!(
                        "{}\n{:padding$}{}\n",
                        command.get_name(),
                        "",
                        description,
                        padding = COLUMN_DESCRIPTION_PADDING
                    )
                } else {
                    format!("{}\n", command.get_name())
                }
            }
        }
    }

    // <ARG1> <ARG2>
    pub fn args_to_string(args: &ArgumentList, display_args: DisplayArgs) -> Option<String> {
        if args.is_empty() {
            None
        } else {
            let DisplayArgs {
                letter_case,
                grouping,
                only_name,
                delimiter,
            } = display_args;

            match args.len() {
                1 => {
                    let arg = &args[0];
                    if arg.get_valid_values().len() > 0 || only_name {
                        // --option <VALUE1|VALUE2|VALUE2>
                        let buf = &mut [0; 4];
                        let str_delimiter = delimiter.encode_utf8(buf);
                        let values: String = arg.get_valid_values().join(str_delimiter);
                        Some(format!(
                            "{1}{0}{2}",
                            letter_case.format(&values),
                            grouping.0,
                            grouping.1
                        ))
                    } else {
                        // --option <ARG>
                        Some(format!(
                            "{1}{0}{2}",
                            letter_case.format(arg.get_name()),
                            grouping.0,
                            grouping.1
                        ))
                    }
                }
                _ => {
                    // --option <ARG1> <ARG2>
                    let args_names: String = args
                        .iter()
                        .map(|s| {
                            format!(
                                "{1}{0}{2}",
                                letter_case.format(s.get_name()),
                                grouping.0,
                                grouping.1
                            )
                        })
                        .collect::<Vec<String>>()
                        .join(" ");

                    Some(args_names)
                }
            }
        }
    }

    // Calculates the min width required for display the command options
    pub fn calculate_required_options_width(
        context: &Context,
        command: &Command,
        include_args: bool,
    ) -> usize {
        fn args_required_len(option: &CommandOption) -> usize {
            if option.get_args().len() == 0 {
                0
            } else {
                const GROUPING: usize = 2 + 1; // padding + <ARG>

                match option.get_args().len() {
                    1 => {
                        if let Some(arg) = option.get_arg() {
                            if arg.get_valid_values().len() > 0 {
                                let valid_values_len = arg
                                    .get_valid_values()
                                    .iter()
                                    .map(|s| s.len())
                                    .sum::<usize>();

                                let delimiters = arg.get_valid_values().len() - 1;

                                // padding + <VALUE1|VALUE2|VALUE3>
                                valid_values_len + delimiters + GROUPING
                            } else {
                                // padding + <NAME>
                                arg.get_name().len() + GROUPING
                            }
                        } else {
                            unreachable!()
                        }
                    }
                    _ => {
                        let args_len = option
                            .get_args()
                            .iter()
                            .map(|s| s.get_name().len())
                            .sum::<usize>();

                        // padding + <ARG1> + padding + <ARG2> ...
                        args_len + (option.get_args().len() * GROUPING)
                    }
                }
            }
        }

        let name_prefix = context.name_prefixes().next().unwrap();
        let alias_prefix = context.alias_prefixes().next().unwrap();

        // Here we calculate the max width needed for write the options
        // for that we select the `max` len of: name + aliases + delimiter
        let total_width = command
            .get_options()
            .iter()
            .filter(|opt| !opt.is_hidden())
            .fold(0, |width, opt| {
                // Length of the option len
                let name_len = opt.get_name().len() + name_prefix.len();

                // Total length required for the aliases + the alias prefix
                let aliases_len = opt.get_aliases().map(|s| s.len()).sum::<usize>()
                    + (opt.get_aliases().count() * alias_prefix.len());

                // Total length required for the delimiters
                let delimiters = opt.get_aliases().count() * 2;

                // Total length required for the args
                let args_len = if include_args {
                    args_required_len(opt)
                } else {
                    0
                };

                // We select the max between the needed length for this option and the previous.
                cmp::max(
                    width,
                    aliases_len + name_len + args_len + delimiters + MIN_SPACING,
                )
            });

        cmp::max(MIN_WIDTH, total_width)
    }

    // Calculates the min width required for display the command subcommands
    pub fn calculate_required_subcommands_width(command: &Command) -> usize {
        let total_width = command
            .get_subcommands()
            .filter(|c| !c.is_hidden())
            .fold(0, |width, subcommand| {
                cmp::max(width, subcommand.get_name().len() + MIN_SPACING)
            });

        cmp::max(MIN_WIDTH, total_width)
    }
}