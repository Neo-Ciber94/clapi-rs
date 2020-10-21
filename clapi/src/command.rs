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
    handler: Option<Rc<RefCell<dyn FnMut(&Options, &[String]) -> Result<()>>>>,
}

// todo: Implement `help` for provides a long description of what the command do.
impl Command {
    /// Constructs a new `Command`.
    #[inline]
    pub fn new(name: &str) -> Self {
        Command::with_options(name, Options::new())
    }

    /// Constructs a new `Command` with the specified `Options`.
    pub fn with_options(name: &str, options: Options) -> Self {
        assert!(!name.trim().is_empty(), "name cannot be empty");

        let args =
            Arguments::none().also_mut(|a| a.parent = Some(Symbol::Command(name.to_string())));

        Command {
            name: name.to_string(),
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
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns a short description of the command, or `None` if is not set.
    pub fn description(&self) -> Option<&str> {
        self.description.as_ref().map(|s| s.as_str())
    }

    /// Returns a description of the usage of this command.
    pub fn help(&self) -> Option<&str>{
        self.help.as_ref().map(|s| s.as_str())
    }

    /// Returns an `ExactSizeIterator` over the children of this command.
    pub fn children(&self) -> impl ExactSizeIterator<Item = &'_ Command> + Debug {
        self.children.iter()
    }

    /// Returns the `Options` of this command.
    pub fn options(&self) -> &Options {
        &self.options
    }

    /// Returns the `Arguments` of this command.
    pub fn args(&self) -> &Arguments {
        &self.args
    }

    /// Returns `true` if this command take args.
    pub fn take_args(&self) -> bool {
        self.args.take_args()
    }

    /// Returns the parent `Symbol` of this command, or `None` if not have a parent.
    pub fn parent(&self) -> Option<&Symbol> {
        self.parent.as_ref()
    }

    /// Returns the handler of this command, or `None` if not set.
    pub fn handler(
        &self,
    ) -> Option<RefMut<'_, dyn FnMut(&Options, &[String]) -> Result<()> + 'static>> {
        self.handler.as_ref().map(|x| x.borrow_mut())
    }

    /// Returns the child with the given name, or `None` if not child if found.
    pub fn get_child(&self, name: &str) -> Option<&Command> {
        self.children.iter().find(|c| c.name() == name)
    }

    /// Sets a short description of this command.
    pub fn set_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Sets a usage description of this command.
    pub fn set_help(mut self, help: &str) -> Self {
        self.help = Some(help.to_string());
        self
    }

    /// Adds an `CommandOption` to this command.
    pub fn set_option(mut self, option: CommandOption) -> Self {
        self.options.add(option);
        self
    }

    /// Replaces the options of this command with the specified.
    pub fn set_new_options(mut self, options: Options) -> Self {
        self.options = options;
        self
    }

    /// Sets the `Arguments` of this command.
    pub fn set_args(mut self, mut args: Arguments) -> Self {
        args.parent = Some(Symbol::Command(self.name.clone()));
        self.args = args;
        self
    }

    /// Sets the argument values of this command.
    pub fn set_args_values<'a, S, I>(&mut self, args: I) -> Result<()>
    where
        S: ToString + 'a,
        I: IntoIterator<Item = &'a S>,
    {
        self.args.set_values(args)
    }

    /// Sets the handler of this command.
    ///
    /// # Example
    /// ```rust
    /// use clapi::command::Command;
    ///
    /// let cmd = Command::new("test")
    ///     .set_handler(|_options, _args| {
    ///         println!("This is a test");
    ///         Ok(())
    /// });
    /// ```
    pub fn set_handler<F>(mut self, f: F, ) -> Self
        where F: FnMut(&Options, &[String]) -> Result<()> + 'static {
        self.handler = Some(Rc::new(RefCell::new(f)));
        self
    }

    /// Adds a new child `Command`.
    pub fn set_command(mut self, mut command: Command) -> Self {
        command.parent = Some(Symbol::Command(self.name.clone()));
        self.children.insert(command);
        self
    }

    #[inline]
    pub(crate) fn add_command(&mut self, mut command: Command) {
        command.parent = Some(Symbol::Command(self.name.clone()));
        self.children.insert(command);
    }

    #[inline]
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
            .field("name", &self.name())
            .field("description", &self.description())
            .field("parent", &self.parent)
            .field("arguments", &self.args())
            .field(
                "handler",
                if self.handler.is_some() {
                    &"fn(&Options, &[String) -> Result<()>"
                } else {
                    &"None"
                },
            )
            .field("children", &self.children())
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

mod cmd_mut {
    use super::*;

    pub struct CommandMut<'a> {
        command: &'a mut Command,
    }

    impl<'a> CommandMut<'a> {
        fn new(command: &'a mut Command) -> Self {
            CommandMut { command }
        }

        pub fn add_description(&mut self, description: &str) {
            self.command.description = Some(description.to_string());
        }

        pub fn add_command(&mut self, command: Command) -> bool {
            self.command.children.insert(command)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::DerefMut;

    #[test]
    fn name_test() {
        let cmd = Command::new("time");
        assert_eq!(cmd.name(), "time");
    }

    #[test]
    fn description() {
        let cmd = Command::new("time").set_description("Shows the time");

        assert_eq!(cmd.description(), Some("Shows the time"));
    }

    #[test]
    fn children_test() {
        let cmd = Command::new("data")
            .set_command(Command::new("set"))
            .set_command(Command::new("get").set_command(Command::new("first")));

        assert_eq!(cmd.children().count(), 2);
        assert_eq!(cmd.get_child("set"), Some(&Command::new("set")));
        assert_eq!(cmd.get_child("get"), Some(&Command::new("get")));
        assert_eq!(
            cmd.get_child("get").unwrap().get_child("first"),
            Some(&Command::new("first"))
        );
    }

    #[test]
    fn option_test() {
        let cmd = Command::new("time")
            .set_option(CommandOption::new("version").set_alias("v"))
            .set_option(CommandOption::new("day_of_week").set_alias("dw"));

        assert_eq!(
            cmd.options().get("version"),
            Some(&CommandOption::new("version"))
        );
        assert_eq!(cmd.options().get("v"), Some(&CommandOption::new("version")));
    }

    #[test]
    fn args_test() {
        let mut cmd = Command::new("time").set_args(Arguments::new(1));

        assert_eq!(cmd.args(), &Arguments::new(1));

        assert!(cmd.set_args_values(&["1"]).is_ok());
        assert!(cmd.args().values().contains(&String::from("1")));
    }

    #[test]
    fn handle_test() {
        static mut VALUE: usize = 0;

        let cmd = Command::new("counter").set_handler(inc);

        fn inc(_: &Options, _: &[String]) -> Result<()> {
            unsafe { VALUE += 1 };
            Ok(())
        }

        let opts = Options::new();
        let args = Vec::<String>::new();

        cmd.handler().unwrap().deref_mut()(&opts, args.as_slice()).unwrap();
        cmd.handler().unwrap().deref_mut()(&opts, args.as_slice()).unwrap();

        assert_eq!(unsafe { VALUE }, 2);
    }
}
