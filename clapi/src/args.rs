#![allow(clippy::len_zero)]
use crate::error::Result;
use crate::{ArgCount, Error, ErrorKind};
use linked_hash_set::LinkedHashSet;
use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Index;
use std::rc::Rc;
use std::str::FromStr;
use std::slice::SliceIndex;
use validator::Type;
use validator::Validator;

#[doc(hidden)]
/// Name used for unnamed `Argument`s.
pub const ARGUMENT_DEFAULT_NAME : &str = "arg";

/// Represents the arguments of an `option` or `command`.
#[derive(Clone)]
pub struct Argument {
    name: Option<String>,
    description: Option<String>,
    values_count: Option<ArgCount>,
    validator: Option<Rc<dyn Validator>>,
    validation_error: Option<String>,
    default_values: Vec<String>,
    valid_values: Vec<String>,
    values: Option<Vec<String>>,
}

impl Argument {
    /// Constructs a new `Argument` that takes 1 value.
    pub fn new() -> Self {
        Argument {
            name: None,
            description: None,
            values_count: None,
            validator: None,
            validation_error: None,
            default_values: vec![],
            valid_values: vec![],
            values: None,
        }
    }

    /// Constructs a new `Argument` with the given name that takes 1 value.
    ///
    /// # Panics:
    /// Panics if the argument `name` is empty or contains whitespaces.
    ///
    /// # Example
    /// ```
    /// use clapi::Argument;
    ///
    /// let arg = Argument::with_name("number");
    /// assert_eq!(arg.get_name(), "number");
    /// ```
    pub fn with_name<S: Into<String>>(name: S) -> Self {
        let name = name.into();
        assert!(!name.is_empty(), "argument `name` cannot be empty");

        Argument {
            name: Some(name),
            description: None,
            values_count: None,
            validator: None,
            validation_error: None,
            default_values: vec![],
            valid_values: vec![],
            values: None,
        }
    }

    /// Constructs a new `Argument` that takes 0 or 1 values.
    ///
    /// # Panics:
    /// Panics if the argument `name` is blank or empty.
    #[inline]
    pub fn zero_or_one<S: Into<String>>(name: S) -> Self {
        Self::with_name(name).values_count(0..=1)
    }

    /// Constructs a new `Argument` that takes 0 or more values.
    ///
    /// # Panics:
    /// Panics if the argument `name` is blank or empty.
    #[inline]
    pub fn zero_or_more<S: Into<String>>(name: S) -> Self {
        Self::with_name(name).values_count(0..)
    }

    /// Constructs a new `Argument` that takes 1 or more values.
    ///
    /// # Panics:
    /// Panics if the argument `name` is blank or empty.
    #[inline]
    pub fn one_or_more<S: Into<String>>(name: S) -> Self {
        Self::with_name(name).values_count(1..)
    }

    /// Returns the name of this argument.
    pub fn get_name(&self) -> &str {
        self.name.as_deref().unwrap_or(ARGUMENT_DEFAULT_NAME)
    }

    /// Returns the description of this argument.
    pub fn get_description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Returns the number of values this argument takes.
    pub fn get_values_count(&self) -> ArgCount {
        self.values_count.unwrap_or_else(ArgCount::one)
    }

    /// Returns the value `Validator` used by this argument.
    pub fn get_validator(&self) -> Option<&dyn Validator> {
        self.validator.as_ref().map(|s| s.as_ref())
    }

    /// Returns the validation error message.
    pub fn get_validation_error(&self) -> Option<&str>{
        self.validation_error.as_deref()
    }

    /// Returns the default values of this argument or a 0-length slice if none.
    pub fn get_default_values(&self) -> &[String] {
        self.default_values.as_slice()
    }

    /// Returns the valid values of this argument or a 0-length slice if none.
    pub fn get_valid_values(&self) -> &[String] {
        self.valid_values.as_slice()
    }

    /// Returns the values of this argument or a 0-length slice if none.
    pub fn get_values(&self) -> &[String] {
        // Returns the `default_values` if `values` was not set in `set_values`
        match &self.values {
            Some(n) => n,
            None => self.default_values.as_slice()
        }
    }

    /// Returns `true` if this argument contains the specified value, `false` otherwise.
    ///
    /// # Example
    /// ```
    /// use clapi::{Command, Argument};
    ///
    /// let result = Command::new("MyApp")
    ///     .arg(Argument::with_name("data"))
    ///     .parse_from(vec!["Hello World"])
    ///     .unwrap();
    ///
    /// assert!(result.get_arg("data").unwrap().contains("Hello World"));
    /// ```
    pub fn contains<S: AsRef<str>>(&self, value: S) -> bool {
        self.get_values().iter().any(|s| s == value.as_ref())
    }

    /// Returns `true` if this argument have default values.
    pub fn has_default_values(&self) -> bool {
        self.default_values.len() > 0
    }

    /// Returns `true` if the given value is valid for this argument.
    ///
    /// # Example
    /// ```
    /// use clapi::{Command, Argument};
    /// use clapi::validator::parse_validator;
    ///
    /// let arg = Argument::with_name("number")
    ///     .validator(parse_validator::<i64>());
    ///
    /// assert!(arg.is_valid("25"));        // Valid `i64` value
    /// assert!(arg.is_valid("-20"));       // Valid `i64` value
    /// assert!(!arg.is_valid("true"));     // Invalid `i64` value
    /// assert!(!arg.is_valid("Hello"));    // Invalid `i64` value
    /// ```
    pub fn is_valid<S: AsRef<str>>(&self, value: S) -> bool {
        if let Some(validator) = &self.validator {
            if validator.validate(value.as_ref()).is_err() {
                return false;
            }
        }

        if self.valid_values.is_empty() {
            true
        } else {
            self.valid_values.iter().any(|s| s == value.as_ref())
        }
    }

    /// Returns `true` if this `Argument` contains values, or false if don't contains values
    /// or only contains default values.
    ///
    /// # Example
    /// ```
    /// use clapi::{Command, Argument};
    ///
    /// let result = Command::new("MyApp")
    ///     .arg(Argument::zero_or_more("data")
    ///         .default("Hello World"))
    ///         .parse_from(Vec::<String>::new())
    ///         .unwrap();
    ///
    /// assert!(!result.arg().unwrap().is_set());
    /// ```
    pub fn is_set(&self) -> bool {
        self.values.is_some()
    }

    /// Sets the number of values this argument takes.
    ///
    /// # Panics
    /// If the value is exactly 0, an argument must take from 0 to 1 values.
    ///
    /// # Example
    /// ```
    /// use clapi::{Command, Argument};
    ///
    /// let command = Command::new("MyApp")
    ///     .arg(Argument::with_name("numbers")
    ///         .values_count(1..=2));
    ///
    /// assert!(command.clone().parse_from(vec!["10"]).is_ok());
    /// assert!(command.clone().parse_from(vec!["10", "20", "30"]).is_err());
    /// ```
    pub fn values_count<A: Into<ArgCount>>(mut self, value_count: A) -> Self {
        let count = value_count.into();
        assert!(!count.takes_exactly(0), "`{}` cannot takes 0 values", self.get_name());
        self.values_count = Some(count);
        self
    }

    /// Sets the min number of values this argument takes.
    ///
    /// # Example
    /// ```
    /// use clapi::{Command, Argument};
    ///
    /// let command = Command::new("MyApp")
    ///     .arg(Argument::with_name("numbers")
    ///         .min_values(1));
    ///
    /// assert!(command.clone().parse_from(vec!["10"]).is_ok());
    /// assert!(command.clone().parse_from(Vec::<String>::new()).is_err());
    /// ```
    pub fn min_values(self, min: usize) -> Self {
        match self.values_count {
            Some(n) => {
                self.values_count(n.with_min(min))
            },
            None => {
                self.values_count(ArgCount::new(Some(min), None))
            },
        }
    }

    /// Sets the max number of values this argument takes.
    ///
    /// # Example
    /// ```
    /// use clapi::{Command, Argument};
    ///
    /// let command = Command::new("MyApp")
    ///     .arg(Argument::with_name("numbers")
    ///         .max_values(2));
    ///
    /// assert!(command.clone().parse_from(vec!["10"]).is_ok());
    /// assert!(command.clone().parse_from(vec!["10", "20", "30"]).is_err());
    /// ```
    pub fn max_values(self, max: usize) -> Self {
        match self.values_count {
            Some(n) => {
                self.values_count(n.with_max(max))
            },
            None => {
                self.values_count(ArgCount::new(None, Some(max)))
            },
        }
    }

    /// Sets the description of this argument.
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the value `Validator` of this argument.
    ///
    /// # Panics
    /// - If there is already a validator.
    /// - If there is default values; a validator must be set before the default values.
    /// - If there is values.
    ///
    /// # Examples
    /// Using the `parse_validator` to ensure the value is an `i64`.
    /// ```
    /// use clapi::{Command, Argument};
    /// use clapi::validator::parse_validator;
    ///
    /// let command = Command::new("MyApp")
    ///     .arg(Argument::with_name("numbers")
    ///         .validator(parse_validator::<i64>()));
    ///
    /// assert!(command.clone().parse_from(vec!["10"]).is_ok());
    /// assert!(command.clone().parse_from(vec!["10", "true"]).is_err());
    /// ```
    ///
    /// Also you can use a closure as a validator in the form: `fn(&str) -> Result<T, String>`.
    /// ```
    /// use clapi::{Command, Argument};
    /// use std::str::FromStr;
    ///
    /// let command = Command::new("MyApp")
    ///     .arg(Argument::with_name("numbers")
    ///         .validator(|s: &str| match i64::from_str(s) {
    ///             // The returned value type will be used
    ///             // as the valid type of the validator
    ///             Ok(v) => Ok(v),
    ///             Err(_) => Err("expected an integer".into())
    ///         }
    ///     ));
    ///
    /// assert!(command.clone().parse_from(vec!["10"]).is_ok());
    /// assert!(command.clone().parse_from(vec!["10", "true"]).is_err());
    /// ```
    pub fn validator<V: Validator + 'static>(mut self, validator: V) -> Self {
        assert!(self.validator.is_none(), "validator is already set");
        assert!(
            self.default_values.is_empty(),
            "validator cannot be set if there is default values"
        );
        assert!(
            self.valid_values.is_empty(),
            "validator cannot be set if there is valid values"
        );
        assert!(
            self.values.is_none(),
            "validator cannot be set if there is values"
        );
        self.validator = Some(Rc::new(validator));
        self
    }

    /// Sets the error message returned when a value is no valid.
    ///
    /// # Example
    /// ```
    /// use clapi::{Command, Argument, Error, ErrorKind};
    /// use clapi::validator::parse_validator;
    /// use std::num::NonZeroUsize;
    ///
    /// let error = Command::new("MyApp")
    ///     .arg(Argument::with_name("number")
    ///         .validator(parse_validator::<NonZeroUsize>())
    ///         .validation_error("expected number greater than 0"))
    ///         .parse_from(vec!["0"])
    ///         .err()
    ///         .unwrap();
    ///
    /// assert_eq!(error.kind(), &ErrorKind::InvalidArgument("expected number greater than 0".to_owned()));
    /// ```
    pub fn validation_error<S: Into<String>>(mut self, error: S) -> Self {
        self.validation_error = Some(error.into());
        self
    }

    /// Sets the valid values of this argument.
    ///
    /// # Panics
    /// If the argument contains default values.
    /// Default values must be set before the valid values.
    ///
    /// # Example
    /// ```
    /// use clapi::{Command, Argument};
    ///
    /// let command = Command::new("MyApp")
    ///     .arg(Argument::with_name("color")
    ///         .valid_values(&["red", "green", "blue"]));
    ///
    /// let result = command.clone().parse_from(vec!["green"]).unwrap();
    /// assert!(result.arg().unwrap().contains("green"));
    ///
    /// // Yellow is an invalid value
    /// assert!(command.clone().parse_from(vec!["yellow"]).is_err());
    /// ```
    pub fn valid_values<S, I>(mut self, values: I) -> Self
    where
        S: ToString,
        I: IntoIterator<Item = S>,
    {
        assert!(
            self.default_values.is_empty(),
            "cannot set valid values when default values are already declared"
        );

        // Keep in mind we aren't removing duplicate values,
        // but duplicates don't have any impact
        let values = values
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        if let Some(validator) = &self.validator {
            for value in &values {
                validator.validate(value).unwrap();
            }
        }

        self.valid_values = values;
        self
    }

    /// Sets the default value of this argument.
    ///
    /// # Panics
    /// - If argument already contains values.
    /// - If already contains default values.
    /// - If the number of arguments is invalid.
    ///
    /// # Example
    /// ```
    /// use clapi::{Command, Argument};
    ///
    /// let command = Command::new("MyApp")
    ///     .arg(Argument::with_name("number")
    ///         .default(0));
    ///
    /// let result_with_value = command.clone().parse_from(vec!["10"]).unwrap();
    /// assert!(result_with_value.arg().unwrap().contains("10"));
    ///
    /// let result = command.clone().parse_from(Vec::<String>::new()).unwrap();
    /// assert!(result.arg().unwrap().contains("0"));
    /// ```
    pub fn default<S: ToString>(self, value: S) -> Self {
        self.defaults(vec![value])
    }

    /// Sets the default values of this argument.
    ///
    /// # Panics
    /// - If argument already contains values.
    /// - If already contains default values.
    /// - If the number of arguments is invalid.
    ///
    /// # Example
    /// ```
    /// use clapi::{Command, Argument};
    ///
    /// let command = Command::new("MyApp")
    ///     .arg(Argument::one_or_more("numbers")
    ///         .defaults(vec![1, 2, 3]));
    ///
    /// let without_values = command.clone().parse_from(Vec::<String>::new()).unwrap();
    /// assert!(without_values.arg().unwrap().contains("1"));
    /// assert!(without_values.arg().unwrap().contains("2"));
    /// assert!(without_values.arg().unwrap().contains("3"));
    ///
    /// let with_values = command.clone().parse_from(vec!["10", "true"]).unwrap();
    /// assert!(with_values.arg().unwrap().contains("10"));
    /// assert!(with_values.arg().unwrap().contains("true"));
    /// ```
    pub fn defaults<S, I>(mut self, values: I) -> Self
    where
        S: ToString,
        I: IntoIterator<Item = S>,
    {
        let values = values
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        assert!(self.get_values().is_empty(), "already contains values");
        assert!(
            self.get_values_count().takes(values.len()),
            "invalid value count expected {} but was {}",
            self.get_values_count(),
            values.len()
        );

        // Validate all the values
        if let Some(validator) = &self.validator {
            for value in &values {
                validator.validate(value).unwrap();
            }
        }

        if !self.valid_values.is_empty() {
            for value in &values {
                if !self.valid_values.iter().any(|s| s == value) {
                    panic!(
                        "invalid default value `{}`, valid values: {}",
                        value,
                        self.valid_values.join(", ")
                    )
                }
            }
        }

        self.default_values = values;
        self
    }

    /// Sets the values of this argument.
    ///
    /// # Example
    /// ```
    /// use clapi::Argument;
    /// use clapi::validator::parse_validator;
    ///
    /// let mut arg = Argument::with_name("number")
    ///     .validator(parse_validator::<i64>())
    ///     .default(0);
    ///
    /// assert!(arg.set_values(vec![2]).is_ok());
    /// assert!(arg.set_values(vec!["hello"]).is_err());
    /// ```
    pub fn set_values<S, I>(&mut self, values: I) -> Result<()>
    where
        S: ToString,
        I: IntoIterator<Item = S>,
    {
        let values = values
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        if !self.get_values_count().takes(values.len()) {
            return Err(Error::new(
                ErrorKind::InvalidArgumentCount,
                format!(
                    "argument `{}` expect {} but was {}",
                    self.get_name(),
                    self.get_values_count(),
                    values.len()
                ),
            ));
        }

        if let Some(validator) = &self.validator {
            for value in &values {
                // Checks if the value is valid
                if let Err(error) = validator.validate(value) {
                    return match self.validation_error.clone() {
                        Some(error) => Err(Error::from(ErrorKind::InvalidArgument(error))),
                        None => Err(error)
                    }
                }
            }
        }

        if !self.valid_values.is_empty() {
            for value in &values {
                if !self.valid_values.iter().any(|s| s == value) {
                    return Err(Error::new(
                        ErrorKind::InvalidArgument(value.clone()),
                        format!("valid values: {}", self.valid_values.join(", ")),
                    ));
                }
            }
        }

        self.values = Some(values);
        Ok(())
    }

    /// Converts the value of this argument to a concrete type.
    ///
    /// # Returns
    /// - `Ok(T)` : If the `String` value can be parse to `T`.
    /// - `Err(error)`:
    ///     - If the value cannot be parse.
    ///     - if there no value to parse.
    ///     - if there is more than 1 value.
    ///
    /// # Example
    /// ```
    /// use clapi::{Command, Argument};
    /// use clapi::validator::parse_validator;
    ///
    /// let result = Command::new("MyApp")
    ///     .arg(Argument::one_or_more("numbers")
    ///         .validator(parse_validator::<i64>()))
    ///     .parse_from(vec!["10"])
    ///     .unwrap();
    ///
    /// assert_eq!(result.get_arg("numbers").unwrap().convert::<i64>().ok(), Some(10));
    /// assert!(result.get_arg("numbers").unwrap().convert::<f32>().is_err());
    /// ```
    pub fn convert<T>(&self) -> Result<T>
    where
        T: FromStr + 'static,
        <T as FromStr>::Err: Display,
    {
        // Checks if the type `T` is valid for the validator
        self.assert_valid_type::<T>()?;

        if self.get_values().is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidArgumentCount,
                "expected at least 1 argument value",
            ));
        }

        if self.get_values().len() != 1 {
            return Err(Error::new(
                ErrorKind::InvalidArgumentCount,
                "multiple argument values found but 1 was expected",
            ));
        }

        try_parse_str(&self.get_values()[0])
    }

    /// Converts the values of this argument to a concrete type.
    ///
    /// # Returns
    /// - `Ok(Vec<T>)` : If all the values are parsed.
    /// - `Err(error)`:
    ///     - If one of the values cannot be parse.
    ///     - if there no values to parse.
    ///
    /// # Example
    /// ```
    /// use clapi::{Command, Argument};
    /// use clapi::validator::parse_validator;
    ///
    /// let result = Command::new("MyApp")
    ///     .arg(Argument::one_or_more("numbers")
    ///         .validator(parse_validator::<i64>()))
    ///     .parse_from(vec!["1", "2", "3"])
    ///     .unwrap();
    ///
    /// assert_eq!(result.get_arg("numbers").unwrap().convert_all::<i64>().ok(), Some(vec![1, 2, 3]));
    /// assert!(result.get_arg("numbers").unwrap().convert_all::<f32>().is_err());
    /// ```
    pub fn convert_all<T>(&self) -> Result<Vec<T>>
    where
        T: FromStr + 'static,
        <T as FromStr>::Err: Display,
    {
        // Checks if the type `T` is valid for the validator
        self.assert_valid_type::<T>()?;

        if self.get_values().is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidArgumentCount,
                "expected at least 1 argument value",
            ));
        }

        let mut ret = Vec::new();
        for value in self.get_values() {
            ret.push(try_parse_str(value)?);
        }
        Ok(ret)
    }

    /// Checks if the type `T` is valid for the validator.
    fn assert_valid_type<T: 'static>(&self) -> Result<()> {
        if let Some(validator) = &self.validator {
            // If the validator returns `None`, we can convert type `T` to any valid type
            // no just the returned by `Validator::valid_type`.
            if let Some(expected) = validator.valid_type() {
                let current = Type::of::<T>();
                if expected != current {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!(
                            "invalid argument type for `{}`, `{}` was expected but was `{}`",
                            self.get_name(),
                            expected.name(),
                            current.name()
                        ),
                    ));
                }
            }
        }

        Ok(())
    }

    #[inline(always)]
    pub(crate) fn set_name_and_description_if_none(&mut self, name: &str, description: Option<&str>) {
        // Ignore all if the `Argument` is named,
        // this method is mostly called when using `Argument::new()`
        if self.name.is_some(){
            return;
        }

        // Sets the same name as than the option
        self.name = Some(name.to_owned());

        // Sets the same description than the option
        if self.description.is_some() && description.is_some() {
            self.description = description.map(|s| s.to_owned());
        }
    }
}

impl Default for Argument {
    #[inline]
    fn default() -> Self {
        Argument::new()
    }
}

impl Eq for Argument {}

impl PartialEq for Argument {
    fn eq(&self, other: &Self) -> bool {
        // This implementation is enough for the purposes of the library
        // but don't reflect the true equality of this struct
        self.get_name() == other.get_name()
    }
}

impl Hash for Argument {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_name().hash(state)
    }
}

impl Debug for Argument {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Argument")
            .field("name", &self.get_name())
            .field("description", &self.get_description())
            .field("values_count", &self.get_values_count())
            .field(
                "validator",
                &if self.validator.is_some() {
                    "Some(Validator)"
                } else {
                    "None"
                },
            )
            .field("default_values", &self.get_default_values())
            .field("valid_values", &self.get_valid_values())
            .field("values", &self.values)
            .finish()
    }
}

impl<I: SliceIndex<[String]>> Index<I> for Argument {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.get_values().index(index)
    }
}

#[doc(hidden)]
pub fn try_parse_str<T: 'static>(value: &str) -> Result<T>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    match T::from_str(value) {
        Ok(n) => Ok(n),
        Err(_) => {
            let type_name = std::any::type_name::<T>();
            Err(Error::new(
                ErrorKind::Other,
                format!("failed to parse `{:?}` to `{}`", value, type_name),
            ))
        }
    }
}

#[doc(hidden)]
pub fn try_parse_values<T: 'static>(values: Vec<String>) -> crate::Result<Vec<T>>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    let mut ret = Vec::new();
    for value in values {
        ret.push(crate::try_parse_str(value.borrow())?);
    }
    Ok(ret)
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct ArgumentList {
    inner: LinkedHashSet<Argument>,
}

impl ArgumentList {
    /// Constructs a new `ArgumentList`.
    pub fn new() -> Self {
        ArgumentList {
            inner: Default::default(),
        }
    }

    /// Returns the number of arguments.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if there is no arguments.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Adds an argument to the list, returns `true` if the argument was added
    /// otherwise if is a duplicate returns `false`.
    ///
    /// # Panics:
    /// Panics if there is multiples options with default values.
    pub fn add(&mut self, arg: Argument) -> std::result::Result<(), Argument> {
        if self.inner.contains(&arg) {
            Err(arg)
        } else {
            self.inner.insert(arg);

            // The list is invalid if:
            // - If there is more than 1 argument with default values
            // - If there is an argument with variable values if other contains default values
            // - If there is more than 1 argument that takes variable values
            self.assert_args();
            Ok(())
        }
    }

    /// Returns the `Argument` with the given name or `None` if no found.
    pub fn get<S: AsRef<str>>(&self, arg_name: S) -> Option<&Argument> {
        self.inner.iter().find(|a| a.get_name() == arg_name.as_ref())
    }

    /// Returns the `String` values of all the arguments of this `ArgumentList`.
    pub fn get_raw_args(&self) -> Vec<String> {
        self.inner
            .iter()
            .flat_map(|arg| arg.get_values())
            .cloned()
            .collect::<Vec<String>>()
    }

    /// Returns the values of all the arguments of this `ArgumentList` and convert them to type `T`.
    ///
    /// # Error
    /// If one of the value cannot be parse to `T`.
    pub fn get_raw_args_as_type<T: 'static>(&self) -> Result<Vec<T>>
    where
        T: std::str::FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Display,
    {
        let mut ret = Vec::new();
        for arg in self.inner.iter() {
            // Check if `T` is valid for the validator
            arg.assert_valid_type::<T>()?;

            for value in arg.get_values() {
                // We use this instead of `convert_all` because it fails
                // if the `Argument` have no values
                ret.push(try_parse_str(value)?);
            }
        }
        Ok(ret)
    }

    /// Returns `true` if contains an argument with the given `name`.
    pub fn contains<S: AsRef<str>>(&self, arg_name: S) -> bool {
        self.inner.iter().any(|a| a.get_name() == arg_name.as_ref())
    }

    /// Converts the value of the `Argument` with the given name.
    ///
    /// # Returns
    /// - `Ok(T)` : If the `String` value of the argument can be parse to `T`.
    /// - `Err(error)`:
    ///     - If the argument cannot be found.
    ///     - If the value cannot be parse.
    ///     - if there no value to parse.
    ///     - if there is more than 1 value.
    pub fn convert<T>(&self, arg_name: &str) -> Result<T>
    where
        T: FromStr + 'static,
        <T as FromStr>::Err: Display, {
        match &self.get(arg_name) {
            Some(arg) => arg.convert(),
            None => Err(Error::from(ErrorKind::InvalidArgument(arg_name.to_owned()))),
        }
    }

    /// Converts the value of the `Argument` with the given index.
    ///
    /// # Panics
    /// If the index is out of bounds.
    ///
    /// # Returns
    /// - `Ok(T)` : If the `String` value of the argument can be parse to `T`.
    /// - `Err(error)`:
    ///     - If the value cannot be parse.
    ///     - if there no value to parse.
    ///     - if there is more than 1 value.
    pub fn convert_at<T>(&self, index: usize) -> Result<T>
    where
        T: FromStr + 'static,
        <T as FromStr>::Err: Display,
    {
        match self.inner.iter().nth(index) {
            Some(arg) => arg.convert(),
            None => panic!(
                "index out of bounds: the len is {} but index was {}",
                self.inner.len(),
                index
            ),
        }
    }

    /// Converts all values of the `Argument` with the given name.
    ///
    /// # Returns
    /// - `Ok(Vec<T>)` : If all the values can be parsed to `T`.
    /// - `Err(error)`:
    ///     - If the argument cannot be found.
    ///     - If one of the values cannot be parse.
    ///     - if there no values to parse.
    pub fn convert_all<T>(&self, arg_name: &str) -> Result<Vec<T>>
    where
        T: FromStr + 'static,
        <T as FromStr>::Err: Display,
    {
        match &self.get(arg_name) {
            Some(arg) => arg.convert_all(),
            None => Err(Error::new(
                ErrorKind::InvalidArgument(arg_name.to_owned()),
                format!("cannot find: `{}`", arg_name),
            )),
        }
    }

    /// Removes all the `Argument`s.
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Returns an `Iterator` over the arguments.
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            iter: self.inner.iter(),
        }
    }

    fn assert_args(&self) {
        if self.len() == 1{
            return;
        }

        // Check if there more than 1 argument with default values.
        //
        // This is not allowed because is no possible to know to what argument a value is being passed to.
        // For example: we have 2 argument with default values: `min` (default 0) `max` (default 10)
        // If we pass: `25` is no possible to know if assign the `25` to `min` or `max`.
        if self.inner.iter().filter(|a| a.has_default_values()).count() > 1 {
            let arg = self.iter()
                .filter(|a| a.has_default_values())
                .nth(1)
                .unwrap();

            panic!("multiple arguments with default values is not allowed: `{}` contains default values", arg.get_name());
        }

        // Check if there is an argument with variable arguments when there is default values
        //
        // This is not allowed because is no possible to know if a value is being passed
        // to the argument with default value.
        // For example: we have 2 arguments: `prefix` (default "hello") and `words` that takes
        // a variable amount of values.
        // If we pass: `Peter Parker` is no possible to know if `Peter` is being passed to `prefix`
        if self.inner.iter().any(|a| a.has_default_values()) {
            if let Some(arg) = self.inner.iter()
                .filter(|arg| !arg.has_default_values())
                .find(|arg| !arg.get_values_count().is_exact()) {
                panic!("arguments with variable values is no allowed if there is default values: `{}` contains variable values", arg.get_name())
            }
        }

        // Check if there is more than 1 argument that take variable values.
        //
        // This is not allowed because the values may overlap.
        // For example: we have 2 arguments: `numbers` (takes 1 to 3) and `ages` (takes 1 to 10)
        // if we pass: -1 0 2 25 10, is no possible to know to what argument the values are being
        // passed
        if self.inner.iter().filter(|a| !a.get_values_count().is_exact()).count() > 1 {
            let arg = self.inner
                .iter()
                .filter(|a| !a.get_values_count().is_exact())
                .nth(1)
                .unwrap();

            panic!("multiple arguments with variable arguments is not allowed: `{}` contains variable values", arg.get_name());
        }
    }
}

/// An iterator over the `Argument`s of a argument list.
#[derive(Debug, Clone)]
pub struct Iter<'a> {
    iter: linked_hash_set::Iter<'a, Argument>,
}

/// An owning iterator over the `Argument`s of a argument list.
pub struct IntoIter {
    iter: linked_hash_set::IntoIter<Argument>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Argument;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl Iterator for IntoIter {
    type Item = Argument;

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

impl<'a> IntoIterator for &'a ArgumentList {
    type Item = &'a Argument;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for ArgumentList {
    type Item = Argument;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.inner.into_iter(),
        }
    }
}

impl Index<usize> for ArgumentList {
    type Output = Argument;

    fn index(&self, index: usize) -> &Self::Output {
        self.inner.iter().nth(index).unwrap_or_else(|| {
            panic!(
                "index out of bounds: len is {} but index was {}",
                self.len(),
                index
            )
        })
    }
}

impl Index<&str> for ArgumentList {
    type Output = Argument;

    #[inline]
    fn index(&self, arg_name: &str) -> &Self::Output {
        self.get(arg_name)
            .unwrap_or_else(|| panic!("cannot find argument: `{}`", arg_name))
    }
}

impl Index<String> for ArgumentList {
    type Output = Argument;

    #[inline]
    fn index(&self, arg_name: String) -> &Self::Output {
        self.index(arg_name.as_str())
    }
}

/// Provides the `Validator` trait used for validate the values of an `Argument`.
pub mod validator {
    use crate::error::{Error, ErrorKind, Result};
    use std::any::TypeId;
    use std::cmp::Ordering;
    use std::fmt::Display;
    use std::hash::{Hash, Hasher};
    use std::marker::PhantomData;
    use std::str::FromStr;
    use crate::ErrorKind::InvalidArgument;

    /// Exposes a method for check if an `str` value is a valid argument value.
    pub trait Validator {
        /// Checks if the given string slice is valid.
        /// Returns `Ok()` if is valid otherwise `Err(error)`.
        fn validate(&self, value: &str) -> Result<()>;

        /// Returns the `Type` that is valid for this `Validator`, by default returns `None`.
        ///
        /// When `None` is returned differents types may be valid for the validator,
        /// for example `"1"` can be valid for types like `i32`, `u64`, `f32`, ...
        /// to ensure the validator is only valid for `u64` the implementor must return: `Some(Type::of::<u64>())`.
        ///
        /// The returned `Type` is used by `Argument::convert` to ensure if safe to convert a type `T`.
        fn valid_type(&self) -> Option<Type> {
            None
        }
    }

    /// A `Validator` where a `str` is considered valid if can be parsed to a type `T`.
    #[derive(Default)]
    pub struct ParseValidator<T>(PhantomData<T>);
    impl<T> ParseValidator<T> {
        #[inline]
        pub fn new() -> Self {
            ParseValidator(PhantomData)
        }
    }
    impl<T: 'static> Validator for ParseValidator<T>
    where
        T: FromStr,
    {
        fn validate(&self, value: &str) -> Result<()> {
            match T::from_str(value) {
                Ok(_) => Ok(()),
                Err(_) => Err(Error::from(ErrorKind::InvalidArgument(value.to_string()))),
            }
        }

        fn valid_type(&self) -> Option<Type> {
            Some(Type::of::<T>())
        }
    }

    /// A `Validator` where a `str` is valid if can be parsed to type `T`
    /// and is within the specified range.
    pub struct RangeValidator<T>(T, T);
    impl<T> RangeValidator<T>
    where
        T: FromStr + PartialOrd + Display,
    {
        #[inline]
        pub fn new(min: T, max: T) -> Self {
            assert!(min < max, "min cannot be greater than max");
            RangeValidator(min, max)
        }
    }
    impl<T: 'static> Validator for RangeValidator<T>
    where
        T: FromStr + PartialOrd + Display,
    {
        fn validate(&self, value: &str) -> Result<()> {
            match T::from_str(value) {
                Err(_) => Err(Error::from(ErrorKind::InvalidArgument(value.to_string()))),
                Ok(n) => {
                    if n >= self.0 && n <= self.1 {
                        Ok(())
                    } else {
                        Err(Error::new(
                            ErrorKind::InvalidArgument(value.to_string()),
                            format!("{} is out of range: {}..{}", n, self.0, self.1),
                        ))
                    }
                }
            }
        }

        fn valid_type(&self) -> Option<Type> {
            Some(Type::of::<T>())
        }
    }

    // This allow to use a closure as a `Validator`
    impl<T: 'static, F> Validator for F where F: Fn(&str) -> std::result::Result<T, String> {
        fn validate(&self, value: &str) -> Result<()> {
            match (self)(value){
                Ok(_) => Ok(()),
                Err(msg) => Err(Error::from(InvalidArgument(msg)))
            }
        }

        fn valid_type(&self) -> Option<Type> {
            Some(Type::of::<T>())
        }
    }

    /// Constructs a `Validator` for the specified type.
    #[inline]
    pub fn parse_validator<T: 'static + FromStr>() -> ParseValidator<T> {
        ParseValidator::new()
    }

    /// Constructs a `Validator` for the given range.
    #[inline]
    pub fn range_validator<T: 'static>(min: T, max: T) -> RangeValidator<T>
    where
        T: FromStr + PartialOrd + Display,
    {
        RangeValidator::new(min, max)
    }

    /// Represents a type.
    ///
    /// # Example
    /// ```
    /// use clapi::validator::Type;
    /// use std::any::TypeId;
    ///
    /// let r#type = Type::of::<i64>();
    /// assert_eq!(r#type.name(), "i64");
    /// assert_eq!(r#type.id(), TypeId::of::<i64>());
    /// ```
    #[derive(Debug, Clone, Copy)]
    pub struct Type {
        type_name: &'static str,
        type_id: TypeId,
    }

    impl Type {
        /// Constructs a new `Type` from the given `T`.
        pub fn of<T: 'static>() -> Self {
            let type_name = std::any::type_name::<T>();
            let type_id = std::any::TypeId::of::<T>();
            Type { type_name, type_id }
        }

        /// Returns the type name of this type.
        pub const fn name(&self) -> &'static str {
            self.type_name
        }

        /// Returns the `TypeId` of this type.
        pub const fn id(&self) -> TypeId {
            self.type_id
        }
    }

    impl Eq for Type {}

    impl PartialEq for Type {
        fn eq(&self, other: &Self) -> bool {
            self.type_id == other.type_id
        }
    }

    impl Ord for Type {
        fn cmp(&self, other: &Self) -> Ordering {
            self.type_id.cmp(&other.type_id)
        }
    }

    impl PartialOrd for Type {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            self.type_id.partial_cmp(&other.type_id)
        }
    }

    impl Hash for Type {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.type_id.hash(state)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::validator::parse_validator;

    #[test]
    fn arg_test() {
        let arg = Argument::with_name("number")
            .description("the values to use")
            .values_count(1..)
            .validator(parse_validator::<i64>())
            .validation_error("expected integer")
            .default(1);

        assert_eq!(arg.get_name(), "number");
        assert_eq!(arg.get_description(), Some("the values to use"));
        assert_eq!(arg.get_values_count(), ArgCount::more_than(1));
        assert!(arg.get_validator().is_some());
        assert_eq!(arg.get_validation_error(), Some("expected integer"));
        assert_eq!(arg.get_default_values()[0].clone(), "1".to_owned());
        assert!(!arg.is_set());
    }

    #[test]
    #[should_panic(expected="argument `name` cannot be empty")]
    fn arg_empty_name_test() {
        Argument::with_name("");
    }

    #[test]
    fn arg_name_with_whitespaces_test() {
        Argument::with_name("my arg");
    }

    #[test]
    fn arg_min_max_values_test() {
        let arg = Argument::with_name("number")
            .min_values(5)
            .max_values(10);

        assert_eq!(arg.get_values_count().min(), Some(5));
        assert_eq!(arg.get_values_count().max(), Some(10));
    }

    #[test]
    fn set_values_test() {
        let mut number = Argument::one_or_more("number").validator(parse_validator::<f64>());

        assert!(!number.is_set());
        assert!(number.set_values(&[1, 2, 3]).is_ok());
        assert!(number.set_values(&[0.2, 5.4]).is_ok());
        assert!(number.set_values(&["1", "0.25", "3"]).is_ok());
        assert!(number.set_values(&[true, false]).is_err());
        assert!(number.is_set());
    }

    #[test]
    fn arg_convert() {
        let mut number = Argument::with_name("number").validator(parse_validator::<i64>());

        number.set_values(&[42]).unwrap();

        assert!(number.convert::<i64>().is_ok());
        assert!(number.convert::<i128>().is_err());
        assert!(number.convert::<bool>().is_err());
        assert_eq!(number.convert::<i64>().ok(), Some(42));
    }

    #[test]
    fn arg_convert_all() {
        let mut number = Argument::one_or_more("numbers").validator(parse_validator::<i64>());

        number.set_values(&[1, 2, 3]).unwrap();

        assert!(number.convert_all::<i64>().is_ok());
        assert!(number.convert::<i8>().is_err());
        assert!(number.convert_all::<bool>().is_err());
        assert_eq!(number.convert_all::<i64>().ok(), Some(vec![1, 2, 3]));
    }

    #[test]
    fn argument_list_test() {
        let mut arg_list = ArgumentList::new();
        let mut colors = Argument::with_name("color");
        colors.set_values(vec!["red"]).unwrap();

        let mut numbers = Argument::one_or_more("number");
        numbers.set_values(vec![1, 2, 3]).unwrap();

        assert!(arg_list.add(colors.clone()).is_ok());
        assert!(arg_list.add(colors).is_err());
        assert!(arg_list.add(numbers).is_ok());
        assert_eq!(arg_list.len(), 2);
        assert_eq!(
            arg_list.get_raw_args(),
            vec![
                "red".to_owned(),
                "1".to_owned(),
                "2".to_owned(),
                "3".to_owned()
            ]
        );
    }

    #[test]
    fn argument_indexer_test(){
        let mut arg = Argument::one_or_more("numbers");
        arg.set_values(vec![1, 2, 3]).unwrap();

        assert_eq!(arg[0].as_str(), "1");
        assert_eq!(arg[1].as_str(), "2");
        assert_eq!(arg[2].as_str(), "3");
        assert_eq!(&arg[1..], &["2".to_owned(), "3".to_owned()]);
    }

    #[test]
    fn argument_list_indexer_test(){
        let mut args = ArgumentList::new();
        args.add(Argument::with_name("number")).unwrap();
        args.add(Argument::one_or_more("values")).unwrap();

        assert_eq!(args["number"].get_name(), "number");
        assert_eq!(args["values"].get_name(), "values");

        assert_eq!(args[0].get_name(), "number");
        assert_eq!(args[1].get_name(), "values");
    }

    #[test]
    #[should_panic(expected="multiple arguments with default values is not allowed: `max` contains default values")]
    fn argument_list_with_default_values_test(){
        let mut args = ArgumentList::new();
        assert!(args.add(Argument::with_name("min").default(0)).is_ok());
        assert!(args.add(Argument::with_name("max").default(i64::max_value())).is_ok());
    }

    #[test]
    #[should_panic(expected="arguments with variable values is no allowed if there is default values: `words` contains variable values")]
    fn argument_list_with_default_values_and_variable_args_test(){
        let mut args = ArgumentList::new();
        assert!(args.add(Argument::with_name("greeting").default("Hello")).is_ok());
        assert!(args.add(Argument::with_name("words").values_count(1..)).is_ok());
    }

    #[test]
    fn argument_list_with_default_values_and_exact_args_test(){
        let mut args = ArgumentList::new();
        assert!(args.add(Argument::with_name("greeting").default("Hello")).is_ok());
        assert!(args.add(Argument::with_name("words").values_count(3)).is_ok());
    }

    #[test]
    #[should_panic(expected="multiple arguments with variable arguments is not allowed: `characters` contains variable values")]
    fn argument_list_with_variable_args_test(){
        let mut args = ArgumentList::new();
        assert!(args.add(Argument::with_name("numbers").values_count(1..10)).is_ok());
        assert!(args.add(Argument::with_name("characters").values_count(1..4)).is_ok());
    }
}
