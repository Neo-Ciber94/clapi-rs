#![allow(clippy::len_zero)]
use crate::args::{Argument, ArgumentList};
use linked_hash_set::LinkedHashSet;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

/// Represents a command-line option.
#[derive(Debug, Clone)]
pub struct CommandOption {
    name: String,
    aliases: LinkedHashSet<String>,
    description: Option<String>,
    args: ArgumentList,
    is_required: bool,
    is_hidden: bool,
    allow_multiple: bool,
}

impl CommandOption {
    /// Constructs a new `CommandOption`.
    ///
    /// # Panics:
    /// Panics if the `name` is blank or empty.
    pub fn new<S: Into<String>>(name: S) -> Self {
        let name = name.into();
        assert_contains_no_whitespaces!(name);

        CommandOption {
            name,
            aliases: LinkedHashSet::new(),
            description: None,
            args: ArgumentList::new(),
            is_required: false,
            is_hidden: false,
            allow_multiple: false,
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
        self.aliases.contains(alias.as_ref())
    }

    /// Adds a new alias to this option.
    ///
    /// # Panics:
    /// Panics if the `alias` is empty or contains whitespaces.
    pub fn alias<S: Into<String>>(mut self, alias: S) -> Self {
        let alias = alias.into();
        assert_contains_no_whitespaces!(alias);
        self.aliases.insert(alias);
        self
    }

    /// Sets a short description of this option.
    ///
    /// # Panics:
    /// Panics if the `description` is blank or empty.
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(assert_not_blank!(
            description.into(),
            "`description` cannot be blank or empty"
        ));
        self
    }

    /// Specify if this option is required, by default is `false`.
    pub fn required(mut self, is_required: bool) -> Self {
        self.is_required = is_required;
        self
    }

    /// Specify if this option is hidden for the `help`.
    pub fn hidden(mut self, is_hidden: bool) -> Self {
        self.is_hidden = is_hidden;
        self
    }

    /// Specify if this option can appear multiple times.
    pub fn multiple(mut self, allow_multiple: bool) -> Self {
        self.allow_multiple = allow_multiple;
        self
    }

    /// Adds a new `Argument` to this option.
    pub fn arg(mut self, mut arg: Argument) -> Self {
        arg.set_name_and_description_if_none(self.get_name(), self.get_description());

        if let Err(duplicated) = self.args.add(arg) {
            panic!("`{}` already contains an argument named: `{}`", self.name, duplicated.get_name());
        }
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
    iter: linked_hash_set::Iter<'a, String>,
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
    pub fn add(&mut self, option: CommandOption) -> std::result::Result<(), CommandOption> {
        if self.is_option_duplicate(&option) {
            return Err(option);
        }

        self.inner.insert_if_absent(option);
        Ok(())
    }

    /// Adds the specified `CommandOption` or replace it it already exists,
    pub fn add_or_replace(&mut self, option: CommandOption) {
        if self.inner.contains(&option) {
            self.inner.remove(&option);
        }

        self.inner.insert(option);
    }

    /// Returns the `CommandOption` with the given name or alias or `None`
    /// if not found.
    pub fn get<S: AsRef<str>>(&self, name_or_alias: S) -> Option<&CommandOption> {
        self.inner.iter().find(|o| {
            o.name == name_or_alias.as_ref() || o.aliases.contains(name_or_alias.as_ref())
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

    /// Returns the `Argument` of the option with the given name or alias or
    /// `None` if the option cannot be found or have more than 1 argument.
    pub fn get_arg<S: AsRef<str>>(&self, name_or_alias: S) -> Option<&Argument> {
        self.get(name_or_alias.as_ref())
            .map(|o| o.get_arg())
            .flatten()
    }

    /// Returns the `ArgumentList` of the option with the given name or alias, or `None`
    /// if the option canno tbe found.
    pub fn get_args<S: AsRef<str>>(&self, name_or_alias: S) -> Option<&ArgumentList> {
        self.get(name_or_alias.as_ref()).map(|o| o.get_args())
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
    iter: linked_hash_set::Iter<'a, CommandOption>,
}

/// An owning iterator over the `CommandOption`s of an option list.
pub struct IntoIter {
    iter: linked_hash_set::IntoIter<CommandOption>,
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
    fn is_required_test() {
        let opt1 = CommandOption::new("date");
        assert!(!opt1.is_required());

        let opt2 = opt1.clone().required(true);
        assert!(opt2.is_required());
    }

    #[test]
    fn is_hidden_test(){
        let opt1 = CommandOption::new("help");
        assert!(!opt1.is_hidden());

        let opt2 = CommandOption::new("help").hidden(true);
        assert!(opt2.is_hidden());
    }

    #[test]
    fn allow_multiple_test(){
        let opt1 = CommandOption::new("values");
        assert!(!opt1.allow_multiple());

        let opt2 = CommandOption::new("values").multiple(true);
        assert!(opt2.allow_multiple());
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
}
