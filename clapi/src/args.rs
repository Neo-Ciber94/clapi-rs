use crate::error::Result;
use crate::{ArgCount, Error, ErrorKind};
use linked_hash_set::LinkedHashSet;
use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Index;
use std::rc::Rc;
use std::str::FromStr;
use validator::Type;
use validator::Validator;

/// Represents the arguments of an `option` or `command`.
#[derive(Clone)]
pub struct Argument {
    name: String,
    description: Option<String>,
    arg_count: ArgCount,
    validator: Option<Rc<dyn Validator>>,
    default_values: Vec<String>,
    valid_values: Vec<String>,
    values: Vec<String>,
}

impl Argument {
    /// Constructs a new `Argument`.
    ///
    /// # Panics:
    /// Panics if the argument `name` is blank or empty.
    pub fn new<S: Into<String>>(name: S) -> Self {
        let name = assert_not_blank!(name.into(), "`name` cannot be blank or empty");

        Argument {
            name,
            description: None,
            arg_count: ArgCount::exactly(1),
            validator: None,
            default_values: vec![],
            valid_values: vec![],
            values: vec![],
        }
    }

    /// Constructs a new `Argument` that takes 0 or 1 values.
    ///
    /// # Panics:
    /// Panics if the argument `name` is blank or empty.
    #[inline]
    pub fn zero_or_one<S: Into<String>>(name: S) -> Self {
        Self::new(name).arg_count(0..=1)
    }

    /// Constructs a new `Argument` that takes 0 or more values.
    ///
    /// # Panics:
    /// Panics if the argument `name` is blank or empty.
    #[inline]
    pub fn zero_or_more<S: Into<String>>(name: S) -> Self {
        Self::new(name).arg_count(0..)
    }

    /// Constructs a new `Argument` that takes 1 or more values.
    ///
    /// # Panics:
    /// Panics if the argument `name` is blank or empty.
    #[inline]
    pub fn one_or_more<S: Into<String>>(name: S) -> Self {
        Self::new(name).arg_count(1..)
    }

    /// Returns the name of this argument.
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the description of this argument.
    pub fn get_description(&self) -> Option<&str> {
        self.description.as_ref().map(|s| s.as_str())
    }

    /// Returns the number of values this argument takes.
    pub fn get_arg_count(&self) -> &ArgCount {
        &self.arg_count
    }

    /// Returns the value `Validator` used by this argument.
    pub fn get_validator(&self) -> Option<&dyn Validator> {
        self.validator.as_ref().map(|s| s.as_ref())
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
        self.values.as_slice()
    }

    /// Returns `true` if this argument contains the specified value, `false` otherwise.
    pub fn contains<S: AsRef<str>>(&self, value: S) -> bool {
        self.values.iter().any(|s| s == value.as_ref())
    }

    /// Returns `true` if this argument have default values.
    pub fn has_default_values(&self) -> bool {
        self.default_values.len() > 0
    }

    /// Returns `true` if the given value is valid for this argument.
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

    /// Sets the number of values this argument takes.
    ///
    /// # Panics
    /// If the value is exactly 0, an argument must take from 0 to 1 values.
    pub fn arg_count<A: Into<ArgCount>>(mut self, arg_count: A) -> Self {
        let arg_count = arg_count.into();
        assert!(
            !arg_count.takes_exactly(0),
            "`{}` cannot takes 0 values",
            self.name
        );
        self.arg_count = arg_count;
        self
    }

    /// Sets the description of this argument.
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

    /// Sets the value `Validator` of this argument.
    ///
    /// # Panics
    /// - If there is already a validator.
    /// - If there is default values; a validator must be set before the default values.
    /// - If there is values.
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
            self.values.is_empty(),
            "validator cannot be set if there is values"
        );
        self.validator = Some(Rc::new(validator));
        self
    }

    /// Sets the valid values of this argument.
    ///
    /// # Panics
    /// If the argument contains default values.
    /// Default values must be set before the valid values.
    pub fn valid_values<S, I>(mut self, values: I) -> Self
    where
        S: ToString,
        I: IntoIterator<Item = S>,
    {
        assert!(
            self.default_values.is_empty(),
            "cannot set valid values when default values are already declared"
        );

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
    pub fn default<S: ToString>(self, value: S) -> Self {
        self.defaults(vec![value])
    }

    /// Sets the default values of this argument.
    ///
    /// # Panics
    /// - If argument already contains values.
    /// - If already contains default values.
    /// - If the number of arguments is invalid.
    pub fn defaults<S, I>(mut self, values: I) -> Self
    where
        S: ToString,
        I: IntoIterator<Item = S>,
    {
        let values = values
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        assert!(self.values.is_empty(), "already contains values");
        assert!(
            self.default_values.is_empty(),
            "default values are already set"
        );
        assert!(
            self.arg_count.takes(values.len()),
            "invalid value count expected {} but was {}",
            self.arg_count,
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

        self.default_values = values.clone();
        self.values = values;
        self
    }

    /// Sets the values of this argument.
    pub fn set_values<S, I>(&mut self, values: I) -> Result<()>
    where
        S: ToString,
        I: IntoIterator<Item = S>,
    {
        let values = values
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        if !self.arg_count.takes(values.len()) {
            return Err(Error::new(
                ErrorKind::InvalidArgumentCount,
                format!(
                    "`{}` expect {} but was {}",
                    self.name,
                    self.arg_count,
                    values.len()
                ),
            ));
        }

        if let Some(validator) = &self.validator {
            for value in &values {
                validator.validate(value)?;
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

        self.values = values;
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
    pub fn convert<T>(&self) -> Result<T>
    where
        T: FromStr + 'static,
        <T as FromStr>::Err: Display,
    {
        // Checks if the type `T` is valid for the validator
        self.assert_valid_type::<T>()?;

        if self.values.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidArgumentCount,
                format!("expected at least 1 argument value"),
            ));
        }

        if self.values.len() != 1 {
            return Err(Error::new(
                ErrorKind::InvalidArgumentCount,
                "multiple argument values found but 1 was expected",
            ));
        }

        try_parse_str(&self.values[0])
    }

    /// Converts the values of this argument to a concrete type.
    ///
    /// # Returns
    /// - `Ok(Vec<T>)` : If all the values are parsed.
    /// - `Err(error)`:
    ///     - If one of the values cannot be parse.
    ///     - if there no values to parse.
    pub fn convert_all<T>(&self) -> Result<Vec<T>>
    where
        T: FromStr + 'static,
        <T as FromStr>::Err: Display,
    {
        // Checks if the type `T` is valid for the validator
        self.assert_valid_type::<T>()?;

        if self.values.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidArgumentCount,
                format!("expected at least 1 argument value"),
            ));
        }

        let mut ret = Vec::new();
        for value in &self.values {
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
                            self.name,
                            expected.name(),
                            current.name()
                        ),
                    ));
                }
            }
        }

        Ok(())
    }
}

impl Eq for Argument {}

impl PartialEq for Argument {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Hash for Argument {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.name.as_bytes())
    }
}

impl Debug for Argument {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Argument")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("arg_count", &self.arg_count)
            .field(
                "validator",
                &if self.validator.is_some() {
                    "Some(Validator)"
                } else {
                    "None"
                },
            )
            .field("default_values", &self.default_values)
            .field("values", &self.values)
            .finish()
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
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    let mut ret = Vec::new();
    for value in values {
        ret.push(crate::try_parse_str(value.borrow())?);
    }
    Ok(ret)
}

#[derive(Default, Debug, Clone)]
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
        if self.len() > 0 {
            // When multiple arguments with default values are defined
            // is not possible know to what argument a value is being passed to
            self.assert_no_default_args();
        }

        if self.inner.contains(&arg) {
            Err(arg)
        } else {
            let was_added = self.inner.insert(arg);
            debug_assert!(was_added);
            Ok(())
        }
    }

    /// Returns the `Argument` with the given name or `None` if no found.
    pub fn get<S: AsRef<str>>(&self, arg_name: S) -> Option<&Argument> {
        self.inner.iter().find(|a| a.name == arg_name.as_ref())
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
        self.inner.iter().any(|a| a.name == arg_name.as_ref())
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
        <T as FromStr>::Err: Display,
    {
        match &self.get(arg_name) {
            Some(arg) => arg.convert(),
            None => Err(Error::new(
                ErrorKind::InvalidArgument(arg_name.to_owned()),
                format!("cannot find: `{}`", arg_name),
            )),
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

    fn assert_no_default_args(&self) {
        if let Some(index) = self.inner.iter().rposition(|a| a.has_default_values()) {
            let arg = self.inner.iter().nth(index).unwrap();
            panic!("multiple arguments with default values is not allowed: `{}` contains default values", arg.name);
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

/// Provides the `Validator` trait used for validate the values of an `Arguments`.
pub mod validator {
    use crate::error::{Error, ErrorKind, Result};
    use std::any::TypeId;
    use std::cmp::Ordering;
    use std::fmt::Display;
    use std::hash::{Hash, Hasher};
    use std::marker::PhantomData;
    use std::str::FromStr;

    /// Exposes a method for check if an `str` value is a valid argument value.
    pub trait Validator {
        /// Checks if the given string slice is valid.
        /// Returns `Ok()` if is valid otherwise `Err(error)`.
        fn validate(&self, value: &str) -> Result<()>;

        /// Returns the `Type` that is valid for this `Validator`, by default returns `None`.
        ///
        /// When `None` is returned differents types are valid for the validator,
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

    /// Constructs a `Validator` for the specified type.
    #[inline]
    pub fn parse_validator<T: 'static + FromStr>() -> ParseValidator<T> {
        ParseValidator::new()
    }

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
    /// assert_eq!(r#type.id(), &TypeId::of::<i64>());
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
        pub fn name(&self) -> &'static str {
            self.type_name
        }

        /// Returns the `TypeId` of this type.
        pub fn id(&self) -> &TypeId {
            &self.type_id
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
        let arg = Argument::new("number")
            .description("the values to use")
            .arg_count(1..)
            .validator(parse_validator::<i64>())
            .default(1);

        assert_eq!(arg.get_name(), "number");
        assert_eq!(arg.get_description(), Some("the values to use"));
        assert_eq!(arg.get_arg_count(), &ArgCount::more_than(1));
        assert!(arg.get_validator().is_some());
        assert_eq!(arg.get_default_values()[0].clone(), "1".to_owned());
    }

    #[test]
    fn set_values_test() {
        let mut number = Argument::one_or_more("number").validator(parse_validator::<f64>());

        assert!(number.set_values(&[1, 2, 3]).is_ok());
        assert!(number.set_values(&[0.2, 5.4]).is_ok());
        assert!(number.set_values(&["1", "0.25", "3"]).is_ok());
        assert!(number.set_values(&[true, false]).is_err());
    }

    #[test]
    fn arg_convert() {
        let mut number = Argument::new("number").validator(parse_validator::<i64>());

        number.set_values(&[42]).unwrap();

        assert!(number.convert::<i64>().is_ok());
        assert!(number.convert::<i128>().is_err());
        assert_eq!(number.convert::<i64>().ok(), Some(42));
    }

    #[test]
    fn arg_convert_all() {
        let mut number = Argument::one_or_more("numbers").validator(parse_validator::<i64>());

        number.set_values(&[1, 2, 3]).unwrap();

        assert!(number.convert_all::<i64>().is_ok());
        assert!(number.convert_all::<i128>().is_err());
        assert_eq!(number.convert_all::<i64>().ok(), Some(vec![1, 2, 3]));
    }

    #[test]
    fn argument_list_test() {
        let mut arg_list = ArgumentList::new();
        let mut colors = Argument::new("color");
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
}
