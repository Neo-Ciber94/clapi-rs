use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Index;
use std::rc::Rc;
use std::str::FromStr;
use linked_hash_set::LinkedHashSet;
use validator::Validator;

use crate::{ArgCount, Error, ErrorKind};
use crate::error::Result;

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
    pub fn new<S: Into<String>>(name: S) -> Self {
        Argument {
            name: name.into(),
            description: None,
            arg_count: ArgCount::exactly(1),
            validator: None,
            default_values: vec![],
            valid_values: vec![],
            values: vec![],
        }
    }

    #[inline]
    pub fn zero_or_one<S: Into<String>>(name: S) -> Self {
        Self::new(name).arg_count(0..=1)
    }

    #[inline]
    pub fn zero_or_more<S: Into<String>>(name: S) -> Self {
        Self::new(name).arg_count(0..)
    }

    #[inline]
    pub fn one_or_more<S: Into<String>>(name: S) -> Self {
        Self::new(name).arg_count(1..)
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_description(&self) -> Option<&str> {
        self.description.as_ref().map(|s| s.as_str())
    }

    pub fn get_arg_count(&self) -> ArgCount {
        self.arg_count
    }

    pub fn get_validator(&self) -> Option<&dyn Validator> {
        self.validator.as_ref().map(|s| s.as_ref())
    }

    pub fn get_default_values(&self) -> &[String] {
        self.default_values.as_slice()
    }

    pub fn get_valid_values(&self) -> &[String]{
        self.valid_values.as_slice()
    }

    pub fn get_values(&self) -> &[String] {
        self.values.as_slice()
    }

    pub fn contains<S: AsRef<str>>(&self, value: S) -> bool {
        self.values.iter().any(|s| s == value.as_ref())
    }

    pub fn has_default_values(&self) -> bool {
        self.default_values.len() > 0
    }

    pub fn is_valid<S: AsRef<str>>(&self, value: S) -> bool {
        if let Some(validator) = &self.validator {
            if validator.validate(value.as_ref()).is_err(){
                return false;
            }
        }

        if self.valid_values.is_empty(){
            true
        } else {
            self.valid_values.iter().any(|s| s == value.as_ref())
        }
    }

    pub fn arg_count<A: Into<ArgCount>>(mut self, arg_count: A) -> Self {
        let arg_count = arg_count.into();
        assert!(!arg_count.takes_exactly(0), "`{}` cannot takes 0 values", self.name);
        self.arg_count = arg_count;
        self
    }

    pub fn description<S: Into<String>>(mut self, name: S) -> Self {
        self.description = Some(name.into());
        self
    }

    pub fn validator<V: Validator + 'static>(mut self, validator: V) -> Self {
        assert!(self.validator.is_none(), "validator is already set");
        assert!(self.default_values.is_empty(), "validator cannot be set if there is default values");
        assert!(self.valid_values.is_empty(), "validator cannot be set if there is valid values");
        assert!(self.values.is_empty(), "validator cannot be set if there is values");
        self.validator = Some(Rc::new(validator));
        self
    }

    pub fn valid_values<S, I>(mut self, values: I) -> Self
    where
        S: ToString,
        I: IntoIterator<Item = S>,
    {
        assert!(self.default_values.is_empty(), "cannot set valid values when default values are already declared");

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

    pub fn default<S: ToString>(self, value: S) -> Self {
        self.defaults(vec![value])
    }

    pub fn defaults<S, I>(mut self, values: I) -> Self
    where
        S: ToString,
        I: IntoIterator<Item = S>,
    {
        let values = values
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        assert!(!values.is_empty(), "no values");
        assert!(self.default_values.is_empty(), "default values are already set");
        assert!(self.values.is_empty(), "already contains values");
        assert!(
            self.arg_count.contains(values.len()),
            "invalid argument count expected {} but was {}",
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
                if !self.valid_values.iter().any(|s| s == value){
                    panic!("invalid default value `{}`, valid values: {}", value, self.valid_values.join(", "))
                }
            }
        }

        self.default_values = values.clone();
        self.values = values;
        self
    }

    pub fn set_values<S, I>(&mut self, values: I) -> Result<()>
    where
        S: ToString,
        I: IntoIterator<Item = S>,
    {
        let values = values
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        if !self.arg_count.contains(values.len()) {
            return Err(Error::new(
                ErrorKind::InvalidArgumentCount,
                format!("`{}` expect {} but was {}", self.name, self.arg_count, values.len()),
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
                        format!("valid values: {}", self.valid_values.join(", "))
                    ));
                }
            }
        }

        self.values = values;
        Ok(())
    }

    pub fn convert<T>(&self) -> Result<T>
        where
            T: FromStr + 'static,
            <T as FromStr>::Err: Display,
    {
        if self.values.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidArgumentCount,
                format!("expected at least 1 argument value")
            ));
        }

        if self.values.len() != 1 {
            return Err(Error::new(
                ErrorKind::InvalidArgumentCount,
                "multiple argument values found but 1 was expected")
            );
        }

        try_parse_str(&self.values[0])
    }

    pub fn convert_all<T>(&self) -> Result<Vec<T>>
        where
            T: FromStr + 'static,
            <T as FromStr>::Err: Display,
    {
        if self.values.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidArgumentCount,
                format!("expected at least 1 argument value")
            ));
        }

        let mut ret = Vec::new();
        for value in &self.values {
            ret.push(try_parse_str(value)?);
        }
        Ok(ret)
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
            .field("validator", &if self.validator.is_some() { "Some(Validator)" } else { "None" })
            .field("default_values", &self.default_values)
            .field("values", &self.values)
            .finish()
    }
}

fn try_parse_str<T: 'static>(value: &str) -> Result<T>
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

#[derive(Default, Debug, Clone)]
pub struct ArgumentList {
    inner: LinkedHashSet<Argument>,
}

impl ArgumentList {
    pub fn new() -> Self {
        ArgumentList {
            inner: Default::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn add(&mut self, arg: Argument) -> bool {
        if self.len() > 0 {
            // When multiple arguments with default values are defined
            // is not possible know to what argument a value is being passed to
            self.assert_no_default_args();
        }

        self.inner.insert_if_absent(arg)
    }

    pub fn get<S: AsRef<str>>(&self, arg_name: S) -> Option<&Argument>{
        self.inner.iter().find(|a| a.name == arg_name.as_ref())
    }

    pub fn get_raw_args(&self) -> Vec<String>{
        let mut ret = Vec::new();
        for arg in &self.inner {
            ret.extend(arg.values.clone());
        }
        ret
    }

    pub fn contains<S: AsRef<str>>(&self, arg_name: S) -> bool {
        self.inner.iter().any(|a| a.name == arg_name.as_ref())
    }

    pub fn convert<T>(&self, arg_name: &str) -> Result<T>
        where T: FromStr + 'static,
            <T as FromStr>::Err : Display{

        match &self.get(arg_name) {
            Some(arg) => arg.convert(),
            None => Err(Error::new(
                ErrorKind::InvalidArgument(arg_name.to_owned()),
            format!("cannot find: `{}`", arg_name))
            )
        }
    }

    pub fn convert_at<T>(&self, index: usize) -> Result<T>
        where T: FromStr + 'static,
              <T as FromStr>::Err : Display{

        match self.inner.iter().nth(index){
            Some(arg) => arg.convert(),
            None => panic!("index out of bounds: the len is {} but index was {}", self.inner.len(), index)
        }
    }

    pub fn convert_all<T>(&self, arg_name: &str) -> Result<Vec<T>>
        where T: FromStr + 'static,
              <T as FromStr>::Err : Display{

        match &self.get(arg_name) {
            Some(arg) => arg.convert_all(),
            None => Err(Error::new(
                ErrorKind::InvalidArgument(arg_name.to_owned()),
                format!("cannot find: `{}`", arg_name))
            )
        }
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item=&Argument>{
        self.inner.iter()
    }

    fn assert_no_default_args(&self){
        if let Some(index) = self.inner.iter().rposition(|a| a.has_default_values()){
            let arg = self.inner.iter().nth(index).unwrap();
            panic!("multiple arguments with default values is not allowed: `{}` contains default values", arg.name);
        }
    }
}

impl Index<usize> for ArgumentList {
    type Output = Argument;

    fn index(&self, index: usize) -> &Self::Output {
        self.inner.iter().nth(index)
            .unwrap_or_else(|| panic!("index out of bounds: len is {} but index was {}", self.len(), index))
    }
}

impl Index<&str> for ArgumentList{
    type Output = Argument;

    #[inline]
    fn index(&self, arg_name: &str) -> &Self::Output {
        self.get(arg_name)
            .unwrap_or_else(|| panic!("cannot find argument: `{}`", arg_name))
    }
}

impl Index<String> for ArgumentList{
    type Output = Argument;

    #[inline]
    fn index(&self, arg_name: String) -> &Self::Output {
        self.index(arg_name.as_str())
    }
}

impl IntoIterator for ArgumentList{
    type Item = Argument;
    type IntoIter = linked_hash_set::IntoIter<Argument>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a> IntoIterator for &'a ArgumentList{
    type Item = &'a Argument;
    type IntoIter = linked_hash_set::Iter<'a, Argument>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

/// Provides the `Validator` trait used for validate the values of an `Arguments`.
pub mod validator {
    use std::fmt::Display;
    use std::marker::PhantomData;
    use std::str::FromStr;
    use crate::error::{Error, ErrorKind, Result};

    /// Exposes a method for check if an `str` value is a valid argument value.
    pub trait Validator {
        fn validate(&self, value: &str) -> Result<()>;
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
    impl<T: FromStr> Validator for ParseValidator<T> {
        fn validate(&self, value: &str) -> Result<()> {
            match T::from_str(value) {
                Ok(_) => Ok(()),
                Err(_) => Err(Error::from(ErrorKind::InvalidArgument(value.to_string()))),
            }
        }
    }

    /// A `Validator` where a `str` is valid if can be parsed to type `T`
/// and is within the specified range.
    pub struct RangeValidator<T>(T, T);
    impl<T: FromStr + PartialOrd + Display> RangeValidator<T> {
        #[inline]
        pub fn new(min: T, max: T) -> Self {
            assert!(min < max, "min cannot be greater than max");
            RangeValidator(min, max)
        }
    }
    impl<T: FromStr + PartialOrd + Display> Validator for RangeValidator<T> {
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
    }

    /// Constructs a `Validator` for the specified type.
    #[inline]
    pub fn parse_validator<T: FromStr>() -> ParseValidator<T> {
        ParseValidator::new()
    }

    #[inline]
    pub fn range_validator<T>(min: T, max: T) -> RangeValidator<T>
        where T: FromStr + PartialOrd + Display {
        RangeValidator::new(min, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::validator::parse_validator;

    #[test]
    fn arg_test(){
        let arg = Argument::new("number")
            .description("the values to use")
            .arg_count(1..)
            .validator(parse_validator::<i64>())
            .default(1);

        assert_eq!(arg.get_name(), "number");
        assert_eq!(arg.get_description(), Some("the values to use"));
        assert_eq!(arg.get_arg_count(), ArgCount::more_than(1));
        assert!(arg.get_validator().is_some());
        assert_eq!(arg.get_default_values()[0].clone(), "1".to_owned());
    }
}