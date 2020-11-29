use linked_hash_set::LinkedHashSet;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use crate::args::{ArgumentList, Argument};

/// Represents a command-line option.
#[derive(Debug, Clone)]
pub struct CommandOption {
    name: String,
    aliases: LinkedHashSet<String>,
    description: Option<String>,
    is_required: bool,
    args: ArgumentList,
}

impl CommandOption {
    /// Constructs a new `CommandOption`.
    pub fn new<S: Into<String>>(name: S) -> Self {
        let name = name.into();
        assert!(!name.trim().is_empty(), "name cannot be empty");

        CommandOption {
            name,
            aliases: LinkedHashSet::new(),
            description: None,
            args: ArgumentList::new(),
            is_required: false,
        }
    }

    /// Returns the name of this option.
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns an `Iterator` over the aliases of this option.
    pub fn get_aliases(&self) -> impl ExactSizeIterator<Item = &'_ String> + Debug {
        self.aliases.iter()
    }

    /// Returns a short description of this option or `None` if not set.
    pub fn get_description(&self) -> Option<&str> {
        self.description.as_ref().map(|s| s.as_ref())
    }

    /// Returns `true` if this option is required.
    pub fn is_required(&self) -> bool {
        self.is_required
    }

    /// Returns the `Argument` this option takes or `None` if have more than 1 argument.
    pub fn get_arg(&self) -> Option<&Argument>{
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
        self.aliases.contains(alias.as_ref())
    }

    /// Adds a new alias to this option.
    pub fn alias<S: AsRef<str>>(mut self, alias: S) -> Self {
        self.aliases.insert(alias.as_ref().to_string());
        self
    }

    /// Sets a short description of this option.
    pub fn description<S: AsRef<str>>(mut self, description: S) -> Self {
        self.description = Some(description.as_ref().to_string());
        self
    }

    /// Specify if this option is required, by default is `false`.
    pub fn required(mut self, is_required: bool) -> Self {
        self.is_required = is_required;
        self
    }

    /// Adds a new `Argument` to this option.
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

    /// Adds a new `Argument` to this option.
    #[cfg(not(debug_assertions))]
    pub fn arg(mut self, arg: Argument) -> Self {
        self.args.add(arg);
        self
    }

    /// Sets the arguments of this option.
    pub fn args(mut self, args: ArgumentList) -> Self {
        self.args = args;
        self
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
#[derive(Default, Debug, Clone)]
pub struct OptionList {
    inner: LinkedHashSet<CommandOption>,
}

impl OptionList {
    /// Constructs a new empty `Options`.
    pub fn new() -> Self {
        OptionList {
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
            if option.aliases.contains(opt.get_name()) || opt.aliases.contains(option.get_name()){
                return false;
            }
        }

        self.inner.insert_if_absent(option)
    }

    /// Returns the `CommandOption` with the given name or alias or `None`
    /// if not found.
    pub fn get<S: AsRef<str>>(&self, name_or_alias: S) -> Option<&CommandOption> {
        self.inner
            .iter()
            .find(|o| o.name == name_or_alias.as_ref() || o.aliases.contains(name_or_alias.as_ref()))
    }

    /// Returns the `CommandOption` with the given name or `None` if not found.
    pub fn get_by_name<S: AsRef<str>>(&self, name: S) -> Option<&CommandOption> {
        self.inner.iter()
            .find(|opt| opt.get_name() == name.as_ref())
    }

    /// Returns the `CommandOption` with the given alias or `None` if not found.
    pub fn get_by_alias<S: AsRef<str>>(&self, alias: S) -> Option<&CommandOption> {
        self.inner.iter()
            .find(|opt| opt.has_alias(alias.as_ref()))
    }

    /// Returns the `Argument` of the option with the given name or alias or
    /// `None` if the option cannot be found or have more than 1 argument.
    pub fn get_arg<S: AsRef<str>>(&self, name_or_alias: S) -> Option<&Argument> {
        self.get(name_or_alias.as_ref())
            .map(|o| o.get_arg())
            .flatten()
    }

    /// Returns the `ArgumentList` of the option with the given name or alias, or `None`
    /// if the option canno tbe found.
    pub fn get_args<S: AsRef<str>>(&self, name_or_alias: S) -> Option<&ArgumentList>{
        self.get(name_or_alias.as_ref())
            .map(|o| o.get_args())
    }

    /// Returns `true` if there is an option with the given name or alias.
    pub fn contains<S: AsRef<str>>(&self, name_or_alias: S) -> bool {
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

impl<'a> IntoIterator for &'a OptionList {
    type Item = &'a CommandOption;
    type IntoIter = linked_hash_set::Iter<'a, CommandOption>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl IntoIterator for OptionList {
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
    fn required_test() {
        let opt1 = CommandOption::new("date");
        assert!(!opt1.is_required());

        let opt2 = opt1.clone().required(true);
        assert!(opt2.is_required());
    }

    #[test]
    fn args_test() {
        let opt1 = CommandOption::new("date");

        let opt2 = opt1
            .clone()
            .arg(Argument::new("value").valid_values(&["day", "hour", "minute"]));

        assert!(opt2.get_arg().unwrap().is_valid("day"));
        assert!(opt2.get_arg().unwrap().is_valid("hour"));
        assert!(opt2.get_arg().unwrap().is_valid("minute"));
        assert!(!opt2.get_arg().unwrap().is_valid("second"));
    }

    #[test]
    fn options_add_test() {
        let mut options = OptionList::new();
        assert!(options.is_empty());

        assert!(options.add(CommandOption::new("version").alias("v")));
        assert!(options.add(CommandOption::new("author").alias("a")));
        assert!(options.add(CommandOption::new("verbose")));
        assert_eq!(options.len(), 3);
    }

    #[test]
    fn options_get_test() {
        let mut options = OptionList::new();
        options.add(CommandOption::new("version").alias("v"));
        options.add(CommandOption::new("author").alias("a"));
        options.add(CommandOption::new("verbose"));

        assert_eq!(options.get("version"), Some(&CommandOption::new("version")));
        assert_eq!(options.get("v"), Some(&CommandOption::new("version")));
        assert_eq!(options.get("author"), Some(&CommandOption::new("author")));
        assert_eq!(options.get("ve"), None);
    }

    #[test]
    fn options_contains_test() {
        let mut options = OptionList::new();
        options.add(CommandOption::new("version").alias("v"));
        options.add(CommandOption::new("author").alias("a"));
        options.add(CommandOption::new("verbose"));

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
            .arg(Argument::new("version").arg_count(1));

        let opt2 = CommandOption::new("author")
            .alias("a")
            .arg(Argument::new("x").arg_count(0..));

        let opt3 = CommandOption::new("verbose")
            .arg(Argument::new("x").arg_count(1..3));

        options.add(opt1);
        options.add(opt2);
        options.add(opt3);

        assert_eq!(options.len(), 3);
        assert!(options.get_arg("version").is_some());
        assert!(options.get_arg("author").is_some());
        assert!(options.get_arg("verbose").is_some());
    }
}
