#![allow(clippy::len_zero)]
use crate::args::{Argument, ArgumentList};
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
use std::ops::Index;
use std::str::FromStr;
use crate::{Error, ErrorKind, Result};

/// Represents a command-line option.
#[derive(Debug, Clone)]
pub struct CommandOption {
    name: String,
    aliases: Vec<String>,
    description: Option<String>,
    args: ArgumentList,
    is_required: bool,
    is_hidden: bool,
    allow_multiple: bool,
    requires_assign: bool,
}

impl CommandOption {
    /// Constructs a new `CommandOption`.
    ///
    /// # Panics:
    /// Panics if the `name` is empty.
    ///
    /// # Example
    /// ```
    /// use clapi::{Command, CommandOption};
    ///
    /// let result = Command::new("MyApp")
    ///     .option(CommandOption::new("enable"))
    ///     .parse_from(vec!["--enable"])
    ///     .unwrap();
    ///
    /// assert!(result.options().contains("enable"));
    /// ```
    pub fn new<S: Into<String>>(name: S) -> Self {
        let name = name.into();
        assert!(!name.is_empty(), "option `name` cannot be empty");

        CommandOption {
            name,
            aliases: Vec::new(),
            description: None,
            args: ArgumentList::new(),
            is_required: false,
            is_hidden: false,
            allow_multiple: false,
            requires_assign: false,
        }
    }

    /// Returns the name of this option.
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns an `Iterator` over the aliases of this option.
    pub fn get_aliases(&self) -> Aliases<'_> {
        Aliases {
            iter: self.aliases.iter(),
        }
    }

    /// Returns a short description of this option or `None` if not set.
    pub fn get_description(&self) -> Option<&str> {
        self.description.as_ref().map(|s| s.as_ref())
    }

    /// Returns `true` if this option is required.
    pub fn is_required(&self) -> bool {
        self.is_required
    }

    /// Returns `true` if this option is no visible for `help`.
    pub fn is_hidden(&self) -> bool {
        self.is_hidden
    }

    /// Returns `true` if this option is allowed to appear multiple times.
    pub fn allow_multiple(&self) -> bool {
        self.allow_multiple
    }

    /// Returns `true` if the option requires an assign operator.
    pub fn is_assign_required(&self) -> bool {
        self.requires_assign
    }

    /// Returns the `Argument` this option takes or `None` if have more than 1 argument.
    pub fn get_arg(&self) -> Option<&Argument> {
        if self.args.len() > 1 {
            None
        } else {
            Some(&self.args[0])
        }
    }

    /// Returns the `Arguments` of this option.
    pub fn get_args(&self) -> &ArgumentList {
        &self.args
    }

    /// Returns `true` if this option take arguments.
    pub fn take_args(&self) -> bool {
        self.args.len() > 0
    }

    /// Returns `true` if option contains the specified alias.
    pub fn has_alias<S: AsRef<str>>(&self, alias: S) -> bool {
        self.aliases.iter().any(|s| s == alias.as_ref())
    }

    /// Adds a new alias to this option.
    ///
    /// # Panics:
    /// Panics if the `alias` is empty.
    ///
    /// # Example
    /// ```
    /// use clapi::{Command, CommandOption};
    ///
    /// let result = Command::new("MyApp")
    ///     .option(CommandOption::new("test").alias("t"))
    ///     .parse_from(vec!["-t"])
    ///     .unwrap();
    ///
    /// assert!(result.options().contains("test"));
    /// ```
    pub fn alias<S: Into<String>>(mut self, alias: S) -> Self {
        let alias = alias.into();
        assert!(!alias.is_empty(), "option `alias` cannot be empty");
        self.aliases.push(alias);
        self
    }

    /// Sets a short description of this option.
    ///
    /// # Example
    /// ```
    /// use clapi::CommandOption;
    ///
    /// let option = CommandOption::new("test")
    ///     .description("Enable tests");
    ///
    /// assert_eq!(option.get_description(), Some("Enable tests"));
    /// ```
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Specify if this option is required, by default is `false`.
    ///
    /// # Examples
    /// ```
    /// use clapi::{Command, CommandOption, Argument};
    /// use clapi::validator::validate_type;
    ///
    /// let result = Command::new("MyApp")
    ///     .option(CommandOption::new("test"))
    ///     .option(CommandOption::new("number")
    ///         .required(true)
    ///         .arg(Argument::new()
    ///             .validator(validate_type::<i64>())))
    ///     .parse_from(vec!["--test", "--number", "10"])
    ///     .unwrap();
    ///
    /// assert!(result.options().get_arg("number").unwrap().contains("10"));
    /// assert!(result.options().contains("test"));
    /// ```
    ///
    /// Other example where the option is ommited
    /// ```
    /// use clapi::{Command, CommandOption, Argument};
    /// use clapi::validator::validate_type;
    ///
    /// let result = Command::new("MyApp")
    ///     .option(CommandOption::new("test"))
    ///     .option(CommandOption::new("number")
    ///         .required(true)
    ///         .arg(Argument::new()
    ///             .validator(validate_type::<i64>())))
    ///     .parse_from(vec!["--test"]);
    ///
    /// assert!(result.is_err());
    /// ```
    pub fn required(mut self, is_required: bool) -> Self {
        self.is_required = is_required;
        self
    }

    /// Specify if this option is hidden for the `help`.
    ///
    /// # Example
    /// ```
    /// use clapi::CommandOption;
    ///
    /// let option = CommandOption::new("enable").hidden(true);
    /// assert!(option.is_hidden());
    /// ```
    pub fn hidden(mut self, is_hidden: bool) -> Self {
        self.is_hidden = is_hidden;
        self
    }

    /// Specify if this option can appear multiple times.
    ///
    /// # Example
    /// ```
    /// use clapi::{Command, CommandOption, Argument};
    ///
    /// let result = Command::new("MyApp")
    ///     .option(CommandOption::new("numbers")
    ///         .multiple(true)
    ///         .arg(Argument::new().min_values(1)))
    ///     .parse_from(vec!["--numbers", "10", "--numbers", "20", "--numbers", "30"])
    ///     .unwrap();
    ///
    /// assert!(result.options().get_arg("numbers").unwrap().contains("10"));
    /// assert!(result.options().get_arg("numbers").unwrap().contains("20"));
    /// assert!(result.options().get_arg("numbers").unwrap().contains("30"));
    /// ```
    pub fn multiple(mut self, allow_multiple: bool) -> Self {
        self.allow_multiple = allow_multiple;
        self
    }

    /// Specify if this option requires an assign operator.
    ///
    /// # Example
    /// ```
    /// use clapi::{Command, CommandOption, Argument};
    ///
    /// let result = Command::new("MyApp")
    ///     .option(CommandOption::new("numbers")
    ///         .requires_assign(true)
    ///         .arg(Argument::new().min_values(1)))
    ///     .parse_from(vec!["--numbers=10,20,30"])
    ///     .unwrap();
    ///
    /// assert!(result.options().get_arg("numbers").unwrap().contains("10"));
    /// assert!(result.options().get_arg("numbers").unwrap().contains("20"));
    /// assert!(result.options().get_arg("numbers").unwrap().contains("30"));
    /// ```
    ///
    /// Using it like this will fail
    /// ```
    /// use clapi::{Command, CommandOption, Argument};
    ///
    /// let result = Command::new("MyApp")
    ///     .option(CommandOption::new("numbers")
    ///         .requires_assign(true)
    ///         .arg(Argument::new().min_values(1)))
    ///     .parse_from(vec!["--numbers", "10", "20", "30"]);
    ///
    /// assert!(result.is_err());
    /// ```
    pub fn requires_assign(mut self, requires_assign: bool) -> Self {
        self.requires_assign = requires_assign;
        self
    }

    /// Adds a new `Argument` to this option.
    ///
    /// # Example
    /// ```
    /// use clapi::{Command, CommandOption, Argument};
    ///
    /// let result = Command::new("MyApp")
    ///     .option(CommandOption::new("copy")
    ///         .arg(Argument::with_name("from"))
    ///         .arg(Argument::with_name("to")))
    ///     .parse_from(vec!["--copy", "/src/file.txt", "/src/utils/"])
    ///     .unwrap();
    ///
    /// assert!(result.options().get_args("copy").unwrap().get("from").unwrap().contains("/src/file.txt"));
    /// assert!(result.options().get_args("copy").unwrap().get("to").unwrap().contains("/src/utils/"));
    /// ```
    pub fn arg(mut self, mut arg: Argument) -> Self {
        arg.set_name_and_description_if_none(self.get_name(), self.get_description());

        if let Err(duplicated) = self.args.add(arg) {
            panic!(
                "`{}` already contains an argument named: `{}`",
                self.name,
                duplicated.get_name()
            );
        }
        self
    }

    /// Sets the arguments of this option.
    ///
    /// # Example
    /// ```
    /// use clapi::{ArgumentList, Argument, CommandOption};
    ///
    /// let mut args = ArgumentList::new();
    /// args.add(Argument::with_name("from")).unwrap();
    /// args.add(Argument::with_name("to")).unwrap();
    ///
    /// let option = CommandOption::new("copy").args(args);
    /// assert!(option.get_args().contains("from"));
    /// assert!(option.get_args().contains("to"));
    /// ```
    pub fn args(mut self, args: ArgumentList) -> Self {
        self.args = args;
        self
    }
}

impl Eq for CommandOption {}

impl PartialEq for CommandOption {
    fn eq(&self, other: &Self) -> bool {
        // This implementation is enough for the purposes of the library
        // but don't reflect the true equality of this struct
        self.name == other.name
    }
}

impl Hash for CommandOption {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.name.as_bytes())
    }
}

/// An iterator over the aliases of `CommandOption`.
#[derive(Debug, Clone)]
pub struct Aliases<'a> {
    iter: std::slice::Iter<'a, String>
}

impl<'a> Iterator for Aliases<'a> {
    type Item = &'a String;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a> ExactSizeIterator for Aliases<'a> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

/// Represents a collection of `CommandOption`s.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct OptionList {
    inner: Vec<CommandOption>,
}

impl OptionList {
    /// Constructs a new empty `Options`.
    pub fn new() -> Self {
        OptionList {
            inner: vec![],
        }
    }

    /// Adds the specified `CommandOption`.
    ///
    /// # Returns
    /// `false` if there is an option with the same alias than the provided one.
    pub fn add(&mut self, option: CommandOption) -> std::result::Result<(), CommandOption> {
        if self.is_option_duplicate(&option) {
            return Err(option);
        }

        self.inner.push(option);
        Ok(())
    }

    /// Adds the specified `CommandOption` or replace it it already exists,
    pub fn add_or_replace(&mut self, option: CommandOption) {
        if self.inner.contains(&option) {
            let pos = self.inner.iter().position(|o| o.get_name() == option.get_name()).unwrap();
            self.inner[pos] = option;
        } else {
            self.add(option).unwrap();
        }
    }

    /// Returns the `CommandOption` with the given name or alias or `None`
    /// if not found.
    pub fn get<S: AsRef<str>>(&self, name_or_alias: S) -> Option<&CommandOption> {
        self.inner.iter().find(|o| {
            o.name == name_or_alias.as_ref() || o.get_aliases().any(|s| s == name_or_alias.as_ref())
        })
    }

    /// Returns the `CommandOption` with the given name or `None` if not found.
    pub fn get_by_name<S: AsRef<str>>(&self, name: S) -> Option<&CommandOption> {
        self.inner
            .iter()
            .find(|opt| opt.get_name() == name.as_ref())
    }

    /// Returns the `CommandOption` with the given alias or `None` if not found.
    pub fn get_by_alias<S: AsRef<str>>(&self, alias: S) -> Option<&CommandOption> {
        self.inner.iter().find(|opt| opt.has_alias(alias.as_ref()))
    }

    /// Converts the argument value of the given option to the type `T` or results `Err` if:
    /// * The option is not found.
    /// * The option takes no arguments.
    /// * The option takes more than 1 argument.
    /// * The argument value parse fail.
    pub fn convert<T>(&self, option: &str) -> Result<T>
    where
        T: FromStr + 'static,
        <T as FromStr>::Err: Display {
        match self.get(option) {
            Some(opt) => {
                opt.get_arg()
                    .unwrap_or_else(|| panic!("`{}` takes no arguments", option))
                    .convert()
            },
            None => Err(Error::new(
                ErrorKind::Other,
                format!("cannot find option named '{}'", option))
            )
        }
    }

    /// Converts all the argument values of the given option to the type `T` or results `Err` if:
    /// * The option is not found.
    /// * The option takes no arguments.
    /// * The option takes more than 1 argument.
    /// * The argument values parse fail.
    pub fn convert_all<T>(&self, option: &str) -> Result<Vec<T>>
        where
            T: FromStr + 'static,
            <T as FromStr>::Err: Display {
        match self.get(option) {
            Some(opt) => {
                opt.get_arg()
                    .unwrap_or_else(|| panic!("`{}` takes no arguments", option))
                    .convert_all()
            },
            None => Err(Error::new(
                ErrorKind::Other,
                format!("cannot find option named '{}'", option))
            )
        }
    }

    /// Returns the `Argument` of the option with the given name or alias or
    /// `None` if the option cannot be found or have more than 1 argument.
    pub fn get_arg<S: AsRef<str>>(&self, option: S) -> Option<&Argument> {
        self.get(option.as_ref()).map(|o| o.get_arg()).flatten()
    }

    /// Returns the `ArgumentList` of the option with the given name or alias, or `None`
    /// if the option canno tbe found.
    pub fn get_args<S: AsRef<str>>(&self, option: S) -> Option<&ArgumentList> {
        self.get(option.as_ref()).map(|o| o.get_args())
    }

    /// Returns `true` if there is an option with the given name or alias.
    pub fn contains<S: AsRef<str>>(&self, option: S) -> bool {
        self.get(option).is_some()
    }

    /// Returns the number of options in this collection.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if there is no options.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Removes all the `Option`s.
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Returns an `ExactSizeIterator` over the `CommandOption` of this collection.
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            iter: self.inner.iter(),
        }
    }

    fn is_option_duplicate(&self, option: &CommandOption) -> bool {
        // Check if there if any option that match the new option `alias` or `name`
        self.contains(&option.name) || option.get_aliases().any(|alias| self.contains(alias))
    }
}

/// An iterator over the `CommandOption`s of an option list.
#[derive(Debug, Clone)]
pub struct Iter<'a> {
    iter: std::slice::Iter<'a, CommandOption>,
}

/// An owning iterator over the `CommandOption`s of an option list.
pub struct IntoIter {
    iter: std::vec::IntoIter<CommandOption>
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a CommandOption;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl Iterator for IntoIter {
    type Item = CommandOption;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a> ExactSizeIterator for Iter<'a> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl ExactSizeIterator for IntoIter {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a> IntoIterator for &'a OptionList {
    type Item = &'a CommandOption;
    type IntoIter = Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for OptionList {
    type Item = CommandOption;
    type IntoIter = IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.inner.into_iter(),
        }
    }
}

impl Index<&str> for OptionList {
    type Output = CommandOption;

    fn index(&self, index: &str) -> &Self::Output {
        match self.get(index) {
            Some(option) => option,
            None => panic!("cannot find option named: `{}`", index),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "option `name` cannot be empty")]
    fn option_empty_name_test() {
        CommandOption::new("");
    }

    #[test]
    #[should_panic(expected = "option `alias` cannot be empty")]
    fn option_empty_alias_test() {
        CommandOption::new("test").alias("");
    }

    #[test]
    fn option_name_with_whitespaces_test() {
        CommandOption::new("my option");
    }

    #[test]
    fn option_alias_with_whitespaces_test() {
        CommandOption::new("test").alias("m o");
    }

    #[test]
    fn alias_test() {
        let opt = CommandOption::new("name").alias("n").alias("nm");

        assert_eq!(opt.get_name(), "name");

        assert!(opt.has_alias("n"));
        assert!(opt.has_alias("nm"));

        assert!(opt.get_aliases().any(|s| s == "n"));
        assert!(opt.get_aliases().any(|s| s == "nm"));
        assert!(!opt.get_aliases().any(|s| s == "name"));
    }

    #[test]
    fn description_test() {
        let opt = CommandOption::new("date").description("Sets the date");

        assert_eq!(opt.get_description(), Some("Sets the date"));
    }

    #[test]
    fn is_required_test() {
        let opt1 = CommandOption::new("date");
        assert!(!opt1.is_required());

        let opt2 = opt1.clone().required(true);
        assert!(opt2.is_required());
    }

    #[test]
    fn is_hidden_test() {
        let opt1 = CommandOption::new("help");
        assert!(!opt1.is_hidden());

        let opt2 = CommandOption::new("help").hidden(true);
        assert!(opt2.is_hidden());
    }

    #[test]
    fn allow_multiple_test() {
        let opt1 = CommandOption::new("values");
        assert!(!opt1.allow_multiple());

        let opt2 = CommandOption::new("values").multiple(true);
        assert!(opt2.allow_multiple());
    }

    #[test]
    fn require_assign_test() {
        let opt1 = CommandOption::new("values");
        assert!(!opt1.is_assign_required());

        let opt2 = CommandOption::new("values").requires_assign(true);
        assert!(opt2.is_assign_required());
    }

    #[test]
    fn args_test() {
        let opt1 = CommandOption::new("date");

        let opt2 = opt1
            .clone()
            .arg(Argument::with_name("value").valid_values(&["day", "hour", "minute"]));

        assert!(opt2.get_arg().unwrap().is_valid("day"));
        assert!(opt2.get_arg().unwrap().is_valid("hour"));
        assert!(opt2.get_arg().unwrap().is_valid("minute"));
        assert!(!opt2.get_arg().unwrap().is_valid("second"));
    }

    #[test]
    fn options_add_test() {
        let mut options = OptionList::new();
        assert!(options.is_empty());

        assert!(options
            .add(CommandOption::new("version").alias("v"))
            .is_ok());
        assert!(options.add(CommandOption::new("author").alias("a")).is_ok());
        assert!(options.add(CommandOption::new("verbose")).is_ok());
        assert_eq!(options.len(), 3);
    }

    #[test]
    fn options_get_test() {
        let mut options = OptionList::new();
        options
            .add(CommandOption::new("version").alias("v"))
            .unwrap();
        options
            .add(CommandOption::new("author").alias("a"))
            .unwrap();
        options.add(CommandOption::new("verbose")).unwrap();

        assert_eq!(options.get("version"), Some(&CommandOption::new("version")));
        assert_eq!(options.get("v"), Some(&CommandOption::new("version")));
        assert_eq!(options.get("author"), Some(&CommandOption::new("author")));
        assert_eq!(options.get("ve"), None);
    }

    #[test]
    fn options_contains_test() {
        let mut options = OptionList::new();
        options
            .add(CommandOption::new("version").alias("v"))
            .unwrap();
        options
            .add(CommandOption::new("author").alias("a"))
            .unwrap();
        options.add(CommandOption::new("verbose")).unwrap();

        assert!(options.contains("version"));
        assert!(options.contains("v"));
        assert!(options.contains("author"));
        assert!(options.contains("a"));
        assert!(options.contains("verbose"));
    }

    #[test]
    fn options_get_args_test() {
        let mut options = OptionList::new();

        let opt1 = CommandOption::new("version")
            .alias("v")
            .arg(Argument::with_name("version").values_count(1));

        let opt2 = CommandOption::new("author")
            .alias("a")
            .arg(Argument::with_name("x").values_count(0..));

        let opt3 = CommandOption::new("verbose").arg(Argument::with_name("x").values_count(1..3));

        options.add(opt1).unwrap();
        options.add(opt2).unwrap();
        options.add(opt3).unwrap();

        assert_eq!(options.len(), 3);
        assert!(options.get_arg("version").is_some());
        assert!(options.get_arg("author").is_some());
        assert!(options.get_arg("verbose").is_some());
    }

    #[test]
    fn options_add_duplicated_test() {
        let mut options = OptionList::new();
        options
            .add(CommandOption::new("version").alias("v"))
            .unwrap();

        assert!(options.add(CommandOption::new("version")).is_err());
        assert!(options.add(CommandOption::new("v")).is_err());
        assert!(options
            .add(CommandOption::new("V").alias("version"))
            .is_err());
        assert!(options.add(CommandOption::new("value").alias("v")).is_err());
    }

    #[test]
    fn option_list_indexer_test() {
        let mut options = OptionList::new();
        options.add(CommandOption::new("number")).unwrap();
        options.add(CommandOption::new("enable")).unwrap();

        assert_eq!(options["number"].get_name(), "number");
        assert_eq!(options["enable"].get_name(), "enable");
    }
}
