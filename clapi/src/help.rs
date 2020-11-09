use crate::command::Command;
use crate::context::Context;
pub use crate::help::help_writer::*;
pub use crate::help::indented_writer::*;

/// Provides help information about a command.
///
/// # Implementing `HelpCommand`:
/// The only method needed to implement is `help` all the other methods provide a default value.
///
/// Example output of `help`:
/// ```text
/// NAME:
///     test
///
/// DESCRIPTION:
///     This is the test command description
///
/// OPTIONS:
///     -v, --version   version of the command.
///     -a, --author    author of the command.
///
/// SUBCOMMANDS:
///     print   Prints information
/// ```
pub trait HelpCommand {
    /// Returns a `String` information about the usage of the command being passed like:
    /// - Name, description, options and subcommands.
    fn help(&self, context: &Context, command: &Command) -> String;

    /// Returns the name of this help command, the default name is: `help`.
    #[inline]
    fn name(&self) -> &str {
        "help"
    }

    /// Returns the description of this help command, the default is:
    /// `Provides information about a command`.
    #[inline]
    fn description(&self) -> &str {
        "Provides information about a command"
    }
}

/// Default implementation of the `HelpCommand` trait.
#[derive(Debug, Clone, Default)]
pub struct DefaultHelpCommand;
impl HelpCommand for DefaultHelpCommand {
    fn help(&self, context: &Context, command: &Command) -> String {
        use std::fmt::Write;

        let mut writer = HelpWriter::new(context);

        // Command name
        writer.writeln(command.name());

        // Command description
        if let Some(description) = command.description() {
            writer.indented(|w| w.writeln(description));
        }

        // Command usage
        if command.args().take_args() || command.options().len() > 0 || command.children().len() > 0
        {
            writer.section("USAGE:", |w| {
                if command.args().take_args() {
                    w.writeln(format!("{} [ARGS]", command.name()));
                }

                if command.options().len() > 0 {
                    let mut result = String::from(command.name());
                    write!(result, " [OPTIONS]").unwrap();

                    if command.options().iter().any(|o| o.take_args()) {
                        write!(result, " [ARGS]").unwrap();
                    }

                    w.writeln(result);
                }

                if command.children().len() > 0 {
                    let mut children = command.children();

                    if children.any(|c| c.args().take_args()) {
                        w.writeln(format!("{} [SUBCOMMAND] [ARGS]", command.name()));
                    }

                    if children.any(|c| c.options().len() > 0) {
                        let mut result = String::from(command.name());
                        write!(result, " [SUBCOMMAND] [OPTIONS]").unwrap();

                        if children.any(|c| c.options().iter().any(|o| o.take_args())) {
                            write!(result, " [ARGS]").unwrap();
                        }

                        w.writeln(result);
                    }
                }
            });
        }

        // Command options
        if command.options().len() > 0 {
            writer.section("OPTIONS:", |w| {
                for option in command.options() {
                    w.write_option(option);
                }
            });
        }

        // Command children
        if command.children().len() > 0 {
            writer.section("SUBCOMMAND:", |w| {
                for child in command.children() {
                    w.write_command(child);
                }
            });
        }

        // Command help
        if let Some(help) = command.help() {
            writer.writeln("");
            writer.writeln(help);
        }

        writer.into_string()
    }
}

mod indented_writer {
    use std::borrow::Borrow;
    use std::fmt::Write;

    /// A writer with indentation.
    ///
    /// # Example
    /// ```rust
    /// use clapi::help::IndentedWriter;
    ///
    /// let mut writer = IndentedWriter::new();
    /// writer.writeln("Hello");
    /// writer.indented(|w|{
    ///     w.writeln("Make the dinner.");
    ///     w.indented(|w| w.writeln("PS: There are potatoes."))
    /// });
    /// writer.writeln("Good bye");
    ///
    /// assert_eq!("Hello\n   Make the dinner.\n      PS: There are potatoes.\nGood bye\n",
    ///     writer.into_string()
    /// );
    /// ```
    #[derive(Debug, Clone)]
    pub struct IndentedWriter {
        buffer: String,
        indentation: u32,
        spacing: String,
    }

    impl IndentedWriter {
        /// Constructs a new `IndentedWriter`.
        #[inline]
        pub fn new() -> Self {
            Self::new_indented_writer(0, " ".repeat(3))
        }

        /// Constructs a new `IndentedWriter` with the specified spacing.
        #[inline]
        pub fn with_spacing(value: &str) -> Self {
            Self::new_indented_writer(0, value.to_string())
        }

        fn new_indented_writer(indentation: u32, spacing: String) -> Self {
            assert!(!spacing.is_empty(), "spacing cannot be empty");

            IndentedWriter {
                buffer: String::new(),
                indentation,
                spacing,
            }
        }

        /// Returns the indentation level of this writer.
        pub fn indentation(&self) -> u32 {
            self.indentation
        }

        /// Returns a reference to this writer buffer.
        pub fn buffer(&self) -> &String {
            &self.buffer
        }

        /// Increment the indentation level by 1.
        pub fn increment_indent(&mut self) {
            self.indentation += 1;
        }

        /// Decrement the indentation level by 1.
        pub fn decrement_indent(&mut self) {
            self.indentation -= 1;
        }

        /// Writes the current value in the buffer.
        pub fn write<S: Borrow<str>>(&mut self, value: S) {
            self.write_indentation();
            self.buffer.push_str(value.borrow());
        }

        /// Writes the current value in the buffer with a `newline`.
        pub fn writeln<S: Borrow<str>>(&mut self, value: S) {
            self.write_indentation();
            self.buffer.push_str(value.borrow());
            self.buffer.push('\n');
        }

        /// Writes the current value in the buffer if the condition is met.
        #[inline]
        pub fn write_if<S: Borrow<str>>(&mut self, condition: bool, value: S) {
            if condition {
                self.write(value)
            }
        }

        /// Writes the current value in the buffer with a `newline` if the condition is met.
        #[inline]
        pub fn writeln_if<S: Borrow<str>>(&mut self, condition: bool, value: S) {
            if condition {
                self.writeln(value)
            }
        }

        /// Increase the level of indentation and pass the writer to the `FnOnce`
        /// to allow writing with indentation.
        ///
        /// ```rust
        /// use clapi::help::IndentedWriter;
        ///
        /// let mut writer = IndentedWriter::new();
        /// writer.indented(|w| w.write("Hello World"));
        /// assert_eq!("   Hello World", writer.into_string());
        /// ```
        pub fn indented<F: FnOnce(&mut Self)>(&mut self, f: F) {
            self.indentation += 1;
            f(self);
            self.indentation -= 1;
        }

        /// Gets the resulting `String` of this writer.
        #[inline]
        pub fn into_string(self) -> String {
            self.buffer
        }

        fn write_indentation(&mut self) {
            for _ in 0..self.indentation {
                self.buffer.write_str(self.spacing.as_str()).unwrap();
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn write_test() {
            let mut writer = IndentedWriter::new();
            writer.write("Hello World");

            assert_eq!("Hello World", writer.buffer());
        }

        #[test]
        fn writeln_test() {
            let mut writer = IndentedWriter::new();
            writer.writeln("Hello World");

            assert_eq!("Hello World\n", writer.buffer());
        }

        #[test]
        fn indented_test() {
            let mut writer = IndentedWriter::new();
            writer.writeln("Hello");
            writer.indented(|w| {
                w.writeln("I hope you have a nice day.");
                w.indented(|w| w.writeln("PS: Cook the dinner."))
            });
            writer.writeln("Good bye");

            assert_eq!(
                writer.buffer(),
                "Hello\n   I hope you have a nice day.\n      PS: Cook the dinner.\nGood bye\n"
            )
        }
    }
}

mod help_writer {
    use crate::command::Command;
    use crate::context::Context;
    use crate::help::IndentedWriter;
    use crate::option::CommandOption;
    use std::borrow::Borrow;

    pub struct HelpWriter<'a> {
        context: &'a Context,
        writer: IndentedWriter,
        option_writer: Box<dyn Fn(&Context, &CommandOption) -> String>,
        command_writer: Box<dyn Fn(&Context, &Command) -> String>,
    }

    impl<'a> HelpWriter<'a> {
        #[inline]
        pub fn new(context: &'a Context) -> Self {
            HelpWriterBuilder::new().build(context)
        }

        #[inline]
        pub fn with_spacing(context: &'a Context, spacing: &str) -> Self {
            HelpWriterBuilder::new()
                .writer(IndentedWriter::with_spacing(spacing))
                .build(context)
        }

        #[inline]
        pub fn buffer(&self) -> &String {
            self.writer.buffer()
        }

        #[inline]
        pub fn write<S: Borrow<str>>(&mut self, value: S) {
            self.writer.write(value);
        }

        #[inline]
        pub fn writeln<S: Borrow<str>>(&mut self, value: S) {
            self.writer.writeln(value);
        }

        #[inline]
        pub fn write_if<S: Borrow<str>>(&mut self, condition: bool, value: S) {
            self.writer.write_if(condition, value);
        }

        #[inline]
        pub fn writeln_if<S: Borrow<str>>(&mut self, condition: bool, value: S) {
            self.writer.writeln_if(condition, value);
        }

        pub fn indented<F: FnOnce(&mut HelpWriter<'_>)>(&mut self, f: F) {
            self.writer.increment_indent();
            f(self);
            self.writer.decrement_indent();
        }

        pub fn section<F: FnOnce(&mut HelpWriter<'_>)>(&mut self, name: &str, f: F) {
            self.writeln("");
            self.writeln(name);
            self.indented(f)
        }

        pub fn write_option(&mut self, option: &CommandOption) {
            let result = self.option_writer.as_ref()(self.context, option);
            self.writeln(result.as_str());
        }

        pub fn write_command(&mut self, command: &Command) {
            let result = self.command_writer.as_ref()(self.context, command);
            self.writeln(result.as_str());
        }

        #[inline]
        pub fn into_string(self) -> String {
            self.writer.into_string()
        }
    }

    #[derive(Default)]
    pub struct HelpWriterBuilder {
        writer: Option<IndentedWriter>,
        option_writer: Option<Box<dyn Fn(&Context, &CommandOption) -> String>>,
        command_writer: Option<Box<dyn Fn(&Context, &Command) -> String>>,
    }

    impl HelpWriterBuilder {
        #[inline]
        pub fn new() -> Self {
            HelpWriterBuilder::default()
        }

        pub fn writer(mut self, writer: IndentedWriter) -> Self {
            self.writer = Some(writer);
            self
        }

        pub fn option_writer<F>(mut self, f: F) -> Self
        where
            F: Fn(&Context, &CommandOption) -> String + 'static,
        {
            self.option_writer = Some(Box::new(f));
            self
        }

        pub fn command_writer<F>(mut self, f: F) -> Self
        where
            F: Fn(&Context, &Command) -> String + 'static,
        {
            self.command_writer = Some(Box::new(f));
            self
        }

        pub fn build(self, context: &Context) -> HelpWriter<'_> {
            HelpWriter {
                context,
                writer: self.writer.unwrap_or_else(|| IndentedWriter::new()),
                option_writer: self
                    .option_writer
                    .unwrap_or_else(|| Box::new(write_option_internal)),
                command_writer: self
                    .command_writer
                    .unwrap_or_else(|| Box::new(write_command_internal)),
            }
        }
    }

    pub fn write_option_internal(context: &Context, option: &CommandOption) -> String {
        const WIDTH: usize = 25;
        let name_prefix: &str = context.name_prefixes().next().unwrap();
        let alias_prefix: &str = context.alias_prefixes().next().unwrap();

        let mut buffer = String::new();

        let mut names = if let Some(alias) = option.aliases().next() {
            format!(
                "{}{}, {}{}",
                alias_prefix,
                alias.as_str(),
                name_prefix,
                option.name(),
            )
        } else {
            // A width of 6 should be enough to align with the alias prefix
            // which is expected to be 2 characters as: '-a', '-v', '/b'
            format!("{:>6}{}", name_prefix, option.name())
        };

        if let Some(args_name) = option.args().name() {
            names.push_str(&format!(" <{}>", args_name.to_uppercase()));
        }

        if let Some(description) = option.description() {
            buffer.push_str(&format!("{:width$}{}", names, description, width = WIDTH));
        } else {
            buffer.push_str(&format!("{:width$}", names, width = WIDTH));
        }

        buffer
    }

    pub fn write_command_internal(_: &Context, command: &Command) -> String {
        const WIDTH: usize = 15;
        let mut buffer = String::new();

        if let Some(description) = command.description() {
            buffer.push_str(&format!(
                "{:width$}{}",
                command.name(),
                description,
                width = WIDTH
            ))
        } else {
            buffer.push_str(&format!("{:width$}", command.name(), width = WIDTH))
        }

        buffer
    }
}
