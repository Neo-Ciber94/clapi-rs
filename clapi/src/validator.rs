use std::fmt::Display;
use std::marker::PhantomData;
use std::str::FromStr;

#[cfg(feature = "typing")]
use crate::typing::Type;

/// Exposes a method for check if an `str` value is a valid argument value.
pub trait Validator {
    /// Checks if the given string slice is valid.
    /// Returns `Ok()` if is valid otherwise `Err(error)`.
    fn validate(&self, value: &str) -> Result<(), String>;

    /// Returns the `Type` that is valid for this `Validator`, by default returns `None`.
    ///
    /// When `None` is returned differents types may be valid for the validator,
    /// for example `"1"` can be valid for types like `i32`, `u64`, `f32`, ...
    /// to ensure the validator is only valid for `u64` the implementor must return: `Some(Type::of::<u64>())`.
    ///
    /// The returned `Type` is used by `Argument::convert` to ensure if safe to convert a type `T`.
    #[cfg(feature = "typing")]
    fn valid_type(&self) -> Option<Type> {
        None
    }
}

/// A `Validator` where a `str` is considered valid if can be parsed to a type `T`.
#[derive(Default)]
pub struct TypeValidator<T>(PhantomData<T>);
impl<T> TypeValidator<T> {
    #[inline]
    pub fn new() -> Self {
        TypeValidator(PhantomData)
    }
}
impl<T: 'static> Validator for TypeValidator<T>
    where
        T: FromStr,
{
    fn validate(&self, value: &str) -> Result<(), String> {
        match T::from_str(value) {
            Ok(_) => Ok(()),
            Err(_) => Err(format!("`{}`", value)),
        }
    }

    #[cfg(feature = "typing")]
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
    fn validate(&self, value: &str) -> Result<(), String> {
        match T::from_str(value) {
            Err(_) => Err(format!("`{}`", value)),
            Ok(n) => {
                if n >= self.0 && n <= self.1 {
                    Ok(())
                } else {
                    Err(format!("{} is out of range: {}..{}", n, self.0, self.1))
                }
            }
        }
    }

    #[cfg(feature = "typing")]
    fn valid_type(&self) -> Option<Type> {
        Some(Type::of::<T>())
    }
}

// This allow to use a closure as a `Validator`
impl<F> Validator for F
    where
        F: Fn(&str) -> std::result::Result<(), String>,
{
    fn validate(&self, value: &str) -> Result<(), String> {
        match (self)(value) {
            Ok(_) => Ok(()),
            Err(msg) => Err(msg),
        }
    }
}

/// Constructs a `Validator` for the specified type.
#[inline]
pub fn validate_type<T: 'static + FromStr>() -> TypeValidator<T> {
    TypeValidator::new()
}

/// Constructs a `Validator` for the given range.
#[inline]
pub fn validate_range<T: 'static>(min: T, max: T) -> RangeValidator<T>
    where
        T: FromStr + PartialOrd + Display,
{
    RangeValidator::new(min, max)
}