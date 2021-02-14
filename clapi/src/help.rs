use std::fmt::Write;
use crate::{Context, Command, CommandOption, OptionList};

// Indentation used to write the help messages
const INDENT : &[u8] = b"   ";

// command_help_message
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
        write_indent(buf, INDENT);
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

        for option in command.get_options().iter().filter(|o| !o.is_hidden()) {
            write_indent(buf, INDENT);
            writeln!(buf, "{}", option_to_string(context, option)).unwrap();
        }
    }

    // Command Subcommands
    if subcommand_count > 0 {
        writeln!(buf).unwrap();
        writeln!(buf, "SUBCOMMANDS:").unwrap();

        for command in command.get_subcommands().filter(|c| !c.is_hidden()) {
            write_indent(buf, INDENT);
            writeln!(buf, "{}", command_to_string(command)).unwrap();
        }
    }

    if let Some(msg) = get_after_help_message(context) {
        writeln!(buf).unwrap();
        writeln!(buf, "{}", msg).unwrap();
    }
}

// command_usage_message
pub fn command_usage(buf: &mut String, context: &Context, command: &Command, after_help_message: bool) {
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
            write_indent(buf, INDENT);
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
            write_indent(buf, INDENT);
            write!(buf, "{} [SUBCOMMAND]", command.get_name()).unwrap();

            if command.get_subcommands().any(|c| count_options(c.get_options()) > 0) {
                write!(buf, " [OPTIONS]").unwrap();
            }

            if command.get_subcommands()
                .filter(|c| !c.is_hidden())
                .any(|c| c.take_args()) {
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
        Some(format!("Use '{} {} <subcommand>' for more information about a command.", command, help_command.get_name()))
    } else if context.help_option().is_some() {
        // SAFETY: `name_prefixes` is never empty
        let prefix = context.name_prefixes().next().unwrap();
        let command = context.root().get_name();
        let help_option = context.help_option().unwrap();
        Some(format!("Use '{} <subcommand> {}{}' for more information about a command.", command, prefix, help_option.get_name()))
    } else {
        None
    }
}

// Number of no-hidden options
#[inline]
fn count_options(options: &OptionList) -> usize {
    options.iter()
        .filter(|opt| !opt.is_hidden())
        .count()
}

// Number of no-hidden subcommands
#[inline]
fn count_subcommands(parent: &Command) -> usize {
    parent.get_subcommands()
        .filter(|c| !c.is_hidden())
        .count()
}

// Add indentation to the buffer
#[inline]
fn write_indent(buf: &mut String, indent: &[u8]) {
    buf.push_str(std::str::from_utf8(indent).unwrap())
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