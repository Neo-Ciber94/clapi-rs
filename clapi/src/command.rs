#![allow(clippy::type_complexity, clippy::len_zero)]
use crate::args::{Argument, ArgumentList};
use crate::error::Result;
use crate::option::{CommandOption, OptionList};
use crate::symbol::Symbol;
use crate::utils::debug_option;
use crate::{CommandLine, ParseResult};
use linked_hash_set::LinkedHashSet;
use std::borrow::Borrow;
use std::cell::{RefCell, RefMut};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

/// A command-line command.
#[derive(Clone)]
pub struct Command {
    parent: Option<Symbol>,
    name: String,
    description: Option<String>,
    usage: Option<String>,
    help: Option<String>,
    children: LinkedHashSet<Command>,
    options: OptionList,
    args: ArgumentList,
    is_hidden: bool,
    handler: Option<Rc<RefCell<dyn FnMut(&OptionList, &ArgumentList) -> Result<()>>>>,
}

impl Command {
    /// Constructs a new `Command`.
    ///
    /// # Panics
    /// Panics if the command `name` is blank or empty.
    #[inline]
    pub fn new<S: Into<String>>(name: S) -> Self {
        Command::with_options(name, OptionList::new())
    }

    /// Constructs a new `Command` named after the running executable.
    #[inline]
    pub fn root() -> Self {
        Command::new(current_filename())
    }

    /// Constructs a new `Command` with the specified `Options`.
    ///
    /// # Panics
    /// Panics if the command `name` is blank or empty.
    pub fn with_options<S: Into<String>>(name: S, options: OptionList) -> Self {
        let name = assert_not_blank!(name.into(), "`name` cannot be blank or empty");

        Command {
            name,
            parent: None,
            description: None,
            usage: None,
            help: None,
            children: LinkedHashSet::new(),
            handler: None,
            args: ArgumentList::new(),
            options,
            is_hidden: false
        }
    }

    /// Returns the name of the command.
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns a short description of the command, or `None` if is not set.
    pub fn get_description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Returns information about the usage of this command.
    pub fn get_usage(&self) -> Option<&str> {
        self.usage.as_deref()
    }

    /// Returns the `help` information of the command.
    pub fn get_help(&self) -> Option<&str> {
        self.help.as_deref()
    }

    /// Returns an `ExactSizeIterator` over the children of this command.
    pub fn get_children(&self) -> Iter<'_> {
        Iter {
            iter: self.children.iter(),
        }
    }

    /// Returns the `Options` of this command.
    pub fn get_options(&self) -> &OptionList {
        &self.options
    }

    /// Returns the `Argument` this option takes or `None` if have more than 1 argument.
    pub fn get_arg(&self) -> Option<&Argument> {
        if self.args.len() > 1 {
            None
        } else {
            Some(&self.args[0])
        }
    }

    /// Returns the `Arguments` of this command.
    pub fn get_args(&self) -> &ArgumentList {
        &self.args
    }

    /// Returns `true` if this command take args.
    pub fn take_args(&self) -> bool {
        self.args.len() > 0
    }

    /// Returns the parent `Symbol` of this command, or `None` if is a root command.
    pub fn get_parent(&self) -> Option<&Symbol> {
        self.parent.as_ref()
    }

    /// Returns `true` if this command is no visible for `help`.
    pub fn is_hidden(&self) -> bool {
        self.is_hidden
    }

    /// Returns the handler of this command, or `None` if not set.
    pub fn get_handler(
        &self,
    ) -> Option<RefMut<'_, dyn FnMut(&OptionList, &ArgumentList) -> Result<()> + 'static>> {
        self.handler.as_ref().map(|x| x.borrow_mut())
    }

    /// Returns the child with the given name, or `None` if not child if found.
    pub fn find_subcommand<S: AsRef<str>>(&self, name: S) -> Option<&Command> {
        self.children.iter().find(|c| c.get_name() == name.as_ref())
    }

    /// Sets a short description of this command.
    ///
    /// # Panics:
    /// Panics if the `description` is blank or empty.
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(assert_not_blank!(description.into(), "`description` cannot be blank or empty"));
        self
    }

    /// Sets information about the usage of this command.
    ///
    /// # Panics:
    /// Panics if the `usage` is blank or empty.
    pub fn usage<S: Into<String>>(mut self, usage: S) -> Self {
        self.usage = Some(assert_not_blank!(usage.into(), "`usage` cannot be blank or empty"));
        self
    }

    /// Sets help information about this command.
    ///
    /// # Panics:
    /// Panics if the `help` is blank or empty.
    pub fn help<S: Into<String>>(mut self, help: S) -> Self {
        self.help = Some(assert_not_blank!(help.into(), "`help` cannot be blank or empty"));
        self
    }

    /// Adds an `CommandOption` to this command.
    ///
    /// # Panics:
    /// Panics it the command contains an `CommandOption` with the same name or alias.
    pub fn option(mut self, option: CommandOption) -> Self {
        self.add_option(option);
        self
    }

    /// Replaces the options of this command with the specified.
    pub fn options(mut self, options: OptionList) -> Self {
        self.options = options;
        self
    }

    /// Adds a new `Argument` to this command.
    ///
    /// # Panics:
    /// Panic if the command contains an `Argument` with the same name.
    pub fn arg(mut self, arg: Argument) -> Self {
        if let Err(duplicated) = self.args.add(arg) {
            panic!(
                "`{}` already contains an argument named: `{}`",
                self.name,
                duplicated.get_name()
            );
        }
        self
    }

    /// Sets the `Arguments` of this command.
    pub fn args(mut self, args: ArgumentList) -> Self {
        self.args = args;
        self
    }

    /// Specify if this command is hidden for the `help`, this property may be ignore
    /// if is the `root` command.
    ///
    /// What will be hidden or not about the command is up to the implementor of the `Help` trait.
    pub fn hidden(mut self, is_hidden: bool) -> Self {
        self.is_hidden = is_hidden;
        self
    }

    /// Sets the handler of this command.
    ///
    /// # Example
    /// ```rust
    /// use clapi::Command;
    ///
    /// let cmd = Command::new("test")
    ///     .handler(|_options, _args| {
    ///         println!("This is a test");
    ///         Ok(())
    /// });
    /// ```
    pub fn handler<F>(mut self, f: F) -> Self
    where
        F: FnMut(&OptionList, &ArgumentList) -> Result<()> + 'static,
    {
        self.handler = Some(Rc::new(RefCell::new(f)));
        self
    }

    /// Adds a new child `Command`.
    pub fn subcommand(mut self, command: Command) -> Self {
        self.add_command(command);
        self
    }

    pub(crate) fn add_command(&mut self, mut command: Command) -> bool {
        if self.children.contains(&command) {
            panic!(
                "`{}` already contains a subcommand named: `{}`",
                self.name,
                command.get_name()
            );
        }

        command.parent = Some(Symbol::Cmd(self.name.clone()));
        self.children.insert(command)
    }

    pub(crate) fn add_option(&mut self, option: CommandOption) {
        if let Err(duplicated) = self.options.add(option) {
            if self.options.contains(duplicated.get_name()) {
                panic!(
                    "`{}` already contains an option named: `{}`",
                    self.name,
                    duplicated.get_name()
                );
            } else {
                for alias in duplicated.get_aliases() {
                    if self.options.contains(alias) {
                        panic!(
                            "`{}` already contains an option with alias: `{}`",
                            self.name, alias
                        );
                    }
                }

                unreachable!()
            }
        }
    }

    //////////////////////////////////////////////////
    //             Utility Parse Methods            //
    //////////////////////////////////////////////////

    /// Constructs a `CommandLine` using this command.
    ///
    /// # Example:
    /// ```no_run
    /// use clapi::{Command, Argument, CommandOption};
    /// use clapi::validator::parse_validator;
    ///
    /// let cli = Command::root()
    ///             .arg(Argument::one_or_more("values").validator(parse_validator::<i64>()))
    ///             .option(CommandOption::new("negate")
    ///                 .arg(Argument::new().validator(parse_validator::<bool>())))
    ///             .handler(|opts, args|{
    ///                 let negate = opts.get_arg("negate").unwrap().convert::<bool>()?;
    ///                 let total = args.convert_all::<i64>("values")?.iter().sum::<i64>();
    ///                 if negate {
    ///                     println!("{}", -total);
    ///                 } else {
    ///                     println!("{}", total);
    ///                 }
    ///                 Ok(())
    ///             })
    ///             .into_cli()
    ///             .use_default_suggestions()
    ///             .use_default_suggestions()
    ///             .run();
    /// ```
    #[inline]
    pub fn into_cli(self) -> CommandLine {
        CommandLine::new(self)
    }

    /// Parse the arguments from `std::env::args` using this command and returns the `ParseResult`.
    ///
    /// # Example:
    /// ```no_run
    /// use clapi::{Command, CommandOption, Argument};
    /// use clapi::validator::parse_validator;
    ///
    /// let result = Command::root()
    ///                 .option(CommandOption::new("negate")
    ///                     .arg(Argument::new().validator(parse_validator::<bool>())))
    ///                 .arg(Argument::one_or_more("values").validator(parse_validator::<i64>()))
    ///                 .parse_args()
    ///                 .unwrap();
    /// ```
    #[inline]
    pub fn parse_args(self) -> Result<ParseResult> {
        self.parse_from(std::env::args().skip(1))
    }

    /// Parse the arguments using this command and returns the `ParseResult`.
    ///
    /// # Example:
    /// ```
    /// use clapi::{Command, CommandOption, Argument};
    /// use clapi::validator::parse_validator;
    ///
    /// let result = Command::root()
    ///                 .option(CommandOption::new("negate")
    ///                     .arg(Argument::new().validator(parse_validator::<bool>())))
    ///                 .arg(Argument::one_or_more("values").validator(parse_validator::<i64>()))
    ///                 .parse_from(vec!["--negate=true", "1", "2", "3"])
    ///                 .unwrap();
    ///
    /// assert!(result.contains_option("negate"));
    /// assert_eq!(result.arg().unwrap().convert_all::<i64>().ok(), Some(vec![1, 2, 3]));
    /// ```
    #[inline]
    pub fn parse_from<I, S>(self, args: I) -> Result<ParseResult>
    where
        I: IntoIterator<Item = S>,
        S: Borrow<str>,
    {
        let context = crate::Context::new(self);
        let mut parser = crate::Parser::new(&context);
        parser.parse(args)
    }
}

impl Eq for Command {}

impl PartialEq for Command {
    fn eq(&self, other: &Self) -> bool {
        // This implementation is enough for the purposes of the library
        // but don't reflect the true equality of this struct
        self.name == other.name
    }
}

impl Hash for Command {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.name.as_bytes())
    }
}

impl Debug for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command")
            .field("name", &self.get_name())
            .field("description", &self.get_description())
            .field("about", &self.get_usage())
            .field("help", &self.get_help())
            .field("parent", &self.get_parent())
            .field("options", &self.get_options())
            .field("arguments", &self.get_args())
            .field(
                "handler",
                &debug_option(
                    &self.get_handler(),
                    "FnMut(&OptionList, &ArgumentList) -> Result<()>",
                ),
            )
            .field("children", &self.get_children())
            .finish()
    }
}

/// An iterator over the children of a `Command`.
#[derive(Debug, Clone)]
pub struct Iter<'a> {
    iter: linked_hash_set::Iter<'a, Command>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Command;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a> ExactSizeIterator for Iter<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a> IntoIterator for &'a Command {
    type Item = &'a Command;
    type IntoIter = Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.get_children()
    }
}

#[doc(hidden)]
pub fn current_filename() -> String {
    current_exe_filename()
        .trim_end_matches(std::env::consts::EXE_SUFFIX)
        .to_string()
}

#[doc(hidden)]
pub fn current_exe_filename() -> String {
    std::env::current_exe()
        .unwrap()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::DerefMut;

    #[test]
    fn command_test1() {
        let cmd = Command::new("time")
            .description("Shows the time")
            .usage("Sets the time or show it");

        assert_eq!(cmd.get_name(), "time");
        assert_eq!(cmd.get_description(), Some("Shows the time"));
        assert_eq!(cmd.get_usage(), Some("Sets the time or show it"));
    }

    #[test]
    #[should_panic]
    fn command_test2() {
        Command::new(" ");
    }

    #[test]
    #[should_panic]
    fn command_test3() {
        Command::new("time").description("");
    }

    #[test]
    #[should_panic]
    fn command_test4() {
        Command::new("time")
            .description("Show the time")
            .usage("\n");
    }

    #[test]
    fn children_test() {
        let cmd = Command::new("data")
            .subcommand(Command::new("set"))
            .subcommand(Command::new("get").subcommand(Command::new("first")));

        assert_eq!(cmd.get_children().count(), 2);
        assert_eq!(cmd.find_subcommand("set"), Some(&Command::new("set")));
        assert_eq!(cmd.find_subcommand("get"), Some(&Command::new("get")));
        assert_eq!(
            cmd.find_subcommand("get").unwrap().find_subcommand("first"),
            Some(&Command::new("first"))
        );
    }

    #[test]
    #[should_panic]
    fn duplicated_command_test() {
        Command::new("data")
            .subcommand(Command::new("set"))
            .subcommand(Command::new("get").subcommand(Command::new("first")))
            .subcommand(Command::new("get"));
    }

    #[test]
    fn option_test() {
        let cmd = Command::new("time")
            .option(CommandOption::new("version").alias("v"))
            .option(CommandOption::new("day_of_week").alias("dw"));

        assert_eq!(
            cmd.get_options().get("version"),
            Some(&CommandOption::new("version"))
        );
        assert_eq!(
            cmd.get_options().get("v"),
            Some(&CommandOption::new("version"))
        );
    }

    #[test]
    #[should_panic]
    fn duplicated_option_test1() {
        Command::new("time")
            .option(CommandOption::new("version").alias("v"))
            .option(CommandOption::new("day_of_week").alias("dw"))
            .option(CommandOption::new("version"));
    }

    #[test]
    #[should_panic]
    fn duplicated_option_test2() {
        Command::new("time")
            .option(CommandOption::new("version").alias("v"))
            .option(CommandOption::new("day_of_week").alias("dw"))
            .option(CommandOption::new("v"));
    }

    #[test]
    #[should_panic]
    fn duplicated_option_test3() {
        Command::new("time")
            .option(CommandOption::new("version").alias("v"))
            .option(CommandOption::new("day_of_week").alias("dw"))
            .option(CommandOption::new("verbose").alias("v"));
    }

    #[test]
    fn args_test() {
        let cmd = Command::new("time").arg(Argument::with_name("arg").value_count(1));

        assert_eq!(cmd.get_arg().unwrap(), &Argument::with_name("arg"));
    }

    #[test]
    fn handle_test() {
        static mut VALUE: usize = 0;

        let cmd = Command::new("counter").handler(inc);

        fn inc(_: &OptionList, _: &ArgumentList) -> Result<()> {
            unsafe { VALUE += 1 };
            Ok(())
        }

        let opts = OptionList::new();
        let args = ArgumentList::new();

        cmd.get_handler().unwrap().deref_mut()(&opts, &args).unwrap();
        cmd.get_handler().unwrap().deref_mut()(&opts, &args).unwrap();

        assert_eq!(unsafe { VALUE }, 2);
    }
}
