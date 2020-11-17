use crate::args::Arguments;
use crate::error::Result;
use crate::symbol::Symbol;
use crate::utils::Also;
use linked_hash_set::LinkedHashSet;
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
use std::str::FromStr;

/// Represents a command-line option.
#[derive(Debug, Clone)]
pub struct CommandOption {
    name: String,
    aliases: LinkedHashSet<String>,
    description: Option<String>,
    is_required: bool,
    args: Arguments,
}

impl CommandOption {
    /// Constructs a new `CommandOption`.
    pub fn new(name: &str) -> Self {
        assert!(!name.trim().is_empty(), "name cannot be empty");

        let args =
            Arguments::none().also_mut(|a| a.parent = Some(Symbol::Option(name.to_string())));

        CommandOption {
            name: name.to_string(),
            aliases: LinkedHashSet::new(),
            description: None,
            args,
            is_required: false,
        }
    }

    /// Returns the name of this option.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns an `Iterator` over the aliases of this option.
    pub fn aliases(&self) -> impl ExactSizeIterator<Item = &'_ String> + Debug {
        self.aliases.iter()
    }

    /// Returns a short description of this option or `None` if not set.
    pub fn description(&self) -> Option<&str> {
        self.description.as_ref().map(|s| s.as_str())
    }

    /// Returns `true` if this option is required.
    pub fn is_required(&self) -> bool {
        self.is_required
    }

    /// Returns the `Arguments` of this option.
    pub fn args(&self) -> &Arguments {
        &self.args
    }

    /// Returns `true` if this option take arguments.
    pub fn take_args(&self) -> bool {
        self.args.take_args()
    }

    /// Returns `true` if option contains the specified alias.
    pub fn has_alias(&self, alias: &str) -> bool {
        self.aliases.contains(alias)
    }

    /// Adds a new alias to this option.
    pub fn set_alias(mut self, alias: &str) -> Self {
        self.aliases.insert(alias.to_string());
        self
    }

    /// Sets a short description of this option.
    pub fn set_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Specify if this option is required, by default is `false`.
    pub fn set_required(mut self, is_required: bool) -> Self {
        self.is_required = is_required;
        self
    }

    /// Sets the `Arguments` of this option.
    pub fn set_args(mut self, mut args: Arguments) -> Self {
        args.parent = Some(Symbol::Option(self.name.clone()));
        self.args = args;
        self
    }

    /// Sets the argument values of this option.
    pub fn set_args_values<'a, S, I>(&mut self, args: I) -> Result<()>
    where
        I: IntoIterator<Item = &'a S>,
        S: ToString + 'a,
    {
        self.args.set_values(args)
    }

    /// Converts the first argument value into the specified type.
    ///
    /// # Error
    /// - If there is no values to convert.
    /// - If this takes not args.
    /// - The value cannot be converted to type `T`.
    #[inline]
    pub fn arg_as<T>(&self) -> Result<T>
    where
        T: FromStr + 'static,
        <T as FromStr>::Err: Display,
    {
        self.args.convert()
    }

    /// Returns an iterator that converts the argument values into the specified type.
    ///
    /// # Error
    /// - If there is no values to convert.
    /// - If this takes not args.
    /// - One of the values cannot be converted to type `T`.
    #[inline]
    pub fn args_as<T>(&self) -> Result<Vec<T>>
    where
        T: FromStr + 'static,
        <T as FromStr>::Err: Display,
    {
        self.args.convert_all()
    }
}

impl Eq for CommandOption {}

impl PartialEq for CommandOption {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Hash for CommandOption {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.name.as_bytes())
    }
}

/// Represents a collection of `CommandOption`s.
#[derive(Debug, Clone)]
pub struct Options {
    inner: LinkedHashSet<CommandOption>,
}

impl Options {
    /// Constructs a new empty `Options`.
    pub fn new() -> Self {
        Options {
            inner: LinkedHashSet::new(),
        }
    }

    /// Adds the specified `CommandOption`.
    ///
    /// # Returns
    /// `false` if there is an option with the same alias than the provided one.
    pub fn add(&mut self, option: CommandOption) -> bool {
        // Check for duplicated aliases
        for alias in &option.aliases {
            if self.contains(alias){
                return false;
            }
        }

        // Check if any of the aliases is equals to the option name
        for opt in &self.inner {
            if option.aliases.contains(opt.name()) || opt.aliases.contains(option.name()){
                return false;
            }
        }

        self.inner.insert(option)
    }

    /// Returns the `CommandOption` with the given name or alias or `None`
    /// if not found.
    pub fn get(&self, name_or_alias: &str) -> Option<&CommandOption> {
        self.inner
            .iter()
            .find(|o| o.name == name_or_alias || o.aliases.contains(name_or_alias))
    }

    /// Returns the `CommandOption` with the given name or `None` if not found.
    pub fn get_by_name(&self, name: &str) -> Option<&CommandOption> {
        self.inner.iter()
            .find(|opt| opt.name() == name)
    }

    /// Returns the `CommandOption` with the given alias or `None` if not found.
    pub fn get_by_alias(&self, alias: &str) -> Option<&CommandOption> {
        self.inner.iter()
            .find(|opt| opt.has_alias(alias))
    }

    /// Returns the `Arguments` for the option with the given name or alias
    /// or `None` if the option is not found.
    pub fn get_args(&self, name_or_alias: &str) -> Option<&Arguments> {
        self.get(name_or_alias).map(|o| o.args())
    }

    /// Returns `true` if there is an option with the given name or alias.
    pub fn contains(&self, name_or_alias: &str) -> bool {
        self.get(name_or_alias).is_some()
    }

    /// Returns the number of options in this collection.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if there is no options.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns an `ExactSizeIterator` over the `CommandOption` of this collection.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &'_ CommandOption> + Debug {
        self.inner.iter()
    }
}

impl<'a> IntoIterator for &'a Options {
    type Item = &'a CommandOption;
    type IntoIter = linked_hash_set::Iter<'a, CommandOption>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl IntoIterator for Options {
    type Item = CommandOption;
    type IntoIter = linked_hash_set::IntoIter<CommandOption>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alias_test() {
        let opt = CommandOption::new("name").set_alias("n").set_alias("nm");

        assert_eq!(opt.name(), "name");

        assert!(opt.has_alias("n"));
        assert!(opt.has_alias("nm"));

        assert!(opt.aliases().any(|s| s == "n"));
        assert!(opt.aliases().any(|s| s == "nm"));
        assert!(!opt.aliases().any(|s| s == "name"));
    }

    #[test]
    fn description_test() {
        let opt = CommandOption::new("date").set_description("Sets the date");

        assert_eq!(opt.description(), Some("Sets the date"));
    }

    #[test]
    fn required_test() {
        let opt1 = CommandOption::new("date");
        assert!(!opt1.is_required());

        let opt2 = opt1.clone().set_required(true);
        assert!(opt2.is_required());
    }

    #[test]
    fn args_test() {
        let opt1 = CommandOption::new("date");
        assert!(!opt1.args().take_args());

        let mut opt2 = opt1
            .clone()
            .set_args(Arguments::new(1..).set_valid_values(&["day", "hour", "minute"]));

        assert!(opt2.args().take_args());
        assert!(opt2.args().is_valid("day"));
        assert!(opt2.args().is_valid("hour"));
        assert!(opt2.args().is_valid("minute"));

        assert!(opt2.set_args_values(&["seconds"]).is_err());
        assert!(opt2.set_args_values(&["day"]).is_ok());
        assert!(opt2.set_args_values(&["hour"]).is_ok());
        assert!(opt2.set_args_values(&["minute"]).is_ok());

        assert!(opt2.args().values().iter().any(|s| s == "minute"));
    }

    #[test]
    fn options_add_test() {
        let mut options = Options::new();
        assert!(options.is_empty());

        assert!(options.add(CommandOption::new("version").set_alias("v")));
        assert!(options.add(CommandOption::new("author").set_alias("a")));
        assert!(options.add(CommandOption::new("verbose")));
        assert_eq!(options.len(), 3);
    }

    #[test]
    fn options_get_test() {
        let mut options = Options::new();
        options.add(CommandOption::new("version").set_alias("v"));
        options.add(CommandOption::new("author").set_alias("a"));
        options.add(CommandOption::new("verbose"));

        assert_eq!(options.get("version"), Some(&CommandOption::new("version")));
        assert_eq!(options.get("v"), Some(&CommandOption::new("version")));
        assert_eq!(options.get("author"), Some(&CommandOption::new("author")));
        assert_eq!(options.get("ve"), None);
    }

    #[test]
    fn options_contains_test() {
        let mut options = Options::new();
        options.add(CommandOption::new("version").set_alias("v"));
        options.add(CommandOption::new("author").set_alias("a"));
        options.add(CommandOption::new("verbose"));

        assert!(options.contains("version"));
        assert!(options.contains("v"));
        assert!(options.contains("author"));
        assert!(options.contains("a"));
        assert!(options.contains("verbose"));
    }

    #[test]
    fn args_for_test() {
        let mut options = Options::new();

        let opt1 = CommandOption::new("version")
            .set_alias("v")
            .set_args(Arguments::new(1));

        let opt2 = CommandOption::new("author")
            .set_alias("a")
            .set_args(Arguments::new(0..));

        let opt3 = CommandOption::new("verbose").set_args(Arguments::new(1..3));

        options.add(opt1);
        options.add(opt2);
        options.add(opt3);

        assert_eq!(options.get_args("v"), Some(&Arguments::new(1)));
        assert_eq!(options.get_args("author"), Some(&Arguments::new(0..)));
        assert_eq!(options.get_args("verbose"), Some(&Arguments::new(1..3)));
        assert_eq!(options.get_args("w"), None);
    }
}
