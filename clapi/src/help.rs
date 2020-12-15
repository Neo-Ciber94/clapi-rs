use std::fmt::Write;
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
pub trait HelpProvider {
    /// Returns a `String` information about the command.
    /// - Name, description, options and subcommands.
    fn help(&self, context: &Context, command: &Command) -> String;

    /// Returns a `String` information about the usage of the command.
    fn usage(&self, context: &Context, command: &Command) -> String;

    /// Type of the `HelpProvider`.
    fn kind(&self) -> HelpKind;

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

/// Type of the `HelpProvider`.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum HelpKind{
    /// The help is a root command child like: `command help`.
    Subcommand,
    /// The help is a root command option like: `command --help`.
    Option
}

/// Default implementation of the `HelpCommand` trait.
#[derive(Debug, Clone)]
pub struct DefaultHelpProvider(pub HelpKind);
impl Default for DefaultHelpProvider{
    fn default() -> Self {
        DefaultHelpProvider(HelpKind::Subcommand)
    }
}
impl HelpProvider for DefaultHelpProvider {
    fn help(&self, context: &Context, command: &Command) -> String {
        let mut writer = HelpWriter::new(context);

        // Command name
        writer.writeln(command.get_name());

        // Command description
        if let Some(description) = command.get_description() {
            writer.indented(|w| w.writeln(description));
        }

        // Command usage
        if command.take_args() || command.get_options().len() > 0 || command.get_children().len() > 0
        {
            writer.section("USAGE:", |w| {
                let mut args_names = Vec::new();

                for arg in command.get_args() {
                    if arg.get_arg_count().takes_exactly(1){
                        args_names.push(format!("<{}>", arg.get_name().to_uppercase()));
                    } else {
                        args_names.push(format!("<{}...>", arg.get_name().to_uppercase()));
                    }
                }

                // Names of the arguments as: <ARG0> <ARG1> <ARG2...>
                let args_names: String = args_names.join(" ");

                if command.take_args() {
                    w.writeln(format!("{} {}", command.get_name(), args_names));
                }

                if command.get_options().len() > 0 {
                    let mut result = String::from(command.get_name());
                    write!(result, " [OPTION]").unwrap();

                    if command.get_options().iter().any(|o| o.take_args()) {
                        write!(result, " {}", args_names).unwrap();
                    }

                    w.writeln(result);
                }

                if command.get_children().len() > 0 {
                    let mut children = command.get_children();

                    if children.any(|c| c.take_args()) {
                        w.writeln(format!("{} [SUBCOMMAND] <ARGS>", command.get_name()));
                    }

                    if children.any(|c| c.get_options().len() > 0) {
                        let mut result = String::from(command.get_name());
                        write!(result, " [SUBCOMMAND] [OPTION]").unwrap();

                        if children.any(|c| c.get_options().iter().any(|o| o.take_args())) {
                            write!(result, " <ARGS>").unwrap();
                        }

                        w.writeln(result);
                    }
                }
            });
        }

        // Command options
        if command.get_options().len() > 0 {
            writer.section("OPTIONS:", |w| {
                for option in command.get_options() {
                    w.write_option(option);
                }
            });
        }

        // Command children
        if command.get_children().len() > 0 {
            writer.section("SUBCOMMAND:", |w| {
                for child in command.get_children() {
                    w.write_command(child);
                }
            });
        }

        // Command about
        if let Some(about) = command.get_about() {
            writer.writeln("");
            writer.writeln(about);
        }

        writer.into_string()
    }

    fn usage(&self, context: &Context, command: &Command) -> String {
        let mut writer = HelpWriter::new(context);
        writer.section("USAGE", |w| {
            let mut args_names = Vec::new();

            for arg in command.get_args() {
                if arg.get_arg_count().takes_exactly(1){
                    args_names.push(format!("<{}>", arg.get_name().to_uppercase()));
                } else {
                    args_names.push(format!("<{}...>", arg.get_name().to_uppercase()));
                }
            }

            let args_names: String = args_names.join(" ");

            if command.take_args() {
                w.writeln(format!("{} {}", command.get_name(), args_names));
            }

            if command.get_options().len() > 0 {
                let mut result = String::from(command.get_name());
                write!(result, " [OPTIONS]").unwrap();

                if command.get_options().iter().any(|o| o.take_args()) {
                    write!(result, " {}", args_names).unwrap();
                }

                w.writeln(result);
            }
        });

        writer.into_string()
    }

    fn kind(&self) -> HelpKind{
        self.0
    }
}

mod indented_writer {
    use std::borrow::Borrow;

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
        current_indent: u32,
        indent: String,
    }

    impl IndentedWriter {
        /// Constructs a new `IndentedWriter`.
        #[inline]
        pub fn new() -> Self {
            Self::with_indent(" ".repeat(3))
        }

        /// Constructs a new `IndentedWriter` with the specified spacing.
        #[inline]
        pub fn with_indent(indent: String) -> Self {
            assert!(!indent.is_empty(), "indent cannot be empty");

            IndentedWriter {
                buffer: String::new(),
                current_indent: 0,
                indent
            }
        }

        /// Returns the indentation level of this writer.
        pub fn current_indent(&self) -> u32 {
            self.current_indent
        }

        /// Returns a reference to this writer buffer.
        pub fn buffer(&self) -> &String {
            &self.buffer
        }

        /// Increment the indentation level by 1.
        pub fn increment_indent(&mut self) {
            self.current_indent += 1;
        }

        /// Decrement the indentation level by 1.
        pub fn decrement_indent(&mut self) {
            self.current_indent -= 1;
        }

        /// Writes the current value in the buffer.
        pub fn write<S: Borrow<str>>(&mut self, value: S) {
            self.write_indent();
            self.buffer.push_str(value.borrow());
        }

        /// Writes the current value in the buffer with a `newline`.
        pub fn writeln<S: Borrow<str>>(&mut self, value: S) {
            self.write_indent();
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
            self.current_indent += 1;
            f(self);
            self.current_indent -= 1;
        }

        /// Gets the resulting `String` of this writer.
        #[inline]
        pub fn into_string(self) -> String {
            self.buffer
        }

        fn write_indent(&mut self) {
            for _ in 0..self.current_indent {
                self.buffer.push_str(self.indent.as_str());
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

    /// Utilities for write command help.
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
        pub fn with_spacing(context: &'a Context, indent: String) -> Self {
            HelpWriterBuilder::new()
                .writer(IndentedWriter::with_indent(indent))
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

    /// A `HelpWriter` builder.
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

    fn write_option_internal(context: &Context, option: &CommandOption) -> String {
        const WIDTH: usize = 25;
        let name_prefix: &str = context.name_prefixes().next().unwrap();
        let alias_prefix: &str = context.alias_prefixes().next().unwrap();

        let mut buffer = String::new();

        let mut names = if let Some(alias) = option.get_aliases().next() {
            format!(
                "{}{}, {}{}",
                alias_prefix,
                alias.as_str(),
                name_prefix,
                option.get_name(),
            )
        } else {
            // A width of 2 should be enough to align with the alias prefix
            // which is expected to be 2 characters as: '-a', '-v', '/b'
            format!("{:>2}{}", name_prefix, option.get_name())
        };

        if option.get_args().len() > 0 {
            let mut args_names = Vec::new();

            for arg in option.get_args() {
                if arg.get_arg_count().takes_exactly(1){
                    args_names.push(format!(" <{}>", arg.get_name().to_uppercase()));
                } else {
                    args_names.push(format!(" <{}...>", arg.get_name().to_uppercase()));
                }
            }

            names.push_str(&args_names.join(" "));
        }

        if let Some(description) = option.get_description() {
            buffer.push_str(&format!("{:width$}{}", names, description, width = WIDTH));
        } else {
            buffer.push_str(&format!("{:width$}", names, width = WIDTH));
        }

        buffer
    }

    fn write_command_internal(_: &Context, command: &Command) -> String {
        const WIDTH: usize = 15;
        let mut buffer = String::new();

        if let Some(description) = command.get_description() {
            buffer.push_str(&format!(
                "{:width$}{}",
                command.get_name(),
                description,
                width = WIDTH
            ))
        } else {
            buffer.push_str(&format!("{:width$}", command.get_name(), width = WIDTH))
        }

        buffer
    }
}
