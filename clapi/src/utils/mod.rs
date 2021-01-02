#![allow(dead_code)]
#![allow(unused_must_use)]

/// Extension methods.
mod ext;
pub use ext::*;

#[macro_use]
mod macros;
pub use macros::*;

/// Performs an operation over `Self` and returns the result.
///
/// This trait is implemented for all types.
pub trait Then: Sized {
    /// Performs an operation over `&Self` and returns the result.
    #[inline]
    fn then<'a, R, F: Fn(&'a Self) -> R>(&'a self, f: F) -> R {
        f(self)
    }

    /// Performs an operation over `&mut Self` and returns the result.
    #[inline]
    fn then_mut<'a, R, F: FnMut(&'a mut Self) -> R>(&'a mut self, mut f: F) -> R {
        f(self)
    }

    /// Performs an operation over `Self` and returns the result.
    ///
    /// # Example
    /// ```
    /// use clapi::utils::Then;
    ///
    /// let value = "Hello World".then_apply(|s| {
    ///    if s.is_empty() { None } else { Some(s.to_owned()) }
    /// });
    ///
    /// assert_eq!(value, Some("Hello World".to_owned()));
    /// ```
    #[inline]
    fn then_apply<R, F: FnOnce(Self) -> R>(self, f: F) -> R {
        f(self)
    }
}

/// Performs an operation over `Self` and returns `Self`.
///
/// This trait is implemented for all types.
pub trait Also: Sized {
    /// Performs an operation over `&Self`.
    #[inline]
    fn also<R, F: Fn(&Self) -> R>(self, f: F) -> Self {
        f(&self);
        self
    }

    /// Performs an operation over `&mut Self`.
    ///
    /// # Example
    /// ```
    /// use clapi::utils::Also;
    ///
    /// let values = vec![1, 2, 3].also_mut(|v| v.extend(&[4, 5, 6]));
    /// assert_eq!(values, vec![1, 2, 3, 4, 5, 6]);
    /// ```
    #[inline]
    fn also_mut<R, F: Fn(&mut Self) -> R>(mut self, f: F) -> Self {
        f(&mut self);
        self
    }

    /// Performs an operation over `Self`.
    #[inline]
    fn also_apply<R, F: FnOnce(Self) -> Self>(self, f: F) -> Self {
        f(self)
    }
}

impl<T> Then for T {}
impl<T> Also for T {}

pub use debug_utils::*;
mod debug_utils {
    use std::fmt::{Debug, Formatter};

    pub fn debug_option<'a, T>(option: &'a Option<T>, if_some: &'a str) -> impl Debug + 'a {
        struct OptionDebug<'a, T> {
            option: &'a Option<T>,
            if_some: &'a str,
        }

        impl<T> Debug for OptionDebug<'_, T> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                match &self.option {
                    None => write!(f, "None"),
                    Some(_) => write!(f, "Some({})", self.if_some),
                }
            }
        }

        OptionDebug { option, if_some }
    }
}
