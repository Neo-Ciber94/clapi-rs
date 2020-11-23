use crate::error::Result;
use crate::option::{CommandOption, Options};
use crate::symbol::Symbol;
use linked_hash_set::LinkedHashSet;
use std::cell::{RefCell, RefMut};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use crate::args::{ArgumentList, Argument};
use crate::utils::debug_option;

/// A command-line command.
#[derive(Clone)]
pub struct Command {
    // Name of the parent command
    pub(crate) parent: Option<Symbol>,
    name: String,
    description: Option<String>,
    help: Option<String>,
    children: LinkedHashSet<Command>,
    options: Options,
    args: ArgumentList,
    handler: Option<Rc<RefCell<dyn FnMut(&Options, &ArgumentList) -> Result<()>>>>,
}

impl Command {
    /// Constructs a new `Command`.
    #[inline]
    pub fn new<S: Into<String>>(name: S) -> Self {
        Command::with_options(name, Options::new())
    }

    /// Constructs a new `Command` named after the running executable.
    #[inline]
    pub fn root() -> Self {
        Command::new(current_filename())
    }

    /// Constructs a new `Command` with the specified `Options`.
    pub fn with_options<S: Into<String>>(name: S, options: Options) -> Self {
        let name = assert_not_blank!(name.into(), "`name` cannot be blank or empty");

        Command {
            name,
            parent: None,
            description: None,
            help: None,
            children: LinkedHashSet::new(),
            handler: None,
            args: ArgumentList::new(),
            options,
        }
    }

    /// Returns the name of the command.
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns a short description of the command, or `None` if is not set.
    pub fn get_description(&self) -> Option<&str> {
        self.description.as_ref().map(|s| s.as_str())
    }

    /// Returns a description of the usage of this command.
    pub fn get_help(&self) -> Option<&str> {
        self.help.as_ref().map(|s| s.as_str())
    }

    /// Returns an `ExactSizeIterator` over the children of this command.
    pub fn get_children(&self) -> impl ExactSizeIterator<Item = &'_ Command> + Debug {
        self.children.iter()
    }

    /// Returns the `Options` of this command.
    pub fn get_options(&self) -> &Options {
        &self.options
    }

    /// Returns the `Argument` this option takes.
    pub fn get_arg(&self) -> Option<&Argument>{
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

    /// Returns the parent `Symbol` of this command, or `None` if not have a parent.
    pub fn get_parent(&self) -> Option<&Symbol> {
        self.parent.as_ref()
    }

    /// Returns the handler of this command, or `None` if not set.
    pub fn get_handler(&self) -> Option<RefMut<'_, dyn FnMut(&Options, &ArgumentList) -> Result<()> + 'static>> {
        self.handler.as_ref().map(|x| x.borrow_mut())
    }

    /// Returns the child with the given name, or `None` if not child if found.
    pub fn find_subcommand<S: AsRef<str>>(&self, name: S) -> Option<&Command> {
        self.children.iter().find(|c| c.get_name() == name.as_ref())
    }

    /// Sets a short description of this command.
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(assert_not_blank!(description.into(), "`description` cannot be blank or empty"));
        self
    }

    /// Sets a usage description of this command.
    pub fn help<S: Into<String>>(mut self, help: S) -> Self {
        self.help = Some(assert_not_blank!(help.into(), "`help` cannot be blank or empty"));
        self
    }

    /// Adds an `CommandOption` to this command.
    #[cfg(debug_assertions)]
    pub fn option(mut self, option: CommandOption) -> Self {
        let option_name = option.get_name().to_string();
        assert!(
            self.options.add(option),
            "`{}` already contains a `CommandOption` named: `{}`",
            self.name, option_name,
        );

        self
    }

    /// Adds an `CommandOption` to this command.
    #[cfg(not(debug_assertions))]
    pub fn option(mut self, option: CommandOption) -> Self {
        self.options.add(option);
        self
    }

    /// Replaces the options of this command with the specified.
    pub fn options(mut self, options: Options) -> Self {
        self.options = options;
        self
    }

    /// Adds a new `Argument` to this command.
    #[cfg(debug_assertions)]
    pub fn arg(mut self, arg: Argument) -> Self {
        let arg_name = arg.get_name().to_string();
        assert!(
            self.args.add(arg),
            "`{}` already contains an `Argument` named: `{}`",
            self.name, arg_name,
        );
        self
    }

    /// Adds a new `Argument` to this command.
    #[cfg(not(debug_assertions))]
    pub fn arg(mut self, arg: Argument) -> Self {
        self.args.add(arg);
        self
    }

    /// Sets the `Arguments` of this command.
    pub fn args(mut self, args: ArgumentList) -> Self {
        self.args = args;
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
        F: FnMut(&Options, &ArgumentList) -> Result<()> + 'static,
    {
        self.handler = Some(Rc::new(RefCell::new(f)));
        self
    }

    /// Adds a new child `Command`.
    pub fn subcommand(mut self, command: Command) -> Self {
        self.add_command(command);
        self
    }

    #[inline]
    pub(crate) fn add_command(&mut self, mut command: Command) {
        debug_assert!(
            !self.children.contains(&command),
            "`{}` already contains a command named: `{}`",
            command.name,
            self.name
        );
        command.parent = Some(Symbol::Cmd(self.name.clone()));
        self.children.insert(command);
    }

    #[allow(dead_code)]
    // todo: remove?
    pub(crate) fn full_name(&self) -> String {
        if let Some(parent) = &self.parent {
            format!("{} {}", parent.name(), self.name)
        } else {
            self.name.clone()
        }
    }
}

impl Eq for Command {}

impl PartialEq for Command {
    fn eq(&self, other: &Self) -> bool {
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
            .field("parent", &self.get_parent())
            .field("options", &self.get_options())
            .field("arguments", &self.get_args())
            .field("handler", &debug_option(&self.get_handler(), "FnMut(&Options, &ArgumentList) -> Result<()>"))
            .field("children", &self.get_children())
            .finish()
    }
}

impl<'a> IntoIterator for &'a Command {
    type Item = &'a Command;
    type IntoIter = linked_hash_set::Iter<'a, Command>;

    fn into_iter(self) -> Self::IntoIter {
        self.children.iter()
    }
}

#[doc(hidden)]
pub fn current_filename() -> &'static str {
    static mut FILE_NAME: Option<String> = None;

    unsafe {
        FILE_NAME
            .get_or_insert_with(|| current_filename_internal(false))
            .as_str()
    }
}

#[doc(hidden)]
pub fn current_filename_internal(include_exe: bool) -> String {
    let path = std::env::current_exe().unwrap();
    let filename = path.file_name().unwrap();

    if include_exe {
        filename.to_str().unwrap().to_string()
    } else {
        let ext = path.extension().unwrap();

        filename
            .to_str()
            .unwrap()
            .trim_end_matches(ext.to_str().unwrap())
            .trim_end_matches('.')
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::DerefMut;

    #[test]
    fn command_test1() {
        let cmd = Command::new("time")
            .description("Shows the time")
            .help("Sets the time or show it");

        assert_eq!(cmd.get_name(), "time");
        assert_eq!(cmd.get_description(), Some("Shows the time"));
        assert_eq!(cmd.get_help(), Some("Sets the time or show it"));
    }

    #[test]
    #[should_panic]
    fn command_test2() {
        Command::new(" ");
    }

    #[test]
    #[should_panic]
    fn command_test3() {
        Command::new("time")
            .description("");
    }

    #[test]
    #[should_panic]
    fn command_test4() {
        Command::new("time")
            .description("Show the time")
            .help("\n");
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
        assert_eq!(cmd.get_options().get("v"), Some(&CommandOption::new("version")));
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
        let cmd = Command::new("time")
            .arg(Argument::new("arg").arg_count(1));

        assert_eq!(cmd.get_arg().unwrap(), &Argument::new("arg"));
    }

    #[test]
    fn handle_test() {
        static mut VALUE: usize = 0;

        let cmd = Command::new("counter").handler(inc);

        fn inc(_: &Options, _: &ArgumentList) -> Result<()> {
            unsafe { VALUE += 1 };
            Ok(())
        }

        let opts = Options::new();
        let args = ArgumentList::new();

        cmd.get_handler().unwrap().deref_mut()(&opts, &args).unwrap();
        cmd.get_handler().unwrap().deref_mut()(&opts, &args).unwrap();

        assert_eq!(unsafe { VALUE }, 2);
    }
}
