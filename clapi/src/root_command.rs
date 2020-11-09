use crate::args::Arguments;
use crate::command::Command;
use crate::error::Result;
use crate::option::{CommandOption, Options};
use std::cell::RefMut;
use std::env::current_exe;
use std::fmt::Debug;

/// A command-line command. This is used as the root command of an application.
#[derive(Debug, Clone)]
pub struct RootCommand {
    inner: Command,
}

impl RootCommand {
    /// Constructs a new `RootCommand`.
    #[inline]
    pub fn new() -> Self {
        RootCommand::with_options(Options::new())
    }

    /// Constructs a new `RootCommand` with the specified `Options`.
    pub fn with_options(options: Options) -> Self {
        RootCommand {
            inner: Command::with_options(current_filename(), options),
        }
    }

    /// Returns the name of the command.
    #[inline]
    pub fn name(&self) -> &str {
        self.inner.name()
    }

    /// Returns the description of the command, or `None` if is not set.
    #[inline]
    pub fn description(&self) -> Option<&str> {
        self.inner.description()
    }

    /// Returns a description of the usage of this command.
    #[inline]
    pub fn help(&self) -> Option<&str> {
        self.inner.help()
    }

    /// Returns an `ExactSizeIterator` over the children of this command.
    #[inline]
    pub fn children(&self) -> impl ExactSizeIterator<Item = &'_ Command> + Debug {
        self.inner.children()
    }

    /// Returns the `Options` of this command.
    #[inline]
    pub fn options(&self) -> &Options {
        self.inner.options()
    }

    /// Returns the `Arguments` of this command.
    #[inline]
    pub fn args(&self) -> &Arguments {
        self.inner.args()
    }

    /// Returns `true` if this command take args.
    pub fn take_args(&self) -> bool {
        self.inner.take_args()
    }

    /// Returns the handler of this command, or `None` if not set.
    #[inline]
    pub fn handler(
        &self,
    ) -> Option<RefMut<'_, dyn FnMut(&Options, &Arguments) -> Result<()> + 'static>> {
        self.inner.handler()
    }

    /// Returns the child with the given name, or `None` if not child if found.
    #[inline]
    pub fn get_child(&self, name: &str) -> Option<&Command> {
        self.inner.get_child(name)
    }

    /// Sets the description of this command.
    #[inline]
    pub fn set_description(self, name: &str) -> Self {
        RootCommand {
            inner: self.inner.set_description(name),
        }
    }

    /// Sets a usage description of this command.
    #[inline]
    pub fn set_help(self, help: &str) -> Self {
        RootCommand {
            inner: self.inner.set_help(help),
        }
    }

    /// Adds an `CommandOption` to this command.
    #[inline]
    pub fn set_option(self, option: CommandOption) -> Self {
        RootCommand {
            inner: self.inner.set_option(option),
        }
    }

    /// Replaces the options of this command with the specified.
    #[inline]
    pub fn set_new_options(self, options: Options) -> Self {
        RootCommand {
            inner: self.inner.set_new_options(options),
        }
    }

    /// Sets the `Arguments` of this command.
    #[inline]
    pub fn set_args(self, args: Arguments) -> Self {
        RootCommand {
            inner: self.inner.set_args(args),
        }
    }

    /// Sets the argument values of this command.
    #[inline]
    pub fn set_args_values<'a, S, I>(&mut self, args: I) -> Result<()>
    where
        I: IntoIterator<Item = &'a S>,
        S: ToString + 'a,
    {
        self.inner.set_args_values(args)
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
    #[inline]
    pub fn set_handler<F: FnMut(&Options, &Arguments) -> Result<()> + 'static>(self, f: F) -> Self {
        RootCommand {
            inner: self.inner.set_handler(f),
        }
    }

    /// Adds a new child `Command`.
    #[inline]
    pub fn set_command(self, command: Command) -> Self {
        RootCommand {
            inner: self.inner.set_command(command),
        }
    }

    /// Converts this `RootCommand` into a `Command`.
    #[inline]
    pub fn into_inner(self) -> Command {
        self.inner
    }
}

impl From<Command> for RootCommand {
    fn from(command: Command) -> Self {
        RootCommand { inner: command }
    }
}

impl AsRef<Command> for RootCommand {
    fn as_ref(&self) -> &Command {
        &self.inner
    }
}

impl AsMut<Command> for RootCommand {
    fn as_mut(&mut self) -> &mut Command {
        &mut self.inner
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
    let path = current_exe().unwrap();
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
