use crate::args::Arguments;
use crate::error::Result;
use crate::option::{CommandOption, Options};
use crate::symbol::Symbol;
use crate::utils::Also;
use linked_hash_set::LinkedHashSet;
use std::cell::{RefCell, RefMut};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

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
    args: Arguments,
    handler: Option<Rc<RefCell<dyn FnMut(&Options, &Arguments) -> Result<()>>>>,
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
        let name = name.into();
        assert!(!name.trim().is_empty(), "name cannot be empty");

        let args =
            Arguments::none().also_mut(|a| a.parent = Some(Symbol::Cmd(name.clone())));

        Command {
            name,
            parent: None,
            description: None,
            help: None,
            children: LinkedHashSet::new(),
            options,
            handler: None,
            args,
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

    /// Returns the `Arguments` of this command.
    pub fn get_args(&self) -> &Arguments {
        &self.args
    }

    /// Returns `true` if this command take args.
    pub fn take_args(&self) -> bool {
        self.args.take_args()
    }

    /// Returns the parent `Symbol` of this command, or `None` if not have a parent.
    pub fn get_parent(&self) -> Option<&Symbol> {
        self.parent.as_ref()
    }

    /// Returns the handler of this command, or `None` if not set.
    pub fn get_handler(
        &self,
    ) -> Option<RefMut<'_, dyn FnMut(&Options, &Arguments) -> Result<()> + 'static>> {
        self.handler.as_ref().map(|x| x.borrow_mut())
    }

    /// Returns the child with the given name, or `None` if not child if found.
    pub fn find_subcommand<S: AsRef<str>>(&self, name: S) -> Option<&Command> {
        self.children.iter().find(|c| c.get_name() == name.as_ref())
    }

    /// Sets a short description of this command.
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets a usage description of this command.
    pub fn help<S: Into<String>>(mut self, help: S) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Adds an `CommandOption` to this command.
    pub fn option(mut self, option: CommandOption) -> Self {
        if cfg!(debug_assertions) {
            let option_name = option.get_name().to_string();
            assert!(
                self.options.add(option),
                "`{}` already contains a `CommandOption` named: `{}`",
                self.name,
                option_name,
            );
        } else {
            self.options.add(option);
        }
        self
    }

    /// Replaces the options of this command with the specified.
    pub fn options(mut self, options: Options) -> Self {
        self.options = options;
        self
    }

    /// Sets the `Arguments` of this command.
    pub fn args(mut self, mut args: Arguments) -> Self {
        args.parent = Some(Symbol::Cmd(self.name.clone()));
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
        F: FnMut(&Options, &Arguments) -> Result<()> + 'static,
    {
        self.handler = Some(Rc::new(RefCell::new(f)));
        self
    }

    /// Adds a new child `Command`.
    pub fn subcommand(mut self, command: Command) -> Self {
        self.add_command(command);
        self
    }

    /// Sets the argument values of this command.
    pub fn set_args_values<'a, S, I>(&mut self, args: I) -> Result<()>
        where
            I: IntoIterator<Item = &'a S>,
            S: ToString + 'a,
    {
        self.args.set_values(args)
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
            .field("parent", &self.parent)
            .field("arguments", &self.get_args())
            .field(
                "handler",
                if self.handler.is_some() {
                    &"fn(&Options, &[String) -> Result<()>"
                } else {
                    &"None"
                },
            )
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
    fn name_test() {
        let cmd = Command::new("time");
        assert_eq!(cmd.get_name(), "time");
    }

    #[test]
    fn description() {
        let cmd = Command::new("time").description("Shows the time");

        assert_eq!(cmd.get_description(), Some("Shows the time"));
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
        let mut cmd = Command::new("time").args(Arguments::new(1));

        assert_eq!(cmd.get_args(), &Arguments::new(1));

        assert!(cmd.set_args_values(&["1"]).is_ok());
        assert!(cmd.get_args().get_values().contains(&String::from("1")));
    }

    #[test]
    fn handle_test() {
        static mut VALUE: usize = 0;

        let cmd = Command::new("counter").handler(inc);

        fn inc(_: &Options, _: &Arguments) -> Result<()> {
            unsafe { VALUE += 1 };
            Ok(())
        }

        let opts = Options::new();
        let args = Arguments::none();

        cmd.get_handler().unwrap().deref_mut()(&opts, &args).unwrap();
        cmd.get_handler().unwrap().deref_mut()(&opts, &args).unwrap();

        assert_eq!(unsafe { VALUE }, 2);
    }
}
