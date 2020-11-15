use crate::arg_count::ArgCount;
use crate::args::validator::{ListValidator, Validator};
use crate::error::{Error, ErrorKind, Result};
use crate::symbol::Symbol;
use linked_hash_set::LinkedHashSet;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;
use std::str::FromStr;

/// Represents the arguments of an option or command.
#[derive(Clone)]
pub struct Arguments {
    pub(crate) parent: Option<Symbol>,
    name: Option<String>,
    arity: ArgCount,
    valid_values: Option<Rc<dyn Validator>>,
    default_values: Vec<String>,
    values: Vec<String>,
}

impl Arguments {
    /// Constructs a new `Arguments` that takes the specified number of values.
    pub fn new<A: Into<ArgCount>>(arg_count: A) -> Self {
        Self {
            parent: None,
            name: None,
            arity: arg_count.into(),
            valid_values: None,
            default_values: Vec::new(),
            values: Vec::new(),
        }
    }

    /// Constructs a new `Arguments` that takes no values.
    #[inline]
    pub fn none() -> Self {
        Self::new(0)
    }

    /// Constructs a new `Arguments` that takes 0 or more values.
    #[inline]
    pub fn zero_or_more() -> Self {
        Self::new(0..)
    }

    /// Constructs a new `Arguments` that takes 0 or 1 value.
    #[inline]
    pub fn zero_or_one() -> Self {
        Self::new(0..=1)
    }

    /// Constructs a new `Arguments` that takes 1 or more values.
    #[inline]
    pub fn one_or_more() -> Self {
        Self::new(1..)
    }

    /// Returns the number of values this takes.
    #[inline]
    pub fn arity(&self) -> ArgCount {
        self.arity
    }

    /// Returns the parent of arguments.
    ///
    /// # Panics
    /// If this `Arguments` is not part of a `Command` or `CommandOption`.
    #[inline]
    pub fn parent(&self) -> Option<&Symbol> {
        self.parent.as_ref()
    }

    /// Returns the name of this `Arguments` or `None` if not set.
    #[inline]
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|s| s.as_str())
    }

    /// Returns the allowed values.
    #[inline]
    pub fn validator(&self) -> Option<&dyn Validator> {
        self.valid_values.as_ref().map(|s| s.as_ref())
    }

    /// Returns the default values.
    #[inline]
    pub fn default_values(&self) -> &[String] {
        self.default_values.as_slice()
    }

    /// Returns the current values.
    #[inline]
    pub fn values(&self) -> &[String] {
        self.values.as_slice()
    }

    /// Returns `true` if `Arguments` contains the specified value.
    #[inline]
    pub fn contains(&self, value: &str) -> bool {
        self.values.iter().any(|s| s == value)
    }

    /// Returns `true` if this `Arguments` takes values.
    #[inline]
    pub fn take_args(&self) -> bool {
        self.arity.takes_args()
    }

    /// Returns `true` if the `Arguments` have default values.
    #[inline]
    pub fn has_default_values(&self) -> bool {
        !self.default_values.is_empty()
    }

    /// Returns `true` if the specified value is a valid for this `Arguments`.
    pub fn is_valid(&self, value: &str) -> bool {
        if let Some(validator) = &self.valid_values {
            validator.is_valid(value).is_ok()
        } else {
            false
        }
    }

    /// Sets the name of this arguments.
    pub fn set_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Sets the default values of this `Arguments`.
    ///
    /// # Panics
    /// - If the default values are already set.
    /// - If the provided default values are not valid values.
    /// - If this takes not args.
    /// - If the number of default values is more than or less than the expected.
    ///
    /// # Example
    /// ```rust
    /// use clapi::args::Arguments;
    ///
    /// let args = Arguments::new(1)
    ///     .set_valid_values(&["zero", "one", "two", "three"])
    ///     .set_default_values(&["zero"]);
    ///
    /// assert!(args.default_values().contains(&String::from("zero")));
    /// ```
    pub fn set_default_values<'a, I, S>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = &'a S>,
        S: ToString + 'a,
    {
        let values = values
            .into_iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>();

        assert!(
            self.default_values.is_empty(),
            already_have_values_msg(&self, true)
        );
        assert!(
            self.take_args(),
            take_no_values_msg(&self)
        );
        assert!(
            self.values.is_empty(),
            already_have_values_msg(&self, false)
        );
        assert!(
            self.arity.contains(values.len()),
            "invalid number of default values, {} was get but {} was expected",
            values.len(),
            self.arity
        );

        for value in values {
            let s = value.to_string();

            if let Some(validator) = &self.valid_values {
                validator.is_valid(s.as_str()).unwrap();
            }

            self.default_values.push(s);
        }

        // Sets the default values
        self.values = self.default_values.clone();
        self
    }

    /// Sets the valid values of this `Arguments`.
    ///
    /// # Panics
    /// - If the valid values are already set.
    /// - If this take no args.
    /// - If this already contains default values or values.
    ///
    /// # Example
    /// ```rust
    /// use clapi::args::Arguments;
    ///
    /// let mut args = Arguments::new(1)
    ///     .set_valid_values(&["zero", "one", "two", "three"])
    ///     .set_default_values(&["zero"]);
    ///
    /// assert!(args.set_values(&["four"]).is_err());
    /// assert!(args.set_values(&["one"]).is_ok());
    /// assert!(args.set_values(&["two"]).is_ok());
    /// ````
    pub fn set_valid_values<'a, I, S>(self, values: I) -> Self
    where
        I: IntoIterator<Item = &'a S>,
        S: ToString + 'a,
    {
        let values = values
            .into_iter()
            .map(ToString::to_string)
            .collect::<LinkedHashSet<String>>();

        self.set_validator(ListValidator(values))
    }

    /// Sets the validator for the valid values of this `Arguments`.
    pub fn set_validator<V: Validator + 'static>(mut self, validator: V) -> Self {
        assert!(self.valid_values.is_none(), "this `Arguments` validator is already set");
        assert!(self.take_args(), "this `Arguments` takes not values");
        assert!(self.values.is_empty(), "this `Arguments` already have values");

        self.valid_values = Some(Rc::new(validator));
        self
    }

    /// Sets the values of this `Arguments`.
    ///
    /// # Error
    /// - If takes not args.
    /// - If the values are not valid.
    /// - If is an invalid number of values.
    ///
    /// # Example
    /// ```rust
    /// use clapi::args::Arguments;
    /// use clapi::args::validator::validator_for;
    ///
    /// let mut args = Arguments::new(2)
    ///     .set_validator(validator_for::<i32>());
    ///
    /// assert!(args.set_values(&[1, 2]).is_ok());
    /// assert!(args.set_values(&["3", "4"]).is_ok());
    /// assert!(args.set_values(&[5]).is_err());
    /// assert!(args.set_values(&["six", "seven"]).is_err());
    /// ````
    pub fn set_values<'a, I, S>(&mut self, values: I) -> Result<()>
    where
        I: IntoIterator<Item = &'a S>,
        S: ToString + 'a,
    {
        let values = values
            .into_iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>();

        if values.len() == 0 && !self.arity.takes_args() {
            return Ok(());
        }

        if !self.arity.takes_args() {
            return Err(Error::new(
                ErrorKind::InvalidArgumentCount,
                take_no_values_msg(&self),
            ));
        }

        if !self.arity.contains(values.len()) {
            return Err(Error::new(
                ErrorKind::InvalidArgumentCount,
                invalid_arg_count_msg(&self, values.len()),
            ));
        }

        // Removes the contents in case have values
        self.values.clear();

        for value in values {
            if let Some(validator) = &self.valid_values {
                validator.is_valid(value.as_str())?;
            }

            self.values.push(value);
        }

        Ok(())
    }

    /// Converts the first value into the specified type.
    ///
    /// # Error
    /// - If there is no values to convert.
    /// - If this takes not args.
    /// - The value cannot be converted to type `T`.
    pub fn convert<T>(&self) -> Result<T>
    where
        T: FromStr + 'static,
        <T as FromStr>::Err: Display,
    {
        if self.values.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidArgumentCount,
                format!("expected {} but {} was found", 1, self.arity)
            ));
            //return Err(Error::from(ErrorKind::InvalidArgumentCount));
        }

        if self.values.len() != 1 {
            return Err(Error::new(ErrorKind::InvalidArgumentCount, "multiple values found but 1 was expected"));
        }

        if self.arity.takes_args() {
            try_parse_str(self.values[0].as_str())
        } else {
            Err(Error::new(
                ErrorKind::InvalidArgumentCount,
                "takes not args",
            ))
        }
    }

    /// Converts the value at the given index into the specified type.
    ///
    /// # Error
    /// - If the index is out of bounds.
    /// - If there is no values to convert.
    /// - If this takes not args.
    /// - The value cannot be converted to type `T`.
    pub fn convert_at<T>(&self, index: usize) -> Result<T>
    where
        T: FromStr + 'static,
        <T as FromStr>::Err: Display,
    {
        if self.values.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidArgumentCount,
                format!("expected {} but {} was found", index, self.arity)
            ));
            //return Err(Error::from(ErrorKind::InvalidArgumentCount));
        }

        if self.arity.takes_args() {
            if index > self.values.len() {
                return Err(Error::new(
                    ErrorKind::Unknown,
                    format!(
                        "index out of bounds: length is {} but index is {}",
                        self.values.len(),
                        index
                    ),
                ));
            }

            try_parse_str(self.values[index].as_str())
        } else {
            Err(Error::new(
                ErrorKind::InvalidArgumentCount,
                "takes not args",
            ))
        }
    }

    /// Returns an iterator that converts the values into the specified type.
    ///
    /// # Error
    /// - If there is no values to convert.
    /// - If this takes not args.
    /// - One of the values cannot be converted to type `T`.
    pub fn convert_all<T>(&self) -> Result<Vec<T>>
    where
        T: FromStr + 'static,
        <T as FromStr>::Err: Display,
    {
        if self.values.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidArgumentCount,
                "no values found"
            ));
            //return Err(Error::from(ErrorKind::InvalidArgumentCount));
        }

        if self.arity.takes_args() {
            let mut ret = Vec::new();
            for value in &self.values {
                ret.push(try_parse_str(value)?);
            }
            Ok(ret)
        } else {
            Err(Error::new(
                ErrorKind::InvalidArgumentCount,
                "takes not args",
            ))
        }
    }
}

impl Eq for Arguments {}

impl PartialEq for Arguments {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}

impl Debug for Arguments {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Arguments")
            .field("parent", &self.parent)
            .field("name", &self.name)
            .field("arity", &self.arity)
            .field(
                "valid_values",
                &if self.valid_values.is_some() {
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

impl<'a> IntoIterator for &'a Arguments {
    type Item = &'a String;
    type IntoIter = std::slice::Iter<'a, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.iter()
    }
}

fn try_parse_str<T: 'static>(value: &str) -> Result<T>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    match T::from_str(value) {
        Ok(n) => Ok(n),
        Err(error) => {
            let type_name = std::any::type_name::<T>();
            return Err(Error::new(
                ErrorKind::Unknown,
            format!("failed to parse `{:?}` to `{}`\n{}", value, type_name, error))
            );
        },
    }
}

fn take_no_values_msg(args: &Arguments) -> String {
    if let Some(parent) = &args.parent {
        let kind = match parent {
            Symbol::Command(_) => "command",
            Symbol::Option(_) => "option"
        };
        return format!("{} `{}` takes no args", kind, parent.name());
    } else if let Some(name) = &args.name {
        return format!("`{}` takes no args", name);
    } else {
        "takes not args".to_owned()
    }
}

fn already_have_values_msg(args: &Arguments, is_default_values: bool) -> String {
    if let Some(parent) = &args.parent {
        let kind = match parent {
            Symbol::Command(_) => "command",
            Symbol::Option(_) => "option"
        };
        format!("{} `{}` already have {}values", kind, parent.name(), if is_default_values { "default" } else {""})
    } else if let Some(name) = &args.name {
        format!("`{}` already have {}values", name, if is_default_values { "default" } else {""})
    } else {
        format!("already have {}values", if is_default_values { "default" } else {""})
    }
}

fn invalid_arg_count_msg(args: &Arguments, actual: usize) -> String {
    if let Some(parent) = &args.parent {
        let kind = match parent {
            Symbol::Command(_) => "command",
            Symbol::Option(_) => "option"
        };

        format!("{} `{}` expected {} but was {}", kind, parent.name(), args.arity, actual)
    } else if let Some(name) = &args.name {
        format!("`{}` expected {} but was {}", name, args.arity, actual)
    } else {
        format!("expected {} but was {}", args.arity, actual)
    }
}

#[allow(dead_code)]
pub mod validator {
    use crate::error::{Error, ErrorKind, Result};
    use linked_hash_set::LinkedHashSet;
    use std::fmt::Display;
    use std::marker::PhantomData;
    use std::str::FromStr;

    pub trait Validator {
        fn is_valid(&self, value: &str) -> Result<()>;
    }

    #[derive(Default)]
    pub struct DefaultValidator<T>(PhantomData<T>);
    impl<T> DefaultValidator<T> {
        pub fn new() -> Self {
            DefaultValidator(PhantomData)
        }
    }
    impl<T: FromStr> Validator for DefaultValidator<T> {
        fn is_valid(&self, value: &str) -> Result<()> {
            match T::from_str(value) {
                Ok(_) => Ok(()),
                Err(_) => Err(Error::from(ErrorKind::InvalidArgument(value.to_string()))),
            }
        }
    }

    pub struct ListValidator(pub LinkedHashSet<String>);
    impl Validator for ListValidator {
        fn is_valid(&self, value: &str) -> Result<()> {
            if self.0.iter().any(|s| s == value) {
                Ok(())
            } else {
                Err(Error::new(
                    ErrorKind::InvalidArgument(value.to_string()),
                    format!("valid values: {:?}", self.0),
                ))
            }
        }
    }

    pub struct RangeValidator<T>(T, T);
    impl<T: FromStr + PartialOrd + Display> RangeValidator<T> {
        pub fn new(min: T, max: T) -> Self {
            assert!(min < max, "min cannot be greater than max");
            RangeValidator(min, max)
        }
    }
    impl<T: FromStr + PartialOrd + Display> Validator for RangeValidator<T> {
        fn is_valid(&self, value: &str) -> Result<()> {
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
    }

    #[inline]
    pub fn validator_for<T: FromStr>() -> DefaultValidator<T> {
        DefaultValidator::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::validator::validator_for;

    #[test]
    fn arity_test() {
        assert_eq!(Arguments::new(1).arity(), 1.into());
        assert_eq!(Arguments::new(0..2).arity(), (0..2).into());
        assert_eq!(Arguments::none().arity(), 0.into());
        assert_eq!(Arguments::zero_or_one().arity(), (0..=1).into());
        assert_eq!(Arguments::zero_or_more().arity(), (0..).into());
        assert_eq!(Arguments::one_or_more().arity(), (1..).into());
    }

    #[test]
    #[should_panic]
    fn valid_values_panic_test() {
        let _args = Arguments::new(0).set_valid_values(&[1, 2, 3]);
    }

    #[test]
    fn is_valid_test() {
        let args = Arguments::new(1).set_valid_values(&["id", "name", "age"]);

        assert!(args.is_valid("id"));
        assert!(args.is_valid("name"));
        assert!(args.is_valid("age"));
    }

    #[test]
    #[should_panic]
    fn default_values_panic_test() {
        let _args = Arguments::new(1).set_default_values(&[1, 2, 3]);
    }

    #[test]
    fn default_values_test() {
        let args = Arguments::new(3).set_default_values(&[1, 2, 3]);

        assert!(args.default_values().iter().any(|n| n == "1"));
        assert!(args.default_values().iter().any(|n| n == "2"));
        assert!(args.default_values().iter().any(|n| n == "3"));
    }

    #[test]
    fn set_values_test() {
        let mut args1 = Arguments::new(1);
        assert!(args1.set_values(&[1]).is_ok());
        assert!(args1.values().iter().any(|s| s == "1"));

        let mut args2 = Arguments::new(1..);
        assert!(args2.set_values(&[1, 2, 3, 4]).is_ok());
        assert!(args2.values().iter().any(|s| s == "1"));
        assert!(args2.values().iter().any(|s| s == "2"));
        assert!(args2.values().iter().any(|s| s == "3"));
        assert!(args2.values().iter().any(|s| s == "4"));

        let mut args3 = Arguments::new(1..3);
        assert!(args3.set_values(&[1, 2, 3]).is_ok());
        assert!(args3.values().iter().any(|s| s == "1"));
        assert!(args3.values().iter().any(|s| s == "2"));
        assert!(args3.values().iter().any(|s| s == "3"));

        let mut args4 = Arguments::none();
        assert!(args4.set_values(&[1]).is_err());

        let mut args5 = Arguments::new(1..);
        let empty: &[usize] = &[];
        assert!(args5.set_values(empty).is_err());

        let mut args6 = Arguments::new(1..3);
        assert!(args6.set_values(&[1, 2, 3, 4]).is_err());
    }

    #[test]
    fn validator_test1() {
        let mut args = Arguments::new(1).set_validator(validator_for::<u32>());

        assert!(args.set_values(&[1]).is_ok());
        assert!(args.set_values(&["2"]).is_ok());
        assert!(args.set_values(&["-3"]).is_err());
        assert!(args.set_values(&["hello"]).is_err());
    }

    #[test]
    fn validator_test2() {
        let mut args = Arguments::new(1).set_validator(validator_for::<f64>());

        assert!(args.set_values(&[1]).is_ok());
        assert!(args.set_values(&["2.5"]).is_ok());
        assert!(args.set_values(&["-0.0065"]).is_ok());
        assert!(args.set_values(&["hello"]).is_err());
    }

    #[test]
    fn validator_test3() {
        let mut args = Arguments::new(1).set_validator(validator_for::<bool>());

        assert!(args.set_values(&[true]).is_ok());
        assert!(args.set_values(&[false]).is_ok());
        assert!(args.set_values(&["false"]).is_ok());
        assert!(args.set_values(&["hello"]).is_err());
    }

    #[test]
    fn convert_ok_test() {
        let mut args = Arguments::new(1..).set_name("numbers");
        args.set_values(&["1"]).unwrap();
        assert_eq!(args.convert::<u32>().ok(), Some(1));
    }

    #[test]
    fn convert_err_test() {
        let mut args = Arguments::new(1..).set_name("numbers");
        args.set_values(&["1", "2", "3"]).unwrap();
        assert!(args.convert::<u32>().is_err());
    }

    #[test]
    fn convert_at_test() {
        let mut args = Arguments::new(1..).set_name("numbers");

        args.set_values(&["1", "2", "3"]).unwrap();

        assert_eq!(1, args.convert_at(0).unwrap());
        assert_eq!(2, args.convert_at(1).unwrap());
        assert_eq!(3, args.convert_at(2).unwrap());
    }

    #[test]
    fn convert_all_test() {
        let mut args = Arguments::new(1..).set_name("numbers");
        args.set_values(&["1", "2", "3"]).unwrap();

        let values = args.convert_all::<u32>().unwrap();
        assert_eq!(vec![1, 2, 3], values);
    }

    #[test]
    fn convert_all_err_test() {
        let mut args = Arguments::new(1..);
        args.set_values(&["1", "bool", "3"]).unwrap();
        assert!(args.convert_all::<u32>().is_err());
    }
}
